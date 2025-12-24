use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use std::collections::HashSet;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use crate::types::DependencyResult;

fn add_texture_with_variants(tex_path: &str, omsi_root: &Path, textures: &mut HashSet<String>) {
    // Add the main texture
    textures.insert(tex_path.to_string());
    
    // Check for .cfg and .surf files
    let cfg_path = format!("{}.cfg", tex_path);
    let surf_path = format!("{}.surf", tex_path);
    
    if let Some(root) = Some(omsi_root) {
        // Check if .cfg file exists
        let cfg_full_path = root.join(&cfg_path);
        if cfg_full_path.exists() {
            textures.insert(cfg_path);
        }
        
        // Check if .surf file exists
        let surf_full_path = root.join(&surf_path);
        if surf_full_path.exists() {
            textures.insert(surf_path);
        }
    }
    
    // Check seasonal variants
    check_seasonal_variants(tex_path, omsi_root, textures);
}

fn check_seasonal_variants(tex_path: &str, omsi_root: &Path, textures: &mut HashSet<String>) {
    let lower_path = tex_path.to_lowercase();
    // Only check if it is in the main Texture folder
    if lower_path.starts_with("texture\\") || lower_path.starts_with("texture/") {
        let path_obj = Path::new(tex_path);
        if let Some(file_name) = path_obj.file_name() {
            let seasons = ["Winter", "WinterSnow", "Spring", "Fall"];
            
            // Get the parent directory of the texture (e.g., "Texture" or "Texture\Kostelec-Bor")
            if let Some(parent_dir) = path_obj.parent() {
                for season in seasons {
                    // Construct path: parent_dir/Season/filename
                    // E.g., Texture\Kostelec-Bor\Winter\alex.bmp or Texture\Winter\gras.bmp
                    let seasonal_rel_path = parent_dir.join(season).join(file_name);
                    let full_path = omsi_root.join(&seasonal_rel_path);
                    
                    if full_path.exists() {
                        textures.insert(seasonal_rel_path.to_string_lossy().replace('/', "\\"));
                    }
                }
            }
        }
    }
}

