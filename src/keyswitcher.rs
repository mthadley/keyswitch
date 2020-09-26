use evdev_rs::{enums::EventType, Device, InputEvent, ReadFlag, UInputDevice};
use std::{convert::From, fmt::Debug, fs::File, io};

pub struct Keyswitcher {
    input_device: Device,
    output_device: UInputDevice,
}

impl Keyswitcher {
    pub fn new(fd: File) -> Result<Self, Error> {
        let input_device = Device::new_from_fd(fd).map_err(|e| Error::NoInputDeviceError(e))?;

        let output_spec = Device::new().ok_or(Error::NoOutputDeviceSpec)?;
        output_spec.set_name("Keyswitcher Virtual Input");
        output_spec.enable(&EventType::EV_KEY)?;

        let output_device = UInputDevice::create_from_device(&output_spec)?;

        println!("{:?}", output_device.devnode());
        println!("{:?}", output_device.syspath());

        Ok(Keyswitcher {
            input_device,
            output_device,
        })
    }

    pub fn run(&self) -> Result<(), Error> {
        loop {
            if let Ok((_read_status, event)) = self
                .input_device
                .next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING)
            {
                self.handle_event(event)?;
            }
        }
    }

    fn handle_event(&self, event: InputEvent) -> Result<(), Error> {
        println!("{:?}", event.event_code);

        self.output_device.write_event(&event)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    NoInputDeviceError(io::Error),
    NoOutputDeviceSpec,
    NoOutputDeviceError(io::Error),
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Error::IOError(io_error)
    }
}
