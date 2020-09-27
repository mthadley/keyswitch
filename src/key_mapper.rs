use input_linux::{Key, KeyEvent, KeyState};
use std::{collections::HashSet, vec::Vec};

pub struct KeyMapper {
    mappings: Vec<Mapping>,
    pressed_keys: HashSet<Key>,
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
        let mappings = self
            .mappings
            .iter()
            .find_map(|Mapping { prefixes, old, new }| {
                if *old == event.key && self.all_pressed(prefixes) {
                    // Immediately release prefixes so that other clients
                    // don't see the other keys as being held down at the
                    // same time.
                    // TODO: Maybe don't contstantly send RELEASED events for keys
                    // that previoulsy have already been released (at least as far
                    // as the other clients are concerned).
                    let mut mappings = prefixes
                        .iter()
                        .map(|key| (*key, KeyState::RELEASED))
                        .collect::<Vec<_>>();

                    mappings.push((*new, event.value));
                    Some(mappings)
                } else {
                    None
                }
            });

        match event.value {
            KeyState::PRESSED | KeyState::AUTOREPEAT => {
                self.pressed_keys.insert(event.key);
            }
            KeyState::RELEASED => {
                self.pressed_keys.remove(&event.key);
            }
            _ => (),
        }

        mappings.unwrap_or_else(|| vec![(event.key, event.value)])
    }

    fn all_pressed(&self, prefixes: &[Key]) -> bool {
        prefixes.iter().all(|key| self.pressed_keys.contains(key))
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
                // TODO: There shouldn't be another CapsLock RELEASED here.
                (Key::CapsLock, KeyState::RELEASED),
                (Key::Down, KeyState::RELEASED)
            ]
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            // TODO: This should be empty
            vec![(Key::CapsLock, KeyState::RELEASED)]
        );
    }

    fn mock_event(key: Key, value: KeyState) -> KeyEvent {
        KeyEvent::new(EventTime::new(0, 0), key, value)
    }
}
