use std::fs;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use rayon::prelude::*;


/// Debug mode - set to true to log all collected paths to file
const DEBUG: bool = true;

/// Collects ALL file paths from map configuration files without categorization
/// Returns a HashSet to eliminate duplicates
pub fn collect_all_dependencies(map_folder: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let mut all_paths = HashSet::new();

    // Phase 1: Collect from all map tiles (.map files)
    collect_from_map_tiles(map_folder, &mut all_paths)?;

    // Phase 1: Collect from global.cfg
    let global_cfg = map_folder.join("global.cfg");
    if global_cfg.exists() {
        collect_from_global_cfg(&global_cfg, &mut all_paths)?;
    }

    // Phase 1: Collect from ailists.cfg
    let ailists_cfg = map_folder.join("ailists.cfg");
    if ailists_cfg.exists() {
        collect_from_ailists_cfg(&ailists_cfg, &mut all_paths)?;
    }

    // Phase 1: Collect from parklist_p.txt
    let parklist = map_folder.join("parklist_p.txt");
    if parklist.exists() {
        collect_from_parklist(&parklist, &mut all_paths)?;
    }

    // Phase 1: Collect from humans.txt
    let humans_txt = map_folder.join("humans.txt");
    if humans_txt.exists() {
        collect_from_text_file(&humans_txt, &mut all_paths)?;
    }

    // Phase 1: Collect from drivers.txt
    let drivers_txt = map_folder.join("drivers.txt");
    if drivers_txt.exists() {
        collect_from_text_file(&drivers_txt, &mut all_paths)?;
    }

    // Scan any other .txt files in map folder recursively (excluding the ones already processed)
    // Collect all txt files first
    let txt_files: Vec<PathBuf> = walkdir::WalkDir::new(map_folder)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("txt")).unwrap_or(false))
        .map(|e| e.path().to_path_buf())
        .collect();

    // Process txt files in parallel
    let txt_results: Vec<HashSet<String>> = txt_files.par_iter()
        .map(|p| {
            let mut local_paths = HashSet::new();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            if name != "parklist_p.txt" && name != "humans.txt" && name != "drivers.txt" && name != "debug_collected_paths.txt" {
                let _ = collect_from_text_file(&p, &mut local_paths);
            }
            local_paths
        })
        .collect();

    // Merge results
    for res in txt_results {
        all_paths.extend(res);
    }

    // Debug: Log all collected paths
    if DEBUG {
        log_to_file(&all_paths, map_folder)?;
    }

    Ok(all_paths)
}

// ============================================================================
// COLLECTION FUNCTIONS - Each extracts paths from specific map config files
// ============================================================================

fn collect_from_map_tiles(
    map_folder: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut map_files = Vec::new();

    // Collect from regular .map files
    if let Ok(entries) = fs::read_dir(map_folder) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("map") {
                    map_files.push(path);
                }
            }
        }
    }

    // Collect from Chrono subfolder .map files
    let chrono_path = map_folder.join("Chrono");
    if chrono_path.exists() {
        for entry in walkdir::WalkDir::new(&chrono_path)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("map") {
                map_files.push(entry.path().to_path_buf());
            }
        }
    }

    // Process map files in parallel
    let results: Vec<Result<HashSet<String>, String>> = map_files.par_iter()
        .map(|path| {
            let mut local_paths = HashSet::new();
            match collect_from_single_map_file(path, &mut local_paths) {
                Ok(_) => Ok(local_paths),
                Err(e) => Err(format!("Error processing {:?}: {}", path, e))
            }
        })
        .collect();

    // Merge results
    for res in results {
        match res {
            Ok(paths) => all_paths.extend(paths),
            Err(e) => eprintln!("{}", e), // Log error but continue
        }
    }

    Ok(())
}

