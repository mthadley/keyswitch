use clap::{App, Arg};
use keyswitch::{
    device::{self, Device},
    key_switcher::{self, KeySwitcher},
};
use std::process;

fn main() -> Result<(), Error> {
    match get_mode_from_args() {
        Some(Mode::ListDevices) => {
            Device::print_list()?;
        }
        Some(Mode::ReadDevice(path)) => {
            KeySwitcher::new(path)?.run()?;
        }
        None => {
            process::exit(1);
        }
    }

    Ok(())
}

enum Mode {
    ReadDevice(String),
    ListDevices,
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
                .required_unless("list"),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long_help("List devices that are readable.")
                .required_unless("device"),
        )
        .get_matches();

    if let Some(path) = args.value_of("device") {
        Some(Mode::ReadDevice(path.to_owned()))
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
