use keyswitch::keyswitcher::{self, Keyswitcher};
use std::{env, fs::File};

fn main() -> Result<(), keyswitcher::Error> {
    let device_name = env::args().nth(1).expect("Expected device name.");
    let fd = File::open(&device_name)?;

    Keyswitcher::new(fd)?.run()?;

    Ok(())
}
