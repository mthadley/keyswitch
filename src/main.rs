use std::env;

fn main() {
    let device_name = env::args().nth(1).expect("Expected device name.");

    println!("Collecting events from {}", device_name);
}
