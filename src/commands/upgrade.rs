use self_update::cargo_crate_version;

use crate::green;
use std::io::{self, Write};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("raccreative")
        .repo_name("clawdrop")
        .bin_name("clawdrop")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?;

    let latest_version = updater.get_latest_release()?.version;
    let current_version = cargo_crate_version!();

    if latest_version == current_version {
        println!(
            "You are already on the latest version ({})",
            current_version
        );

        return Ok(());
    }

    println!(
        "A new version {} is available! You are on {}.",
        latest_version, current_version
    );

    print!("Do you want to update now? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    let input_valid = input == "y" || input == "yes";

    if !input_valid {
        println!(
            "Update cancelled, continuing with version {}",
            current_version
        );

        return Ok(());
    }

    let update_status = updater.update()?;
    if update_status.updated() {
        println!(
            "{} Clawdrop updated from {} to {}",
            green!("✓"),
            current_version,
            update_status.version()
        );
    }

    Ok(())
}
