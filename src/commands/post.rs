use std::{fs, path::Path};

use mime_guess::MimeGuess;
use reqwest::blocking::multipart;

use crate::{
    constants::GAME_POST_URL,
    errors::post::PostError,
    green,
    network::build_client,
    utils::{get_api_key, get_target_game},
};

pub fn run(
    id: Option<u64>,
    title: String,
    body: String,
    cover: Option<String>,
    slug: Option<String>,
) -> Result<(), PostError> {
    let game_id = match id {
        Some(id) => id,
        None => match get_target_game() {
            Ok(Some(game)) => game.id,
            Ok(None) => return Err(PostError::NoIdSpecified),
            Err(e) => return Err(PostError::Common(e.into())),
        },
    };

    let final_body = resolve_body(body)?;

    let api_key = get_api_key()?;
    let client = build_client()?;

    let mut form = multipart::Form::new()
        .text("gameId", game_id.to_string())
        .text("title", title)
        .text("body", final_body);

    if let Some(s) = slug {
        form = form.text("slug", s);
    }

    if let Some(cover_path) = &cover {
        let bytes = fs::read(cover_path)?;
        let mime_type = MimeGuess::from_path(cover_path).first_or_octet_stream();

        let part = multipart::Part::bytes(bytes)
            .file_name("cover.jpg")
            .mime_str(mime_type.essence_str())?;

        form = form.part("cover", part);
    }

    let response = match client
        .post(GAME_POST_URL)
        .header("x-api-key", api_key)
        .multipart(form)
        .send()
    {
        Ok(r) => match r.status() {
            reqwest::StatusCode::FORBIDDEN => return Err(PostError::UnauthorizedToPost),
            status if !status.is_success() => {
                let text = r.text().unwrap_or_else(|_| "<no body>".into());
                return Err(PostError::ServerError {
                    code: status.as_u16(),
                    message: text,
                });
            }
            _ => r,
        },
        Err(e) => return Err(e.into()),
    };

    println!(
        "Post created. Code: {} {}",
        response.status(),
        green!("✓ OK")
    );

    Ok(())
}

// Function to check if body is text or path to text file
fn resolve_body(body: String) -> Result<String, PostError> {
    let path = Path::new(&body);

    if path.is_file() {
        return fs::read_to_string(path).map_err(PostError::from);
    }

    Ok(body)
}