// Extract all dependencies from map files
pub fn extract_dependencies(map_folder: String) -> DependencyResult {
    // Debug logging toggle - set to true to enable detailed debug logs
    const DEBUG_LOGGING: bool = false;
    
    let path = Path::new(&map_folder);
    let omsi_root = path.parent().and_then(|p| p.parent());
    
    let mut sceneryobjects = HashSet::new();
    let mut splines = HashSet::new();
    let mut textures = HashSet::new();
    let mut humans = HashSet::new();
    let mut vehicles = HashSet::new();
    let mut tile_maps = Vec::new();
    let mut money_systems = HashSet::new();
    let mut ticket_packs = HashSet::new();
    
    // Create debug log file (only if DEBUG_LOGGING is enabled)
    let debug_log_path = path.join("bundle_debug.log");
    let mut debug_log = if DEBUG_LOGGING {
        match File::create(&debug_log_path) {
            Ok(file) => Some(BufWriter::new(file)),
            Err(e) => {
                println!("Warning: Could not create debug log: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    if let Some(ref mut log) = debug_log {
        let _ = writeln!(log, "=== OMSI Map Bundler Debug Log ===");
        let _ = writeln!(log, "Map folder: {}", map_folder);
        let _ = writeln!(log, "OMSI root: {:?}\n", omsi_root);
    }
    
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
                        
                        // Extract ground textures - format is [groundtex] then 2 texture paths
                        if trimmed == "[groundtex]" {
                            // Read first texture (main texture)
                            if let Some(tex_line) = lines_iter.next() {
                                let tex_path = tex_line.trim();
                                let lower_tex = tex_path.to_lowercase();
                                // Support .bmp, .jpg, .jpeg, .png, .dds, .tga
                                if lower_tex.ends_with(".bmp") || lower_tex.ends_with(".jpg") || 
                                   lower_tex.ends_with(".jpeg") || lower_tex.ends_with(".png") || 
                                   lower_tex.ends_with(".dds") || lower_tex.ends_with(".tga") {
                                    
                                    // Add texture with .cfg, .surf variants and seasonal variants
                                    if let Some(root) = omsi_root {
                                        add_texture_with_variants(tex_path, root, &mut textures);
                                    }
                                }
                            }
                            
                            // Read second texture (detail/normal texture) - always present in [groundtex]
                            if let Some(detail_line) = lines_iter.next() {
                                let detail_path = detail_line.trim();
                                let lower_detail = detail_path.to_lowercase();
                                if lower_detail.ends_with(".bmp") || lower_detail.ends_with(".jpg") || 
                                   lower_detail.ends_with(".jpeg") || lower_detail.ends_with(".png") || 
                                   lower_detail.ends_with(".dds") || lower_detail.ends_with(".tga") {
                                    
                                    // Add texture with .cfg, .surf variants and seasonal variants
                                    if let Some(root) = omsi_root {
                                        add_texture_with_variants(detail_path, root, &mut textures);
                                    }
                                }
                            }
                        }

                        // Extract money system
                        if trimmed == "[moneysystem]" {
                            if let Some(money_line) = lines_iter.next() {
                                let money_path = money_line.trim();
                                if !money_path.is_empty() {
                                    money_systems.insert(money_path.to_string());
                                }
                            }
                        }

                        // Extract ticket pack
                        if trimmed == "[ticketpack]" {
                            if let Some(ticket_line) = lines_iter.next() {
                                let ticket_path = ticket_line.trim();
                                if !ticket_path.is_empty() {
                                    ticket_packs.insert(ticket_path.to_string());
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
                        // Write tile content to debug log
                        if let Some(ref mut log) = debug_log {
                            let _ = writeln!(log, "\n=== TILE: {} ===", tile);
                            let _ = writeln!(log, "{}", tile_content);
                            let _ = writeln!(log, "=== END TILE ===\n");
                        }
                        
                        let mut lines = tile_content.lines();
                    
                        while let Some(line) = lines.next() {
                            let trimmed = line.trim();
                            
                            // Extract splines - format: [spline] -> number -> path -> ...
                            if trimmed == "[spline]" || trimmed == "[spline_h]" {
                                lines.next(); // Skip number line
                                if let Some(path_line) = lines.next() {
                                    let path = path_line.trim();
                                    if path.ends_with(".sli") {
                                        splines.insert(path.to_string());
                                    } else {
                                        println!("Warning: Spline path does not end with .sli: {}", path);
                                        println!("Map tile: {}", tile);
                                    }
                                }
                            }
                            
                            // Extract objects - format: [object] -> number -> path -> ...
                            if trimmed == "[object]" || trimmed == "[splineAttachement]" {
                                lines.next(); // Skip number line
                                if let Some(path_line) = lines.next() {
                                    let path = path_line.trim();
                                    if path.ends_with(".sco") {
                                        sceneryobjects.insert(path.to_string());
                                    } else {
                                        println!("Warning: Object path does not end with .sco: {}", path);
                                        println!("Map tile: {}", tile);
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
                                            // Write chrono tile content to debug log
                                            if let Some(ref mut log) = debug_log {
                                                let _ = writeln!(log, "\n=== CHRONO TILE: {} ===", map_file_path.display());
                                                let _ = writeln!(log, "{}", map_content);
                                                let _ = writeln!(log, "=== END CHRONO TILE ===\n");
                                            }
                                            
                                            let mut lines = map_content.lines();
                                            
                                            while let Some(line) = lines.next() {
                                                let trimmed = line.trim();
                                                
                                                if trimmed == "[spline]" || trimmed == "[spline_h]" {
                                                    lines.next();
                                                    if let Some(path_line) = lines.next() {
                                                        let path = path_line.trim();
                                                        if path.ends_with(".sli") {
                                                            splines.insert(path.to_string());
                                                        } else {
                                                            println!("Warning: Spline path does not end with .sli: {}", path);
                                                            println!("Map tile: {}", map_entry.path().display());
                                                        }
                                                    }
                                                }
                                                
                                                if trimmed == "[object]" || trimmed == "[splineAttachement]" {
                                                    lines.next();
                                                    if let Some(path_line) = lines.next() {
                                                        let path = path_line.trim();
                                                        if path.ends_with(".sco") {
                                                            sceneryobjects.insert(path.to_string());
                                                        } else {
                                                            println!("Warning: Object path does not end with .sco: {}", path);
                                                            println!("Map tile: {}", map_entry.path().display());
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
        let mut lines_iter = ailists_content.lines().peekable();
        let mut in_depot_typgroup = false;
        
        while let Some(line) = lines_iter.next() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }
            
            // Check if we're entering a depot typgroup section
            if trimmed.starts_with("[aigroup_depot_typgroup") {
                in_depot_typgroup = true;
                continue;
            }
            
            // Check if we're leaving any section
            if trimmed == "[end]" {
                in_depot_typgroup = false;
                continue;
            }
            
            // Skip other section headers
            if trimmed.starts_with('[') {
                in_depot_typgroup = false;
                continue;
            }
            
            // If we're in a depot typgroup, the first non-empty line after the section header is the vehicle path
            if in_depot_typgroup {
                // This should be the vehicle path line (e.g., "Vehicles\SOR\SORc10 AUTOMAT.bus")
                if trimmed.ends_with(".bus") || trimmed.ends_with(".ovh") || trimmed.ends_with(".zug") || trimmed.ends_with(".sco") {
                    vehicles.insert(trimmed.to_string());
                    // After we found the vehicle path, the remaining lines in this section are instances
                    // We need to skip them until we hit [end] or another section
                    in_depot_typgroup = false;
                }
            } else {
                // Normal aigroup entries: format is "path<tab/space>number"
                if let Some(first_part) = trimmed.split_whitespace().next() {
                    if first_part.ends_with(".bus") || first_part.ends_with(".ovh") || first_part.ends_with(".zug") || first_part.ends_with(".sco") {
                        vehicles.insert(first_part.to_string());
                    }
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
    println!("  - {} money systems", money_systems.len());
    println!("  - {} ticket packs", ticket_packs.len());
    
    // Write all found dependencies to debug log
    if let Some(ref mut log) = debug_log {
        let _ = writeln!(log, "\n=== FOUND DEPENDENCIES ===");
        let _ = writeln!(log, "\nTiles checked: {}", tile_maps.len());
        
        let _ = writeln!(log, "\n--- SPLINES ({}) ---", splines.len());
        let mut splines_sorted: Vec<_> = splines.iter().collect();
        splines_sorted.sort();
        for spline in &splines_sorted {
            let _ = writeln!(log, "  {}", spline);
        }
        
        let _ = writeln!(log, "\n--- SCENERYOBJECTS ({}) ---", sceneryobjects.len());
        let mut scos_sorted: Vec<_> = sceneryobjects.iter().collect();
        scos_sorted.sort();
        for sco in &scos_sorted {
            let _ = writeln!(log, "  {}", sco);
        }
        
        let _ = writeln!(log, "\n--- TEXTURES ({}) ---", textures.len());
        let mut textures_sorted: Vec<_> = textures.iter().collect();
        textures_sorted.sort();
        for tex in &textures_sorted {
            let _ = writeln!(log, "  {}", tex);
        }
        
        let _ = writeln!(log, "\n--- HUMANS ({}) ---", humans.len());
        for human in &humans {
            let _ = writeln!(log, "  {}", human);
        }
        
        let _ = writeln!(log, "\n--- VEHICLES ({}) ---", vehicles.len());
        for vehicle in &vehicles {
            let _ = writeln!(log, "  {}", vehicle);
        }
        
        // Check for specific files if debug_expected.txt exists
        let expected_path = path.join("debug_expected.txt");
        if expected_path.exists() {
            let _ = writeln!(log, "\n=== CHECKING EXPECTED FILES ===");
            if let Ok(expected_content) = fs::read_to_string(&expected_path) {
                for line in expected_content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    
                    let normalized = trimmed.replace('/', "\\");
                    let found = splines.contains(&normalized) ||
                               splines.contains(trimmed) ||
                               sceneryobjects.contains(&normalized) ||
                               sceneryobjects.contains(trimmed) ||
                               textures.contains(&normalized) ||
                               textures.contains(trimmed);
                    
                    if found {
                        let _ = writeln!(log, "  ✓ FOUND: {}", trimmed);
                    } else {
                        let _ = writeln!(log, "  ✗ MISSING: {}", trimmed);
                        println!("⚠ WARNING: Expected file not found: {}", trimmed);
                    }
                }
            }
        } else {
            let _ = writeln!(log, "\n=== TIP ===");
            let _ = writeln!(log, "Create 'debug_expected.txt' in map folder with list of files to check.");
            let _ = writeln!(log, "Example content:");
            let _ = writeln!(log, "  Splines\\Marcel\\Damm1_40m.sli");
            let _ = writeln!(log, "  Sceneryobjects\\something.sco");
        }
        
        let _ = log.flush();
        if DEBUG_LOGGING {
            println!("\nDebug log created: {:?}", debug_log_path);
        }
    }
    
    DependencyResult {
        sceneryobjects: sceneryobjects.into_iter().collect(),
        splines: splines.into_iter().collect(),
        textures: textures.into_iter().collect(),
        humans: humans.into_iter().collect(),
        vehicles: vehicles.into_iter().collect(),
        money_systems: money_systems.into_iter().collect(),
        ticket_packs: ticket_packs.into_iter().collect(),
        tile_maps,
        error: None,
    }
}
