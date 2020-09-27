use input_linux::{Key, KeyEvent};
use std::{collections::HashSet, vec::Vec};

pub struct KeyMapper {
    mappings: Vec<(Vec<Key>, (Key, Key))>,
    pressed_keys: HashSet<Key>,
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
                self.mappings.push((Vec::from(prefixes), (*old, *new)));
                Ok(())
            }
        }
    }

    pub fn handle_key_event(&mut self, event: &KeyEvent) -> Key {
        let mapping = self.mappings.iter().find_map(|(prefixes, (old, new))| {
            if *old == event.key && self.all_pressed(prefixes) {
                Some(new)
            } else {
                None
            }
        });

        if event.value.is_pressed() {
            self.pressed_keys.insert(event.key);
        } else {
            self.pressed_keys.remove(&event.key);
        }

        println!("Mapped? {:?}", mapping);

        *mapping.unwrap_or(&event.key)
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
            Key::J
        );
    }

    #[test]
    fn it_returns_same_key_if_no_mappings_match() {
        let mut mapper = KeyMapper::new();
        mapper.add_mapping(&[Key::CapsLock], &Key::J).unwrap();

        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            Key::J
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
            Key::LeftCtrl
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::LeftCtrl, KeyState::RELEASED)),
            Key::LeftCtrl
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
            Key::CapsLock
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::PRESSED)),
            Key::Down
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::J, KeyState::RELEASED)),
            Key::Down
        );
        assert_eq!(
            mapper.handle_key_event(&mock_event(Key::CapsLock, KeyState::RELEASED)),
            Key::CapsLock
        );
    }

    fn mock_event(key: Key, value: KeyState) -> KeyEvent {
        KeyEvent::new(EventTime::new(0, 0), key, value)
    }
}
