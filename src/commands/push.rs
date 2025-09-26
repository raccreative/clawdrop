use std::{
    path::Path,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use aws_sdk_s3::config::{Credentials, Region};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use crate::{
    constants::{COMPLETE_PUSH_URL, REQUEST_UPLOAD_URL, VERIFY_UPLOAD_URL},
    errors::push::PushError,
    file_index::{FileEntry, FileIndex, compare_fileindex, generate_fileindex},
    network::Game,
    ui::CliUi,
    utils::{get_api_key, get_target_game, set_target_game},
};

#[derive(Clone)]
pub struct PushArgs {
    pub id: Option<u64>,
    pub os: Option<String>,
    pub exe: Option<String>,
    pub version: Option<String>,
    pub path: String,
    pub ignore: Vec<String>,
    pub no_bump: bool,
    pub shorthand: Option<String>,
    pub force: bool,
}

pub struct ShorthandParams {
    pub id: Option<u64>,
    pub os: String,
    pub exe: String,
    pub version: Option<String>,
}

pub struct PushParams {
    pub id: u64,
    pub os: String,
    pub exe: String,
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub expiration: String,
    pub session_token: String,
    pub bucket: String,
    pub prefix: String,
    pub region: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestUploadResponse {
    pub upload_credentials: TemporaryCredentials,
    pub delete_credentials: TemporaryCredentials,
    pub extra_uploads: ExtraUploads,
    pub extra_downloads: ExtraDownloads,
    pub original_zip_name: Option<String>,
    pub upload_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestUploadBody {
    version: String,
    fileindex: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyUploadBody {
    version: String,
    upload_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletePushBody {
    os: String,
    new_version: String,
    file_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraUploads {
    pub manifest: String,
    pub fileindex: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraDownloads {
    pub fileindex: Option<String>,
}

#[derive(Serialize)]
struct Manifest {
    path: String,
    version: String,
}

pub async fn run(args: PushArgs) -> Result<(), PushError> {
    let ui = CliUi::new();

    // First of all, we check if target game exist and update its data doing a new set
    let mut spinner = ui.start_spinner("Checking target game");

    let target_game = match get_target_game()? {
        Some(target) => {
            ui.set_status("Updating target data");
            let game = tokio::task::spawn_blocking(move || set_target_game(target.id.to_string()))
                .await??;
            Some(game)
        }
        None => None,
    };

    // 1. We parse and resolve the params
    let params: PushParams = resolve_push_params(args.clone(), target_game)?;

    let api_key: String = get_api_key()?;
    let http_client = build_client()?;

    // 2. We generate fileindex.json local with --ignore if any
    ui.set_status("Generating local fileindex.json");

    let fileindex_local = generate_fileindex(&args.path, &args.ignore)?;
    let fileindex_json_string = serde_json::to_string(&fileindex_local)?;

    // 3. We request the temporal credentials and use them in S3
    ui.set_status("Requesting credentials");

    let response = request_upload(
        &http_client,
        &api_key,
        params.id.clone(),
        params.os.clone(),
        fileindex_json_string,
    )
    .await?;

    let RequestUploadResponse {
        delete_credentials,
        upload_credentials,
        extra_uploads,
        extra_downloads,
        original_zip_name,
        upload_id,
    } = response;

    let expiration_time: Option<SystemTime> =
        match DateTime::parse_from_rfc3339(&upload_credentials.expiration) {
            Ok(dt) => {
                let timestamp = dt.timestamp();
                Some(UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64))
            }
            Err(_) => None,
        };

    // We have separate clients for upload and deleting files for better security
    let upload_config = aws_sdk_s3::Config::builder()
        .region(Region::new(upload_credentials.region.clone()))
        .credentials_provider(Credentials::new(
            upload_credentials.access_key_id,
            upload_credentials.secret_access_key,
            Some(upload_credentials.session_token),
            expiration_time,
            "ApiTemporaryCredentials",
        ))
        .behavior_version_latest()
        .build();

    let delete_config = aws_sdk_s3::Config::builder()
        .region(Region::new(delete_credentials.region.clone()))
        .credentials_provider(Credentials::new(
            delete_credentials.access_key_id,
            delete_credentials.secret_access_key,
            Some(delete_credentials.session_token),
            expiration_time,
            "ApiTemporaryCredentials",
        ))
        .behavior_version_latest()
        .build();

    let s3_client_upload = aws_sdk_s3::Client::from_conf(upload_config);
    let s3_client_delete = aws_sdk_s3::Client::from_conf(delete_config);

    // 4. We download remote fileindex.json for local comparison (if no fileindex in remote we create an empty one)
    ui.set_status("Retrieving remote fileindex.json");

    let fileindex_remote: FileIndex = if let Some(url) = &extra_downloads.fileindex {
        let res_fileindex = http_client.get(url).send().await?;
        match res_fileindex.status() {
            reqwest::StatusCode::NOT_FOUND => FileIndex::default(),
            status if status.is_success() => res_fileindex.json().await?,
            status => {
                let text = res_fileindex
                    .text()
                    .await
                    .unwrap_or_else(|_| "<no body>".into());
                return Err(PushError::ServerError {
                    code: status.as_u16(),
                    message: text,
                });
            }
        }
    } else {
        FileIndex::default()
    };

    // 5. Compare local and remote fileindex to see what changed (if --force upload all files)
    ui.set_status("Comparing local and remote fileindex for changes");

    let changes = compare_fileindex(&fileindex_local, &fileindex_remote, &original_zip_name);
    let files_to_upload = changes.to_upload(args.force, &fileindex_local);

    spinner.stop();
    if files_to_upload.is_empty() && changes.deleted_files.is_empty() {
        println!(
            "No changes to upload or delete, you can use --force to ignore this and upload everything"
        );
        return Ok(());
    }

    println!(
        "\n New files: {}, Modified files: {}, Obsolete files: {}",
        changes.new_files.len(),
        changes.modified_files.len(),
        changes.deleted_files.len()
    );

    // 6. Upload new/modified files with progress bar (everything if --force except --ignore [step 4 & 5])
    let total = files_to_upload.len();
    let base_path = std::path::PathBuf::from(&args.path);
    upload_files_if_any(
        &ui,
        &s3_client_upload,
        total,
        &files_to_upload,
        upload_credentials.prefix,
        &upload_credentials.bucket,
        &base_path,
    )
    .await?;

    spinner = ui.start_spinner("Verifying Uploaded files");
    verify_upload(
        &http_client,
        &api_key,
        params.os.clone(),
        upload_id,
        params.id,
    )
    .await?;

    spinner.stop();

    // 7. Delete obsolete remote files (except manifest.json, *.zip)
    let total_to_delete = changes.deleted_files.len();
    delete_files_if_any(
        &ui,
        &s3_client_delete,
        total_to_delete,
        &changes.deleted_files,
        delete_credentials.prefix,
        &delete_credentials.bucket,
    )
    .await?;

    // 8. Upload manifest.json and fileindex.json with presigned urls
    spinner = ui.start_spinner("Uploading new manifest.json and fileindex.json");

    let manifest = Manifest {
        path: params.exe.clone(),
        version: params.version.clone(),
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest)?;
    let fileindex_json = serde_json::to_vec_pretty(&fileindex_local)?;

    upload_extra_files(
        &http_client,
        extra_uploads.manifest,
        manifest_json,
        extra_uploads.fileindex,
        fileindex_json,
    )
    .await?;

    // 9. We update the new version in database and close the upload
    ui.set_status("Finalizing");

    let complete_push_body = CompletePushBody {
        os: params.os.clone(),
        new_version: params.version.clone(),
        file_name: original_zip_name
            .clone()
            .unwrap_or_else(|| format!("{}-{}.zip", params.id, params.os)),
    };

    complete_push(&http_client, api_key, params.id, complete_push_body).await?;

    spinner.stop();

    Ok(())
}

// Build client for async (not the same as crate::network::build_client)
fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().use_rustls_tls().build()
}

async fn request_upload(
    client: &reqwest::Client,
    api_key: &String,
    id: u64,
    os: String,
    fileindex: String,
) -> Result<RequestUploadResponse, PushError> {
    let body = RequestUploadBody {
        version: os,
        fileindex: fileindex,
    };

    let res = client
        .post(REQUEST_UPLOAD_URL.replace("{id}", &id.to_string()))
        .header("x-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    let res_to_parse = match res.status() {
        reqwest::StatusCode::FORBIDDEN => return Err(PushError::UnauthorizedToUpload),
        reqwest::StatusCode::NOT_FOUND => return Err(PushError::GameNotFound),
        reqwest::StatusCode::BAD_REQUEST => return Err(PushError::FileSizeLimitReach),
        status if !status.is_success() => {
            let text = res.text().await.unwrap_or_else(|_| "<no body>".into());
            return Err(PushError::ServerError {
                code: status.as_u16(),
                message: text,
            });
        }
        _ => res,
    };

    let response: RequestUploadResponse = res_to_parse.json().await?;
    Ok(response)
}

async fn upload_extra_files(
    client: &reqwest::Client,
    manifest_url: String,
    manifest_json: Vec<u8>,
    fileindex_url: String,
    fileindex_json: Vec<u8>,
) -> Result<(), PushError> {
    client
        .put(manifest_url)
        .header("content-type", "application/json")
        .body(manifest_json)
        .send()
        .await
        .map_err(|e| PushError::S3Error {
            message: format!("Failed uploading manifest.json: {:?}", e),
        })?;

    client
        .put(fileindex_url)
        .header("content-type", "application/json")
        .body(fileindex_json)
        .send()
        .await
        .map_err(|e| PushError::S3Error {
            message: format!("Failed uploading fileindex.json: {:?}", e),
        })?;

    Ok(())
}

async fn upload_files_if_any(
    ui: &CliUi,
    s3_client: &aws_sdk_s3::Client,
    total: usize,
    files_to_upload: &Vec<FileEntry>,
    prefix: String,
    bucket: &String,
    base_path: &Path,
) -> Result<(), PushError> {
    if total == 0 {
        return Ok(());
    }

    let mut prepared_files = Vec::with_capacity(files_to_upload.len());
    let mut total_bytes: u64 = 0;
    for entry in files_to_upload {
        let local_path = base_path.join(&entry.path);
        let size = std::fs::metadata(&local_path)?.len();
        total_bytes += size;
        prepared_files.push((entry.path.clone(), local_path, size));
    }

    ui.show_progress_bytes(0, total_bytes, "Uploading files", None);

    let (tx, mut rx) = tokio::sync::mpsc::channel(prepared_files.len());
    let concurrency = 8;
    let sem = Arc::new(Semaphore::new(concurrency));

    // Spawn concurrent tasks
    for (file_path, local_path, size) in prepared_files {
        let s3_client = s3_client.clone();
        let bucket = bucket.clone();
        let key = format!("{}{}", prefix, file_path);
        let tx = tx.clone();
        let sem = sem.clone();

        tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();

            let body = aws_sdk_s3::primitives::ByteStream::from_path(&local_path)
                .await
                .map_err(|e| PushError::S3Error {
                    message: format!("Failed to read {}: {:?}", file_path, e),
                })?;

            s3_client
                .put_object()
                .bucket(&bucket)
                .key(&key)
                .checksum_algorithm(aws_sdk_s3::types::ChecksumAlgorithm::Sha256)
                .body(body)
                .send()
                .await
                .map_err(|e| PushError::S3Error {
                    message: format!("Failed uploading {}: {:?}", file_path, e),
                })?;

            // Send progress to main thread
            tx.send(size).await.unwrap();

            Ok::<(), PushError>(())
        });
    }

    drop(tx);

    // Update UI from main thread while results arrive
    let mut done_bytes: u64 = 0;
    let start = Instant::now();

    while let Some(uploaded) = rx.recv().await {
        done_bytes += uploaded;
        let elapsed = start.elapsed().as_secs_f64();
        // Speed could be inaccurate... But will be solved when delta-patching is implemented
        let speed = (elapsed > 0.0).then(|| (done_bytes as f64 / elapsed) as u64);

        ui.show_progress_bytes(done_bytes, total_bytes, "Uploading files", speed);
    }

    ui.finish_progress();
    Ok(())
}

async fn delete_files_if_any(
    ui: &CliUi,
    s3_client: &aws_sdk_s3::Client,
    total: usize,
    files_to_delete: &Vec<FileEntry>,
    prefix: String,
    bucket: &String,
) -> Result<(), PushError> {
    if total <= 0 {
        return Ok(());
    }

    ui.show_progress_count(0, total, "Deleting remote obsolete files");
    let mut deleted_done = 0;

    for entry in files_to_delete {
        let key = format!("{}{}", prefix, entry.path);

        s3_client
            .delete_object()
            .bucket(bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| PushError::S3Error {
                message: format!("Failed deleting {}: {:?}", entry.path, e),
            })?;

        deleted_done += 1;
        ui.show_progress_count(deleted_done, total, "Deleting remote obsolete files");
    }

    ui.finish_progress();
    Ok(())
}

async fn verify_upload(
    client: &reqwest::Client,
    api_key: &String,
    os: String,
    upload_id: String,
    id: u64,
) -> Result<(), PushError> {
    let verify_body = VerifyUploadBody {
        version: os,
        upload_id: upload_id,
    };

    let verify_res = client
        .put(VERIFY_UPLOAD_URL.replace("{id}", &id.to_string()))
        .header("x-api-key", api_key)
        .json(&verify_body)
        .send()
        .await?;

    match verify_res.status() {
        reqwest::StatusCode::BAD_REQUEST => return Err(PushError::FileindexMismatch),
        status if !status.is_success() => {
            let text = verify_res
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".into());
            return Err(PushError::ServerError {
                code: status.as_u16(),
                message: text,
            });
        }
        _ => (),
    };

    Ok(())
}

async fn complete_push(
    client: &reqwest::Client,
    api_key: String,
    id: u64,
    complete_push_body: CompletePushBody,
) -> Result<(), PushError> {
    let complete_push_res = client
        .put(COMPLETE_PUSH_URL.replace("{id}", &id.to_string()))
        .header("x-api-key", api_key)
        .json(&complete_push_body)
        .send()
        .await?;

    match complete_push_res.status() {
        reqwest::StatusCode::FORBIDDEN => return Err(PushError::UnauthorizedToUpload),
        status if !status.is_success() => {
            let text = complete_push_res
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".into());
            return Err(PushError::ServerError {
                code: status.as_u16(),
                message: text,
            });
        }
        _ => (),
    };

    Ok(())
}

// Checks if given executable file exists and returns a fixed path
fn find_executable(base_path: &Path, exe_name: &str) -> Option<String> {
    // Root first
    let candidate = base_path.join(exe_name);
    if candidate.is_file() {
        return Some(exe_name.to_string());
    }

    // Subdirectories
    let entries = std::fs::read_dir(base_path).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let candidate = path.join(exe_name);
        if !candidate.is_file() {
            continue;
        }

        return candidate
            .strip_prefix(base_path)
            .ok()
            .map(|rel| rel.to_string_lossy().to_string());
    }

    None
}

pub fn resolve_push_params(
    args: PushArgs,
    target_game: Option<Game>,
) -> Result<PushParams, PushError> {
    // We get shorthand params and target game if exists
    let short_hand_params: Option<ShorthandParams> =
        args.shorthand.map(|s| parse_shorthand(&s)).transpose()?;

    // Then we start setting the parameters from availability and flags
    let id = short_hand_params
        .as_ref()
        .and_then(|sh| sh.id)
        .or(args.id)
        .or(target_game.as_ref().map(|g| g.id))
        .ok_or(PushError::MissingId)?;

    let os = short_hand_params
        .as_ref()
        .map(|sh| sh.os.clone())
        .or(args.os)
        .ok_or(PushError::MissingOS)?;

    if !["windows", "linux", "mac", "html"].contains(&os.as_str()) {
        return Err(PushError::InvalidOS);
    }

    let exe = short_hand_params
        .as_ref()
        .map(|sh| sh.exe.as_str())
        .or(args.exe.as_deref())
        .ok_or(PushError::MissingExecutableName)?;

    let exe_path =
        find_executable(Path::new(&args.path), exe).ok_or(PushError::MissingExecutableFile)?;

    let version = short_hand_params
        .and_then(|sh| sh.version.clone())
        .or(args.version)
        .or_else(|| {
            target_game
                .as_ref()
                .and_then(|game| target_version_for_os(game, &os, args.no_bump))
        })
        .unwrap_or_else(|| "0.0.1".to_string());

    Ok(PushParams {
        id,
        os,
        exe: exe_path,
        version,
    })
}

fn target_version_for_os(target_game: &Game, os: &str, no_bump: bool) -> Option<String> {
    let base_version = match os {
        "windows" => target_game.windows_version.as_ref(),
        "linux" => target_game.linux_version.as_ref(),
        "mac" => target_game.mac_version.as_ref(),
        "html" => target_game.html_version.as_ref(),
        _ => return None,
    };

    base_version.map(|v| if no_bump { v.clone() } else { bump_version(v) })
}

// This function does not cover al cases
fn bump_version(version: &str) -> String {
    if let Some(pos) = version.rfind(|c: char| c.is_ascii_digit() == false) {
        let (left, right) = version.split_at(pos + 1);
        if let Ok(num) = right.parse::<u64>() {
            let width = right.len();
            return format!("{}{:0width$}", left, num + 1, width = width);
        }
    }

    version.to_string() // We skip bump if not compatible
}

// <id>:<os>/<exe>:<version>
pub fn parse_shorthand(input: &str) -> Result<ShorthandParams, PushError> {
    let (left, right) = input
        .split_once('/')
        .ok_or(PushError::InvalidShorthandFormat)?;

    // Left: (id:os)
    let (id, os_str) = match left.split_once(':') {
        Some((id_str, os)) => {
            let id = id_str
                .parse::<u64>()
                .map_err(|_| PushError::InvalidShorthandId)?;
            (Some(id), os)
        }
        None => (None, left),
    };

    let os = os_str.to_string();
    if os.is_empty() {
        return Err(PushError::InvalidShorthandFormat);
    }

    // Right: (exe:version)
    let (exe_str, ver_opt) = match right.split_once(':') {
        Some((exe, ver)) => (exe, Some(ver)),
        None => (right, None),
    };

    let exe = exe_str.to_string();
    if exe.is_empty() {
        return Err(PushError::InvalidShorthandFormat);
    }

    let version = ver_opt.map(|s| s.to_string());

    Ok(ShorthandParams {
        id,
        os,
        exe,
        version,
    })
}
