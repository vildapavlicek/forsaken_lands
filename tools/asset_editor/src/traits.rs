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
}

/// Generic function to load assets using an AssetLoader implementation.
pub fn load_assets<T: AssetLoader>(assets_dir: &Path, loader: &T) -> LoadedAssets {
    let mut result = LoadedAssets::default();
    let dir_path = assets_dir.join(loader.sub_path());

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();

                // Check extension
                if !filename_str.ends_with(loader.extension()) {
                    continue;
                }

                // Check custom filter
                if !loader.accept_filename(&filename_str) {
                    continue;
                }

                // Get stem
                if let Some(stem) = filename_str.strip_suffix(loader.extension()) {
                    result.filenames.push(stem.to_string());

                    // Read content and extract ID
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(id) = loader.extract_id(stem, &content) {
                            result.ids.push(id.clone());
                            result.id_to_filename.insert(id, stem.to_string());
                        }
                    }
                }
            }
        }
    }

    result.ids.sort();
    result.filenames.sort();
    result
}
