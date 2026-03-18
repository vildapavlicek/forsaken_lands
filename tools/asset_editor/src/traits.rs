use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Result of loading assets from a directory.
/// Contains sorted lists of IDs and filenames, and a mapping between them.
#[derive(Debug, Default, Clone)]
pub struct LoadedAssets {
    pub ids: Vec<String>,
    pub filenames: Vec<String>,
    pub id_to_filename: HashMap<String, String>,
    /// Maps filename stem → relative subfolder (e.g. "1-1", "autopsies", or "" for root)
    pub filename_to_subfolder: HashMap<String, String>,
}

/// Trait to define how to load a specific type of asset.
pub trait AssetLoader {
    /// The subdirectory under the main assets directory (e.g., "research", "weapons")
    fn sub_path(&self) -> PathBuf;

    /// The file extension to look for (e.g., ".research.ron")
    fn extension(&self) -> &str;

    /// Extract the ID from the file stem and/or content.
    /// Returns None if the ID cannot be extracted (skips the file).
    fn extract_id(&self, stem: &str, content: &str) -> Option<String>;

    /// Optional filter to determine if a file should be processed based on its name.
    /// Default implementation accepts all files matching the extension.
    fn accept_filename(&self, _filename: &str) -> bool {
        true
    }

    /// Whether to recurse into subdirectories when scanning.
    /// Default is false for backwards compatibility.
    fn recursive(&self) -> bool {
        false
    }
}

/// Generic function to load assets using an AssetLoader implementation.
/// Supports recursive directory walking when `loader.recursive()` returns true.
pub fn load_assets<T: AssetLoader>(assets_dir: &Path, loader: &T) -> LoadedAssets {
    let mut result = LoadedAssets::default();
    let root_path = assets_dir.join(loader.sub_path());

    if loader.recursive() {
        load_assets_recursive(&root_path, &root_path, loader, &mut result);
    } else {
        load_assets_flat(&root_path, &root_path, loader, &mut result);
    }

    result.ids.sort();
    result.filenames.sort();
    result
}

/// Load assets from a single directory (non-recursive).
fn load_assets_flat<T: AssetLoader>(
    dir_path: &Path,
    root_path: &Path,
    loader: &T,
    result: &mut LoadedAssets,
) {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                process_asset_file(&path, root_path, loader, result);
            }
        }
    }
}

/// Recursively load assets from a directory and all subdirectories.
fn load_assets_recursive<T: AssetLoader>(
    dir_path: &Path,
    root_path: &Path,
    loader: &T,
    result: &mut LoadedAssets,
) {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                load_assets_recursive(&path, root_path, loader, result);
            } else if path.is_file() {
                process_asset_file(&path, root_path, loader, result);
            }
        }
    }
}

/// Process a single asset file: check extension, filter, extract ID, and record subfolder.
fn process_asset_file<T: AssetLoader>(
    path: &Path,
    root_path: &Path,
    loader: &T,
    result: &mut LoadedAssets,
) {
    let Some(filename) = path.file_name() else {
        return;
    };
    let filename_str = filename.to_string_lossy();

    // Check extension
    if !filename_str.ends_with(loader.extension()) {
        return;
    }

    // Check custom filter
    if !loader.accept_filename(&filename_str) {
        return;
    }

    // Get stem
    let Some(stem) = filename_str.strip_suffix(loader.extension()) else {
        return;
    };

    // Compute relative subfolder from root
    let subfolder = path
        .parent()
        .and_then(|parent| parent.strip_prefix(root_path).ok())
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();

    result.filenames.push(stem.to_string());
    result
        .filename_to_subfolder
        .insert(stem.to_string(), subfolder);

    // Read content and extract ID
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Some(id) = loader.extract_id(stem, &content) {
            result.ids.push(id.clone());
            result.id_to_filename.insert(id, stem.to_string());
        }
    }
}
