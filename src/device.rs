use input_linux::EvdevHandle;
use std::{
    fs::{self, File},
    io,
    iter::Iterator,
    path::PathBuf,
    str,
};

pub struct Device {
    pub dev_path: PathBuf,
    pub name: String,
}

impl Device {
    pub fn available() -> Result<impl Iterator<Item = Device>, Error> {
        Ok(fs::read_dir("/dev/input")?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .into_string()
                    .map(|s| s.contains("event"))
                    .unwrap_or(false)
            })
            .filter_map(|entry| Self::open(entry.path()).ok()))
    }

    pub fn print_available() -> Result<(), Error> {
        println!("Available devices: \n");

        let devices = Self::available()?.collect::<Vec<_>>();
        let dev_path_width = devices
            .iter()
            .map(|d| d.dev_path.to_str().unwrap_or("").len())
            .max()
            .unwrap_or(0);

        for Device { dev_path, name, .. } in devices {
            let path = dev_path.to_str().unwrap_or("");
            println!(
                "{path:dev_path_width$}  {name}",
                path = path,
                dev_path_width = dev_path_width,
                name = name
            );
        }

        Ok(())
    }

    pub fn open(dev_path: PathBuf) -> Result<Self, Error> {
        let file = File::open(&dev_path)?;
        let handle = EvdevHandle::new(file);

        let name_bytes = handle.device_name()?;
        let name = str::from_utf8(&name_bytes)?.trim_end_matches('\u{0}');

        Ok(Device {
            dev_path: dev_path,
            name: String::from(name),
        })
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    Utf8Error(str::Utf8Error),
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Self {
        Error::IOError(io_error)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(utf8_error: str::Utf8Error) -> Self {
        Error::Utf8Error(utf8_error)
    }
}
