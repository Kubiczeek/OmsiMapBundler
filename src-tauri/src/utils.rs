use std::path::Path;
use std::fs;
use std::fs::File;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;
use walkdir::WalkDir;

// Helper function to copy a file and create parent directories
pub fn copy_file_with_folders(src: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)?;
    Ok(())
}

// Helper function to copy entire directory recursively
pub fn copy_dir_all(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    
    Ok(())
}

// Helper function to create ZIP from directory
pub fn create_zip(src_dir: &Path, dest_file: &Path) -> std::io::Result<()> {
    let file = File::create(dest_file)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap();
        
        // Convert path to string with forward slashes for ZIP format
        let name_str = name.to_string_lossy().replace('\\', "/");
        
        if path.is_file() {
            zip.start_file(&name_str, options)?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(&name_str, options)?;
        }
    }
    
    zip.finish()?;
    Ok(())
}
