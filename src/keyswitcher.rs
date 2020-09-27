use crate::key_mapper::{self, KeyMapper};
use input_linux::{
    EvdevHandle, Event, EventKind, EventTime, InputEvent, InputId, Key, KeyEvent, SynchronizeEvent,
    UInputHandle,
};
use libc::{input_event, timeval};
use std::{
    convert::{AsRef, From, TryInto},
    fmt::Debug,
    fs::{self, File},
    io,
    path::Path,
    time::{SystemTime, SystemTimeError},
};

/// Taken from <linux/input.h>
const BUS_USB: u16 = 0x03;

/// Virtual Device info
const VENDOR: u16 = 0x3232;
const VERSION: u16 = 0x1234;
const PRODUCT: u16 = 0x5678;

pub struct Keyswitcher {
    input_device: EvdevHandle<File>,
    output_device: UInputHandle<File>,
    key_mapper: KeyMapper,
}

impl Keyswitcher {
    pub fn new(input_path: impl AsRef<Path>) -> Result<Self, Error> {
        let input_file = File::open(input_path)?;
        let input_device = EvdevHandle::new(input_file);
        input_device.grab(true)?;

        let uinput = fs::OpenOptions::new().write(true).open("/dev/uinput")?;
        let output_device = UInputHandle::new(uinput);

        output_device.set_evbit(EventKind::Key)?;
        output_device.set_evbit(EventKind::Synchronize)?;

        for key in Key::iter() {
            output_device.set_keybit(key)?;
        }

        output_device.create(
            &InputId {
                bustype: BUS_USB,
                vendor: VENDOR,
                product: PRODUCT,
                version: VERSION,
            },
            "Keyswitcher Virtual Input".as_bytes(),
            0,
            &[],
        )?;

        Ok(Self {
            input_device,
            output_device,
            key_mapper: test_key_mapper()?,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        //  Temporary limit on processed events, just in-case I lock up
        //  my keyboard while working on this thing.
        for _ in 0..200 {
            // Initialize empty input_event buffer
            let mut raw_events = [EMPTY_INPUT_EVENT; 24];

            let len = self.input_device.read(&mut raw_events)?;

            for raw_event in raw_events.iter().take(len) {
                let event = InputEvent::from_raw(raw_event)?.to_owned();
                self.handle_event(event)?;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: InputEvent) -> Result<(), Error> {
        if let Ok(Event::Key(key_event)) = Event::new(event) {
            for (mapped_key, state) in self.key_mapper.handle_key_event(&key_event) {
                let events: [input_event; 2] = [
                    InputEvent::from(KeyEvent::new(get_timestamp()?, mapped_key, state))
                        .as_raw()
                        .to_owned(),
                    InputEvent::from(SynchronizeEvent::report(get_timestamp()?))
                        .as_raw()
                        .to_owned(),
                ];

                self.output_device.write(&events)?;
            }
        }

        Ok(())
    }
}

fn get_timestamp() -> Result<EventTime, Error> {
    let time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

    Ok(EventTime::new(
        (time.as_secs() as u64)
            .try_into()
            .map_err(|_| Error::SystemTimeError)?,
        (time.subsec_micros() as u64)
            .try_into()
            .map_err(|_| Error::SystemTimeError)?,
    ))
}

/// TODO: Replace this with a real configuration system
fn test_key_mapper() -> Result<KeyMapper, key_mapper::Error> {
    let mut mapper = KeyMapper::new();

    mapper.add_mapping(&[Key::CapsLock, Key::H], &Key::Left)?;
    mapper.add_mapping(&[Key::CapsLock, Key::J], &Key::Down)?;
    mapper.add_mapping(&[Key::CapsLock, Key::K], &Key::Up)?;
    mapper.add_mapping(&[Key::CapsLock, Key::L], &Key::Right)?;

    Ok(mapper)
}

const EMPTY_INPUT_EVENT: input_event = input_event {
    time: timeval {
        tv_sec: 0,
        tv_usec: 0,
    },
    type_: 0,
    code: 0,
    value: 0,
};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InputEventRangeError,
    SystemTimeError,
    BadMappingError(key_mapper::Error),
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Error::IOError(io_error)
    }
}

impl From<input_linux::RangeError> for Error {
    fn from(_: input_linux::RangeError) -> Self {
        Error::InputEventRangeError
    }
}

impl From<SystemTimeError> for Error {
    fn from(_: SystemTimeError) -> Self {
        Error::SystemTimeError
    }
}

impl From<key_mapper::Error> for Error {
    fn from(error: key_mapper::Error) -> Self {
        Error::BadMappingError(error)
    }
}
