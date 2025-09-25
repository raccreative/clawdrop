use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Read},
    path::Path,
};

use globset::{Glob, GlobSet, GlobSetBuilder};
use mime_guess::MimeGuess;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub hash: String,
    pub content_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileIndex {
    pub files: Vec<FileEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileChanges {
    pub new_files: Vec<FileEntry>,
    pub modified_files: Vec<FileEntry>,
    pub deleted_files: Vec<FileEntry>,
}

impl Default for FileIndex {
    fn default() -> Self {
        FileIndex { files: Vec::new() }
    }
}

impl FileChanges {
    pub fn to_upload(&self, force: bool, local: &FileIndex) -> Vec<FileEntry> {
        if force {
            return local.files.clone();
        }
        let mut v = Vec::with_capacity(self.new_files.len() + self.modified_files.len());
        v.extend(self.new_files.clone());
        v.extend(self.modified_files.clone());
        v
    }
}

pub fn generate_fileindex<P: AsRef<Path>>(
    dir: P,
    ignore_patterns: &[String],
) -> Result<FileIndex, std::io::Error> {
    let base_path = dir.as_ref();

    let mut globset_builder = GlobSetBuilder::new();
    for pattern in ignore_patterns {
        globset_builder.add(Glob::new(pattern).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Invalid glob: {}", e))
        })?);
    }

    let globset = globset_builder.build().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("GlobSet build error: {}", e),
        )
    })?;

    let mut paths = Vec::new();
    collect_paths(base_path, base_path, &mut paths, &globset)?;

    let entries: Vec<FileEntry> = paths
        .into_par_iter()
        .filter_map(|(path, relative_path, metadata)| {
            let size = metadata.len();
            let file = fs::File::open(&path).ok()?;
            let mut reader = BufReader::new(file);
            let mut hasher = Sha256::new();
            let mut buffer = vec![0u8; 262144];

            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }

            let hash = format!("{:x}", hasher.finalize());
            let mime_type = MimeGuess::from_path(&path).first_or_octet_stream();
            let content_type = mime_type.essence_str().to_string();

            Some(FileEntry {
                path: relative_path,
                size,
                hash,
                content_type,
            })
        })
        .collect();

    let mut sorted = entries;
    sorted.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(FileIndex { files: sorted })
}

fn collect_paths(
    dir: &Path,
    base: &Path,
    paths: &mut Vec<(std::path::PathBuf, String, std::fs::Metadata)>,
    ignore_set: &GlobSet,
) -> Result<(), std::io::Error> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current_dir) = stack.pop() {
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            let relative_path = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid path encoding")
                })?
                .replace('\\', "/");

            if ignore_set.is_match(&relative_path) {
                continue;
            }

            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                paths.push((path, relative_path, metadata));
            }
        }
    }
    Ok(())
}

pub fn compare_fileindex(
    local: &FileIndex,
    remote: &FileIndex,
    original_zip_name: &Option<String>,
) -> FileChanges {
    let mut new_files = Vec::new();
    let mut modified_files = Vec::new();
    let mut deleted_files = Vec::new();

    let remote_map: HashMap<_, _> = remote.files.iter().map(|f| (&f.path, f)).collect();
    let local_map: HashMap<_, _> = local.files.iter().map(|f| (&f.path, f)).collect();

    // New or modified files
    for local_file in &local.files {
        match remote_map.get(&local_file.path) {
            Some(remote_file) => {
                if remote_file.hash != local_file.hash {
                    modified_files.push((*local_file).clone());
                }
            }
            None => new_files.push((*local_file).clone()),
        }
    }

    // Deleted files (except manifest.json and original .zip)
    for remote_file in &remote.files {
        let is_manifest = remote_file.path.ends_with("manifest.json");
        let is_original_zip = original_zip_name
            .as_ref()
            .map_or(false, |name| remote_file.path.ends_with(name));

        if !local_map.contains_key(&remote_file.path) && !is_manifest && !is_original_zip {
            deleted_files.push(remote_file.clone());
        }
    }

    FileChanges {
        new_files,
        modified_files,
        deleted_files,
    }
}
