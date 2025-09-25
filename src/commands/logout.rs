use std::{env, fs, io};

use crate::utils::get_config_path;

pub fn run() -> Result<(), io::Error> {
    unsafe { env::remove_var("CLAWDROP_API_KEY") };

    let key_path = get_config_path().join(".api_key");

    match fs::remove_file(&key_path) {
        Ok(_) => println!("Logged out. API key removed from: {}", key_path.display()),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => println!("No API key found. Nothing to remove."),
        Err(e) => return Err(e),
    }

    Ok(())
}