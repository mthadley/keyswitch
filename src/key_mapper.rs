use input_linux::{Key, KeyEvent, KeyState};
use linked_hash_set::LinkedHashSet;
use std::{
    collections::{HashMap, HashSet},
    vec::Vec,
};

pub struct KeyMapper {
    mappings: Vec<Mapping>,
    pressed_keys: HashSet<Key>,

    /// Keys that were already released to isolate a mapping.
    already_released: LinkedHashSet<Key>,

    /// Keys that were previously mapped, which we'll need to be able to identify
    /// again if their prefixes are no longer held.
    mapped_keys: HashMap<Key, Key>,
}

struct Mapping {
    prefixes: Vec<Key>,
    old: Key,
    new: Key,
}

impl KeyMapper {
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            pressed_keys: HashSet::new(),
            already_released: LinkedHashSet::new(),
            mapped_keys: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, keys: &[Key], new: &Key) -> Result<(), Error> {
        match keys.split_last() {
            None => Err(Error::EmptyMappingError),
            Some((old, prefixes)) => {
                self.mappings.push(Mapping {
                    prefixes: Vec::from(prefixes),
                    old: *old,
                    new: *new,
                });
                Ok(())
            }
        }
    }

    pub fn handle_key_event(&mut self, event: &KeyEvent) -> Vec<(Key, KeyState)> {
        let matched_mapping = match event.value {
            KeyState::PRESSED | KeyState::AUTOREPEAT => {
                self.mappings.iter().find(|Mapping { prefixes, old, .. }| {
                    *old == event.key && self.all_pressed(prefixes)
                })
            }
            _ => None,
        };

        update_pressed_keys(&mut self.pressed_keys, event);

        if let Some(mapping) = matched_mapping {
            self.mapped_keys.insert(event.key, mapping.new);

            let keys = mapping
                .prefixes
                .iter()
                .filter(|key| !self.already_released.contains(key))
                .collect::<Vec<_>>();

            // Ignore release events later for these keys, as they'll already
            // have been reported.
            for key in keys.iter() {
                self.already_released.insert(**key);
            }

            // Release prefixes so other clients don't see them as being pressed
            // at the same time as the mapped key.
            let mut final_keys = keys
                .into_iter()
                .map(|key| (*key, KeyState::RELEASED))
                .collect::<Vec<_>>();
            final_keys.push((mapping.new, event.value));
            final_keys
        } else if let Some((_old, new)) = self.mapped_keys.remove_entry(&event.key) {
            self.mapped_keys.remove_entry(&event.key);

            // First, release the mapped key.
            let mut final_keys = vec![(new, event.value)];

            // Then, re-press any prefixes that are still being held down.
            while let Some(key) = self.already_released.pop_back() {
                final_keys.push((key, KeyState::PRESSED));
            }

            final_keys
        } else if self.already_released.contains(&event.key) {
            self.already_released.remove(&event.key);
            vec![]
        } else {
            vec![(event.key, event.value)]
        }
    }

    fn all_pressed(&self, prefixes: &[Key]) -> bool {
        prefixes.iter().all(|key| self.pressed_keys.contains(key))
    }
}

fn update_pressed_keys(pressed_keys: &mut HashSet<Key>, event: &KeyEvent) {
    match event.value {
        KeyState::PRESSED | KeyState::AUTOREPEAT => {
            pressed_keys.insert(event.key);
        }
        KeyState::RELEASED => {
            pressed_keys.remove(&event.key);
        }
        _ => (),
    }
}

#[derive(Debug)]
pub enum Error {
    EmptyMappingError,
}

#[cfg(test)]
mod tests {
    use super::KeyMapper;
    use input_linux::{EventTime, Key, KeyEvent, KeyState};

    #[test]
    fn it_returns_same_key_if_no_mappings() {
        let mut mapper = KeyMapper::new();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![(Key::J, KeyState::PRESSED)]
        );
    }

    #[test]
    fn it_returns_same_key_if_no_mappings_match() {
        let mut mapper = KeyMapper::new();
        mapper.add_mapping(&[Key::CapsLock], &Key::J).unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![(Key::J, KeyState::PRESSED)]
        );
    }

    #[test]
    fn it_maps_a_single_key() {
        let mut mapper = KeyMapper::new();
        mapper
            .add_mapping(&[Key::CapsLock], &Key::LeftCtrl)
            .unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::PRESSED)),
            vec![(Key::LeftCtrl, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::LeftCtrl, KeyState::RELEASED)),
            vec![(Key::LeftCtrl, KeyState::RELEASED)]
        );
    }

    #[test]
    fn it_maps_a_sequence() {
        let mut mapper = KeyMapper::new();
        mapper
            .add_mapping(&[Key::CapsLock, Key::J], &Key::Down)
            .unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::PRESSED)),
            vec![(Key::CapsLock, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![
                (Key::CapsLock, KeyState::RELEASED),
                (Key::Down, KeyState::PRESSED)
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::RELEASED)),
            vec![
                (Key::Down, KeyState::RELEASED),
                (Key::CapsLock, KeyState::PRESSED)
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            vec![(Key::CapsLock, KeyState::RELEASED)]
        );
    }

    #[test]
    fn it_maps_a_sequence_with_more_than_one_prefix() {
        let mut mapper = KeyMapper::new();
        mapper
            .add_mapping(&[Key::CapsLock, Key::LeftShift, Key::J], &Key::Down)
            .unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::PRESSED)),
            vec![(Key::CapsLock, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::LeftShift, KeyState::PRESSED)),
            vec![(Key::LeftShift, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![
                (Key::CapsLock, KeyState::RELEASED),
                (Key::LeftShift, KeyState::RELEASED),
                (Key::Down, KeyState::PRESSED)
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::RELEASED)),
            vec![
                (Key::Down, KeyState::RELEASED),
                (Key::LeftShift, KeyState::PRESSED),
                (Key::CapsLock, KeyState::PRESSED),
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::LeftShift, KeyState::RELEASED)),
            vec![(Key::LeftShift, KeyState::RELEASED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            vec![(Key::CapsLock, KeyState::RELEASED)]
        );
    }

    #[test]
    fn it_maps_key_on_release_even_if_prefix_released() {
        let mut mapper = KeyMapper::new();
        mapper
            .add_mapping(&[Key::CapsLock, Key::J], &Key::Down)
            .unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::PRESSED)),
            vec![(Key::CapsLock, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![
                (Key::CapsLock, KeyState::RELEASED),
                (Key::Down, KeyState::PRESSED)
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            vec![]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::RELEASED)),
            vec![(Key::Down, KeyState::RELEASED)]
        );
    }

    #[test]
    fn it_handles_prefixes_pressed_after_mapped_key() {
        let mut mapper = KeyMapper::new();
        mapper
            .add_mapping(&[Key::CapsLock, Key::J], &Key::Down)
            .unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            vec![(Key::J, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::PRESSED)),
            vec![(Key::CapsLock, KeyState::PRESSED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::RELEASED)),
            vec![(Key::J, KeyState::RELEASED)]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            vec![(Key::CapsLock, KeyState::RELEASED)]
        );
    }

    fn mock_event(key: Key, value: KeyState) -> KeyEvent {
        KeyEvent::new(EventTime::new(0, 0), key, value)
    }
}
