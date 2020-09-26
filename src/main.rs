use keyswitch::keyswitcher::{self, Keyswitcher};
use std::env;

fn main() -> Result<(), keyswitcher::Error> {
    let device_name = env::args().nth(1).expect("Expected device name.");
    Keyswitcher::new(device_name)?.run()?;

    Ok(())
}
