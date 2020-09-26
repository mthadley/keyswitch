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

        Ok(Keyswitcher {
            input_device,
            output_device,
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        //  Temporary limit on processed events, just in-case I lock up
        //  my keyboard while working on this thing.
        for _ in 0..50 {
            // Initialize empty input_event buffer
            let mut raw_events = [input_event {
                time: timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                },
                type_: 0,
                code: 0,
                value: 0,
            }; 24];

            let len = self.input_device.read(&mut raw_events)?;

            for raw_event in raw_events.iter().take(len) {
                let event = InputEvent::from_raw(raw_event)?.to_owned();
                self.forward_event(event)?;
            }
        }

        Ok(())
    }

    fn forward_event(&self, event: InputEvent) -> Result<(), Error> {
        if let Ok(Event::Key(key_event)) = Event::new(event) {
            // TODO: Add the actual remapping logic here.
            let events: [input_event; 2] = [
                InputEvent::from(KeyEvent::new(
                    get_timestamp()?,
                    key_event.key,
                    key_event.value,
                ))
                .as_raw()
                .to_owned(),
                InputEvent::from(SynchronizeEvent::report(get_timestamp()?))
                    .as_raw()
                    .to_owned(),
            ];

            self.output_device.write(&events)?;
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

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    InputEventRangeError,
    SystemTimeError,
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
