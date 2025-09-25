use std::env;
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;
    println!("{}", exe_path.display());
    Ok(())
}
