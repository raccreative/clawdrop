use std::{fs, io};

use crate::{green, utils::get_config_path};

pub fn run() -> Result<(), io::Error> {
    let path = get_config_path().join("target.json");

    match fs::remove_file(&path) {
        Ok(_) => println!("Target removed correctly {}", green!("✓")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => println!("No target configured."),
        Err(e) => return Err(e),
    }

    Ok(())
}
 