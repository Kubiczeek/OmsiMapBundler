use std::path::Path;
use std::fs;
use std::fs::File;
use mtzip::ZipArchive;
use walkdir::WalkDir;

// Helper function to copy a file and create parent directories
pub fn copy_file_with_folders(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

// Helper function to create ZIP from directory with configurable compression
pub fn create_zip(
    src_dir: &Path,
    dest_file: &Path,
    _compression_method: &str,
    _compression_level: u8,
) -> std::io::Result<()> {
    let mut zipper = ZipArchive::new();
    
    // We need to collect paths into a vector to ensure they live long enough for the zipper
    // mtzip stores references to paths, so the paths must outlive the zipper.write() call
    let mut entries = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        entries.push(entry.path().to_path_buf());
    }

    println!("[ZIP] Found {} entries to compress in {:?}", entries.len(), src_dir);

    // 1. Add all directories first
    let mut dir_count = 0;
    for path in &entries {
        if path.is_dir() {
            let name = path.strip_prefix(src_dir).unwrap();
            if name.as_os_str().is_empty() {
                continue;
            }
            let name_str = name.to_string_lossy().replace('\\', "/");
            zipper.add_directory(name_str.to_owned()).done();
            dir_count += 1;
        }
    }
    println!("[ZIP] Added {} directories", dir_count);

    // 2. Add all files
    let mut file_count = 0;
    for path in &entries {
        if path.is_file() {
            let name = path.strip_prefix(src_dir).unwrap();
            let name_str = name.to_string_lossy().replace('\\', "/");
            zipper.add_file_from_fs(path.as_path(), name_str.to_owned()).done();
            file_count += 1;
        }
    }
    println!("[ZIP] Added {} files", file_count);

    println!("[ZIP] Writing ZIP file to {:?}", dest_file);
    let mut file = File::create(dest_file)?;
    zipper.write(&mut file).map_err(|e| {
        println!("[ZIP ERROR] Failed to write: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;
    println!("[ZIP] Successfully written {} bytes", file.metadata()?.len());
    Ok(())
}
