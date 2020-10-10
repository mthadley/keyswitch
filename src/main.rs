use clap::{App, Arg};
use keyswitch::{
    device::{self, Device},
    key_switcher::{self, KeySwitcher},
};
use std::{path::PathBuf, process};

fn main() {
    match match get_mode_from_args() {
        Some(Mode::ListDevices) => Device::print_available().map_err(Error::from),
        Some(Mode::ReadDevice(id)) => {
            let device_path = match id {
                DeviceId::ByPath(path) => Ok(PathBuf::from(path)),
                DeviceId::ByName(name) => {
                    Device::available()
                        .map_err(Error::from)
                        .and_then(|mut devices| {
                            devices
                                .find(|d| d.name == name)
                                .map(|d| d.dev_path)
                                .ok_or(Error::NoDeviceFoundError(name))
                        })
                }
            };

            device_path.and_then(|path| {
                KeySwitcher::open(path)
                    .and_then(|mut s| s.run())
                    .map_err(Error::from)
            })
        }
        None => process::exit(1),
    } {
        Err(Error::DeviceListingError(_)) => {
            println!("Ran into an error when attempting to list devices.");
            process::exit(1);
        }
        Err(Error::NoDeviceFoundError(name)) => {
            println!("Device with name not found: {}", name);
            process::exit(1);
        }
        Err(Error::KeySwitcherError(err)) => {
            let message = match err {
                key_switcher::Error::BadMappingError(_) => {
                    "Encountered a bad key mapping. Check your configuration."
                }
                _ => "Encountered an unexpected error when mapping a key.",
            };

            println!("{}", message);
            process::exit(1);
        }
        Ok(()) => (),
    }
}

enum Mode {
    ReadDevice(DeviceId),
    ListDevices,
}

enum DeviceId {
    ByName(String),
    ByPath(String),
}

fn get_mode_from_args() -> Option<Mode> {
    let args = App::new("Keyswitcher")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Maps keys at a low-level.")
        .arg(
            Arg::with_name("device")
                .short("d")
                .number_of_values(1)
                .long_help("The device to read events from.")
                .conflicts_with("device-name")
                .required_unless("list"),
        )
        .arg(
            Arg::with_name("device-name")
                .short("n")
                .number_of_values(1)
                .long_help("The name of the device to read events from.")
                .conflicts_with("device")
                .required_unless("list"),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long_help("List devices that are readable.")
                .required_unless_one(&["device", "device-name"]),
        )
        .get_matches();

    if let Some(path) = args.value_of("device") {
        Some(Mode::ReadDevice(DeviceId::ByPath(path.to_owned())))
    } else if let Some(name) = args.value_of("device-name") {
        Some(Mode::ReadDevice(DeviceId::ByName(name.to_owned())))
    } else if args.is_present("list") {
        Some(Mode::ListDevices)
    } else {
        None
    }
}

#[derive(Debug)]
enum Error {
    DeviceListingError(device::Error),
    KeySwitcherError(key_switcher::Error),
    NoDeviceFoundError(String),
}

impl From<device::Error> for Error {
    fn from(device_error: device::Error) -> Self {
        Error::DeviceListingError(device_error)
    }
}

impl From<key_switcher::Error> for Error {
    fn from(key_switcher_error: key_switcher::Error) -> Self {
        Error::KeySwitcherError(key_switcher_error)
    }
}