fn collect_from_single_map_file(
    map_file: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Map files typically use UTF-16LE encoding, but we try robustly
    let bytes = fs::read(map_file)?;
    
    let content = if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16LE with BOM (most common for .map files)
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(UTF_16LE))
            .build(std::io::Cursor::new(&bytes));
        let mut s = String::new();
        match decoder.read_to_string(&mut s) {
            Ok(_) => s,
            Err(_) => {
                // Fallback to lossy decoding
                let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                cow.into_owned()
            }
        }
    } else if let Ok(s) = String::from_utf8(bytes.clone()) {
        s
    } else {
        // Fallback to Windows-1252
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
        cow.into_owned()
    };
    
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // [spline] section (includes [spline_h])
        if trimmed == "[spline]" || trimmed == "[spline_h]" {
            lines.next(); // skip ID
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }

        // [object] and [splineAttachement] sections
        if trimmed == "[object]" || trimmed == "[splineAttachement]" {
            lines.next(); // skip ID
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }
    }

    // Generic scan: extract any path-like strings from entire file content
    extract_paths_generic(&content, all_paths);

    Ok(())
}

fn collect_from_global_cfg(
    global_cfg: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // global.cfg typically uses UTF-16LE encoding, but we try robustly
    let bytes = fs::read(global_cfg)?;
    
    let content = if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16LE with BOM (most common for global.cfg)
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(UTF_16LE))
            .build(std::io::Cursor::new(&bytes));
        let mut s = String::new();
        match decoder.read_to_string(&mut s) {
            Ok(_) => s,
            Err(_) => {
                // Fallback to lossy decoding
                let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
                cow.into_owned()
            }
        }
    } else if let Ok(s) = String::from_utf8(bytes.clone()) {
        s
    } else {
        // Fallback to Windows-1252
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&bytes);
        cow.into_owned()
    };
    
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // [groundtex]
        if trimmed == "[groundtex]" {
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }

        // [humans]
        if trimmed == "[humans]" {
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }

        // [spline]
        if trimmed == "[spline]" {
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }

        // [moneysystem]
        if trimmed == "[moneysystem]" {
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }

        // [ticketpack]
        if trimmed == "[ticketpack]" {
            if let Some(path_line) = lines.next() {
                let path = path_line.trim().to_string();
                if !path.is_empty() {
                    try_insert_path(all_paths, &path);
                }
            }
        }
    }

    // Generic scan
    extract_paths_generic(&content, all_paths);

    Ok(())
}

fn collect_from_ailists_cfg(
    ailists_cfg: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_file_robust(ailists_cfg)?;
    let mut lines = content.lines().peekable();
    let mut in_depot_typgroup = false;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.starts_with("[aigroup_depot_typgroup") {
            in_depot_typgroup = true;
        }

        if in_depot_typgroup {
            // Check if this line ends with vehicle extension (inside depot section)
            if trimmed.ends_with(".bus") || trimmed.ends_with(".ovh") || 
               trimmed.ends_with(".zug") || trimmed.ends_with(".sco") {
                let path = trimmed.to_string();
                try_insert_path(all_paths, &path);
                in_depot_typgroup = false;
            }
        } else {
            // Regular lines: "path<whitespace>count"
            if let Some(first_part) = trimmed.split_whitespace().next() {
                if first_part.ends_with(".bus") || first_part.ends_with(".ovh") || 
                   first_part.ends_with(".zug") || first_part.ends_with(".sco") {
                    let path = first_part.to_string();
                    try_insert_path(all_paths, &path);
                }
            }
        }
    }

    // Generic scan
    extract_paths_generic(&content, all_paths);

    Ok(())
}

fn collect_from_parklist(
    parklist: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_file_robust(parklist)?;

    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }

        // Check if line contains any known file extension
        if trimmed.ends_with(".sco") || trimmed.ends_with(".ovh") || 
           trimmed.ends_with(".bus") || trimmed.ends_with(".zug") ||
           trimmed.ends_with(".sli") || trimmed.ends_with(".hum") {
            let path = trimmed.to_string();
            try_insert_path(all_paths, &path);
        }
    }

    // Generic scan
    extract_paths_generic(&content, all_paths);

    Ok(())
}

