use keyswitch::key_switcher::{self, KeySwitcher};
use std::env;

fn main() -> Result<(), key_switcher::Error> {
    let device_name = env::args().nth(1).expect("Expected device name.");
    KeySwitcher::new(device_name)?.run()?;

    Ok(())
}
