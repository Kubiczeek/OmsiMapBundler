use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use crate::types::DependencyResult;

// Extract all dependencies from map files
pub fn extract_dependencies(map_folder: String) -> DependencyResult {
    let path = Path::new(&map_folder);
    
    let mut sceneryobjects = HashSet::new();
    let mut splines = HashSet::new();
    let mut textures = HashSet::new();
    let mut humans = HashSet::new();
    let mut vehicles = HashSet::new();
    let mut tile_maps = Vec::new();
    
    println!("Starting dependency extraction from: {}", map_folder);
    
    // Check if global.cfg exists
    let global_path = path.join("global.cfg");
    println!("Looking for global.cfg at: {:?}", global_path);
    println!("global.cfg exists: {}", global_path.exists());
    
    // Read global.cfg with UTF-16 LE encoding (has BOM ÿþ)
    match File::open(&global_path) {
        Ok(file) => {
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(UTF_16LE))
                .build(file);
            let mut global_content = String::new();
            
            match decoder.read_to_string(&mut global_content) {
                Ok(_) => {
                    println!("Successfully read global.cfg ({} bytes)", global_content.len());
                    let mut lines_iter = global_content.lines().peekable();
        
                    while let Some(line) = lines_iter.next() {
                        let trimmed = line.trim();
                        
                        // Extract tile maps - format is [map] then 2 numbers then filename
                        if trimmed == "[map]" {
                            lines_iter.next(); // Skip coord1
                            lines_iter.next(); // Skip coord2
                            if let Some(map_line) = lines_iter.next() {
                                let map_file = map_line.trim();
                                if map_file.ends_with(".map") {
                                    tile_maps.push(map_file.to_string());
                                }
                            }
                        }
                        
                        // Extract ground textures - format is [groundtex] then texture path
                        if trimmed == "[groundtex]" {
                            if let Some(tex_line) = lines_iter.next() {
                                let tex_path = tex_line.trim();
                                if tex_path.ends_with(".bmp") {
                                    textures.insert(tex_path.to_string());
                                }
                            }
                            // There might be a detail texture on the next line
                            if let Some(detail_line) = lines_iter.peek() {
                                let detail_path = detail_line.trim();
                                if detail_path.ends_with(".bmp") {
                                    lines_iter.next();
                                    textures.insert(detail_path.to_string());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to decode global.cfg: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to open global.cfg: {}", e);
        }
    }
    
    println!("Found {} tile maps in global.cfg", tile_maps.len());
    
    // Read all tile_*.map files
    for tile in &tile_maps {
        println!("Reading tile map: {}", tile);
        match File::open(path.join(tile)) {
            Ok(file) => {
                let mut decoder = DecodeReaderBytesBuilder::new()
                    .encoding(Some(UTF_16LE))
                    .build(file);
                let mut tile_content = String::new();
                
                match decoder.read_to_string(&mut tile_content) {
                    Ok(_) => {
                        let mut lines = tile_content.lines();
                    
                        while let Some(line) = lines.next() {
                            let trimmed = line.trim();
                            
                            // Extract splines - format: [spline] -> number -> path -> ...
                            if trimmed == "[spline]" {
                                lines.next(); // Skip number line
                                if let Some(path_line) = lines.next() {
                                    let path = path_line.trim();
                                    if path.ends_with(".sli") {
                                        splines.insert(path.to_string());
                                    }
                                }
                            }
                            
                            // Extract objects - format: [object] -> number -> path -> ...
                            if trimmed == "[object]" {
                                lines.next(); // Skip number line
                                if let Some(path_line) = lines.next() {
                                    let path = path_line.trim();
                                    if path.ends_with(".sco") {
                                        sceneryobjects.insert(path.to_string());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to decode tile map {}: {}", tile, e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to open tile map {}: {}", tile, e);
            }
        }
    }
    
    // Read Chrono folder for event-specific map tiles
    let chrono_path = path.join("Chrono");
    if chrono_path.exists() && chrono_path.is_dir() {
        if let Ok(chrono_entries) = fs::read_dir(&chrono_path) {
            for chrono_entry in chrono_entries.flatten() {
                let chrono_subfolder = chrono_entry.path();
                if chrono_subfolder.is_dir() {
                    // Find all .map files in this chrono subfolder
                    if let Ok(map_entries) = fs::read_dir(&chrono_subfolder) {
                        for map_entry in map_entries.flatten() {
                            let map_file_path = map_entry.path();
                            if map_file_path.extension().and_then(|s| s.to_str()) == Some("map") {
                                // Read and parse this chrono .map file
                                match File::open(&map_file_path) {
                                    Ok(file) => {
                                        let mut decoder = DecodeReaderBytesBuilder::new()
                                            .encoding(Some(UTF_16LE))
                                            .build(file);
                                        let mut map_content = String::new();
                                        
                                        if decoder.read_to_string(&mut map_content).is_ok() {
                                            let mut lines = map_content.lines();
                                            
                                            while let Some(line) = lines.next() {
                                                let trimmed = line.trim();
                                                
                                                if trimmed == "[spline]" {
                                                    lines.next();
                                                    if let Some(path_line) = lines.next() {
                                                        let path = path_line.trim();
                                                        if path.ends_with(".sli") {
                                                            splines.insert(path.to_string());
                                                        }
                                                    }
                                                }
                                                
                                                if trimmed == "[object]" {
                                                    lines.next();
                                                    if let Some(path_line) = lines.next() {
                                                        let path = path_line.trim();
                                                        if path.ends_with(".sco") {
                                                            sceneryobjects.insert(path.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Read humans.txt
    if let Ok(humans_content) = fs::read_to_string(path.join("humans.txt")) {
        for line in humans_content.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(".hum") {
                humans.insert(trimmed.to_string());
            }
        }
    }
    
    // Read drivers.txt
    if let Ok(drivers_content) = fs::read_to_string(path.join("drivers.txt")) {
        for line in drivers_content.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(".hum") {
                humans.insert(trimmed.to_string());
            }
        }
    }
    
    // Read parklist_p.txt
    if let Ok(parklist_content) = fs::read_to_string(path.join("parklist_p.txt")) {
        for line in parklist_content.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(".sco") {
                // Check if this .sco is in Vehicles folder (static vehicle)
                let lower_path = trimmed.to_lowercase();
                if lower_path.starts_with("vehicles\\") || lower_path.starts_with("vehicles/") {
                    vehicles.insert(trimmed.to_string());
                } else {
                    sceneryobjects.insert(trimmed.to_string());
                }
            }
        }
    }
    
    // Read ailists.cfg
    if let Ok(ailists_content) = fs::read_to_string(path.join("ailists.cfg")) {
        for line in ailists_content.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with(".bus") || trimmed.ends_with(".ovh") || trimmed.ends_with(".zug") || trimmed.ends_with(".sco") {
                // Split by tab to get just the path
                let parts: Vec<&str> = trimmed.split('\t').collect();
                if !parts.is_empty() {
                    vehicles.insert(parts[0].to_string());
                }
            }
        }
    }
    
    println!("Extraction complete:");
    println!("  - {} scenery objects", sceneryobjects.len());
    println!("  - {} splines", splines.len());
    println!("  - {} textures", textures.len());
    println!("  - {} humans", humans.len());
    println!("  - {} vehicles", vehicles.len());
    
    DependencyResult {
        sceneryobjects: sceneryobjects.into_iter().collect(),
        splines: splines.into_iter().collect(),
        textures: textures.into_iter().collect(),
        humans: humans.into_iter().collect(),
        vehicles: vehicles.into_iter().collect(),
        tile_maps,
        error: None,
    }
}