/// Robustly read a file with various encodings: UTF-8, UTF-16LE, Windows-1252
fn read_file_robust(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(file_path)?;

    let content = if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16LE with BOM
        let mut decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(UTF_16LE))
            .build(std::io::Cursor::new(bytes));
        let mut s = String::new();
        decoder.read_to_string(&mut s)?;
        s
    } else if let Ok(s) = String::from_utf8(bytes.clone()) {
        s
    } else {
        // Fallback to Windows-1252 (common for OMSI text files)
        let (cow, _enc_used, _had_errors) = encoding_rs::WINDOWS_1252.decode(&bytes);
        cow.into_owned()
    };

    Ok(content)
}

fn collect_from_text_file(
    text_file: &Path,
    all_paths: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = read_file_robust(text_file)?;
    extract_paths_generic(&content, all_paths);
    Ok(())
}

// ============================================================================
// GENERIC PATH SCANNER
// ============================================================================

fn extract_paths_generic(content: &str, all_paths: &mut HashSet<String>) {
    // Known file extensions (lowercase)
    const EXTS: &[&str] = &[
        "sco", "ovh", "bus", "zug", "sli", "hum", "wav", "jpg", "jpeg", "bmp", "dds", "png", "tga", "cfg", "osc", "x", "o3d", "surf", "map", "otp", "txt",
    ];

    let bytes = content.as_bytes();
    let len = bytes.len();

    for &ext in EXTS {
        let ext_bytes = ext.as_bytes();
        let mut i = 0;
        while i < len {
            // Look for '.' followed by the extension (ASCII, case-insensitive)
            if bytes[i] == b'.' {
                let mut matched = true;
                let mut j = 0;
                while matched && j < ext_bytes.len() {
                    let k = i + 1 + j;
                    if k >= len {
                        matched = false;
                        break;
                    }
                    let b = bytes[k].to_ascii_lowercase();
                    if b != ext_bytes[j] {
                        matched = false;
                    }
                    j += 1;
                }

                if matched {
                    // Found a ".ext" at position i..i+1+ext.len()
                    // Walk backward over ASCII path bytes
                    let mut start = i;
                    while start > 0 {
                        let b = bytes[start - 1];
                        if is_path_byte(b) {
                            start -= 1;
                        } else {
                            break;
                        }
                    }

                    // Walk forward over ASCII path bytes
                    let mut end = i + 1 + ext_bytes.len();
                    while end < len {
                        let b = bytes[end];
                        if is_path_byte(b) {
                            end += 1;
                        } else {
                            break;
                        }
                    }

                    // Extract candidate from bytes safely
                    let mut cand = String::from_utf8_lossy(&bytes[start..end]).trim().to_string();
                    // Trim common trailing punctuation
                    cand = cand
                        .trim_end_matches([';', ',', ')', ']', '"', '\'', '»', '“', '”', '’'])
                        .to_string();
                    cand = cand.trim_start_matches(['"', '\'', '(', '[']).to_string();

                    // Heuristic: must contain a separator or start with known folders
                    let lc = cand.to_lowercase();
                    let has_sep = lc.contains('/') || lc.contains('\\');
                    let starts_with_known = lc.starts_with("sceneryobjects")
                        || lc.starts_with("splines")
                        || lc.starts_with("vehicles")
                        || lc.starts_with("humans")
                        || lc.starts_with("texture")
                        || lc.starts_with("sound")
                        || lc.starts_with("script");
                    // Also allow trains folder
                    let starts_with_known = starts_with_known || lc.starts_with("trains");

                    // Ensure it ends with the ext (case-insensitive)
                    let ends_with_ext = lc.ends_with(&format!(".{ext}"));

                    if ends_with_ext && (has_sep || starts_with_known) {
                        let normalized = cand.replace('/', "\\");
                        try_insert_path(all_paths, &normalized);
                    }
                }
            }
            i += 1;
        }
    }
}

