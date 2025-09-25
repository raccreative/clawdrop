use crate::{errors::set::SetError, green, utils::set_target_game};

pub fn run(id: String) -> Result<(), SetError> {
    let target_game = set_target_game(id)?;

    println!(
        "Game '{}' set as default target {}",
        target_game.title,
        green!("✓")
    );

    Ok(())
}
