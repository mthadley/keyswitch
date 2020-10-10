use input_linux::EvdevHandle;
use std::{
    fs::{self, File},
    io,
    path::PathBuf,
    str,
    vec::Vec,
};

pub struct Device {
    handle: EvdevHandle<File>,
    dev_path: PathBuf,
}

impl Device {
    pub fn list() -> Result<Vec<Device>, io::Error> {
        let devices = fs::read_dir("/dev/input")?
            .filter_map(|res| res.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .into_string()
                    .map(|s| s.contains("event"))
                    .unwrap_or(false)
            })
            .filter_map(|entry| Self::open(entry.path()).ok())
            .collect();

        Ok(devices)
    }

    pub fn print_list() -> Result<(), Error> {
        println!("Available devices: \n");

        for Device { handle, dev_path } in Self::list()? {
            let name_bytes = handle.device_name()?;
            let name = str::from_utf8(&name_bytes)?;
            let path = dev_path.to_str().unwrap_or("");
            println!("{:19} {}", path, name);
        }

        Ok(())
    }

    pub fn open(dev_path: PathBuf) -> Result<Self, io::Error> {
        File::open(&dev_path).map(|file| Device {
            handle: EvdevHandle::new(file),
            dev_path: dev_path,
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