fn is_path_byte(b: u8) -> bool {
    // Accept ASCII path characters, spaces, separators, and extended bytes (>= 0x80)
    b.is_ascii_alphanumeric()
        || matches!(b as char, '_' | '-' | '.' | '/' | '\\' | ':' | ' ' | '\t')
        || b == b'#'
        || b >= 0x80
}

// ============================================================================
// PATH NORMALIZATION & VALIDATION
// ============================================================================

/// Normalize separators and validate path candidates before insertion.
/// Returns None if the path is invalid (e.g., starts with a backslash after trimming).
fn normalize_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Unify separators to Windows-style
    let mut s = trimmed.replace('/', "\\");

    // Remove optional leading ./ or .\ segments
    while s.starts_with(".\\") || s.starts_with("./") {
        s = s[2..].to_string();
        s = s.replace('/', "\\");
    }

    // Reject if first non-space char is a backslash (e.g., "\\kos1\\kos1.sco")
    if s.starts_with('\\') {
        return None;
    }

    Some(s)
}

/// Centralized insertion with validation
fn try_insert_path(all_paths: &mut HashSet<String>, raw: &str) {
    if let Some(p) = normalize_path(raw) {
        if is_plausible_path(&p) {
            all_paths.insert(p);
        }
    }
}

/// Validates that a normalized path starts with one of the known root folders.
/// Example: "sceneryobjects\xyz\abc.sco" → valid; "Borovka\cekarna2.sco" → invalid
fn is_plausible_path(normalized: &str) -> bool {
    const KNOWN_ROOTS: &[&str] = &[
        "sceneryobjects",
        "vehicles",
        "splines",
        "humans",
        "texture",
        "sound",
        "script",
        "trains",
        "money",
        "ticketpacks",
    ];

    let lower = normalized.to_lowercase();
    for root in KNOWN_ROOTS {
        let prefix = format!("{}\\", root);
        if lower.starts_with(&prefix) {
            return true;
        }
    }
    false
}

// ============================================================================
// DEBUG LOGGING
// ============================================================================

fn log_to_file(
    all_paths: &HashSet<String>,
    map_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = map_folder.join("debug_collected_paths.txt");
    
    // Group by extension for readability and summary
    let mut grouped: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for path in all_paths {
        let ext = path.rsplit('.').next().unwrap_or("unknown").to_lowercase();
        grouped.entry(ext).or_insert_with(Vec::new).push(path.clone());
    }

    // Prepare summary counts sorted by extension
    let mut extensions: Vec<_> = grouped.keys().cloned().collect();
    extensions.sort();

    let mut summary_lines = String::new();
    summary_lines.push_str("Summary by extension:\n");
    for ext in &extensions {
        if let Some(paths) = grouped.get(ext) {
            summary_lines.push_str(&format!("  .{}: {}\n", ext, paths.len()));
        }
    }

    let mut log_content = String::new();
    log_content.push_str(&format!("=== PHASE 1: Collected {} file paths ===\n", all_paths.len()));
    log_content.push_str(&format!("Map folder: {}\n", map_folder.display()));
    log_content.push_str("Timestamp: [Generated on extraction]\n\n");
    // Add summary at the top
    log_content.push_str(&summary_lines);
    log_content.push('\n');

    // Detailed listing grouped by extension
    for ext in extensions {
        if let Some(paths) = grouped.get(&ext) {
            log_content.push_str(&format!("\n.{} files ({}):\n", ext, paths.len()));
            log_content.push_str(&"─".repeat(60));
            log_content.push('\n');
            for path in paths {
                log_content.push_str(&format!("  {}\n", path));
            }
        }
    }

    log_content.push_str(&format!("\n{}\n", "=".repeat(60)));
    log_content.push_str(&format!("Total: {} paths collected\n", all_paths.len()));

    fs::write(&log_path, log_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_collect_all_dependencies() {
        // This will be tested with actual map folder
    }
}
