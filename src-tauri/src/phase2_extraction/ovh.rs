use std::path::Path;
use std::collections::HashSet;
use crate::phase2_extraction::{utils, cfg};

/// Extract all dependencies from a .ovh file (AI vehicles in Sceneryobjects)
/// Returns a set of file paths relative to OMSI root folder
pub fn extract_ovh_dependencies(ovh_path: &str, omsi_root: &Path) -> Option<HashSet<String>> {
    let full_ovh_path = omsi_root.join(ovh_path);
    
    if !full_ovh_path.exists() {
        println!("OVH file not found: {:?}", full_ovh_path);
        return None;
    }
    
    let mut dependencies = HashSet::new();
    
    // Add the .ovh file itself
    dependencies.insert(ovh_path.to_string());
    
    // Read .ovh file with Windows-1252 encoding
    let ovh_content = match utils::read_file_windows1252(&full_ovh_path) {
        Some(content) => content,
        None => return None,
    };
    
    let ovh_folder = Path::new(ovh_path).parent().unwrap_or(Path::new(""));
    let mut lines = ovh_content.lines();
    
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        
        // Extract model config from [model] section
        if trimmed == "[model]" {
            if let Some(model_line) = lines.next() {
                let model_file = model_line.trim();
                if !model_file.is_empty() && model_file.ends_with(".cfg") {
                    // Try multiple locations for the model file
                    // Option 1: relative to ovh folder + model subfolder
                    let option1 = ovh_folder.join("model").join(model_file);
                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                    let test1 = omsi_root.join(&option1_str);
                    
                    // Option 2: relative to ovh folder directly
                    let option2 = ovh_folder.join(model_file);
                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                    let test2 = omsi_root.join(&option2_str);
                    
                    // Option 3: try as-is (might have ..\\ path)
                    let option3 = model_file.replace('/', "\\");
                    let test3 = if option3.starts_with("..\\") {
                        // Resolve relative path from ovh folder
                        ovh_folder.join(&option3).to_string_lossy().replace('/', "\\")
                    } else {
                        option3.clone()
                    };
                    let test3_full = omsi_root.join(&test3);
                    
                    if test1.exists() {
                        dependencies.insert(option1_str);
                    } else if test2.exists() {
                        dependencies.insert(option2_str);
                    } else if test3_full.exists() {
                        dependencies.insert(test3);
                    }
                }
            }
        }
        
        // Extract sound configs from [sound] sections
        if trimmed == "[sound]" {
            if let Some(sound_line) = lines.next() {
                let sound_file = sound_line.trim();
                if !sound_file.is_empty() && sound_file.ends_with(".cfg") {
                    // Handle relative paths like ..\..\Sounds\AI_Cars\sound.cfg
                    let sound_path = if sound_file.starts_with("..\\") || sound_file.starts_with("../") {
                        ovh_folder.join(sound_file).to_string_lossy().replace('/', "\\")
                    } else {
                        let option1 = ovh_folder.join("sound").join(sound_file);
                        let option1_str = option1.to_string_lossy().replace('/', "\\");
                        let test1 = omsi_root.join(&option1_str);
                        
                        let option2 = ovh_folder.join(sound_file);
                        let option2_str = option2.to_string_lossy().replace('/', "\\");
                        let test2 = omsi_root.join(&option2_str);
                        
                        if test1.exists() {
                            option1_str
                        } else if test2.exists() {
                            option2_str
                        } else {
                            sound_file.replace('/', "\\")
                        }
                    };
                    
                    let test_sound = omsi_root.join(&sound_path);
                    if test_sound.exists() {
                        dependencies.insert(sound_path.clone());
                        
                        // Extract nested sound dependencies
                        if let Some(sound_deps) = cfg::extract_sound_cfg_dependencies(&sound_path, omsi_root) {
                            dependencies.extend(sound_deps);
                        }
                    }
                }
            }
        }
        
        // Extract varname lists from [varnamelist] sections
        if trimmed == "[varnamelist]" {
            // Next line is count
            lines.next();
            // Following lines are the varlist paths (count times)
            if let Some(count_str) = lines.next() {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    for _ in 0..count {
                        if let Some(varlist_line) = lines.next() {
                            let varlist_file = varlist_line.trim();
                            if !varlist_file.is_empty() && varlist_file.ends_with(".txt") {
                                // Handle relative paths like ..\..\Scripts\AI_Cars\AI_varlist.txt
                                let varlist_path = if varlist_file.starts_with("..\\") || varlist_file.starts_with("../") {
                                    ovh_folder.join(varlist_file).to_string_lossy().replace('/', "\\")
                                } else {
                                    let option1 = ovh_folder.join("script").join(varlist_file);
                                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                                    let test1 = omsi_root.join(&option1_str);
                                    
                                    let option2 = ovh_folder.join(varlist_file);
                                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                                    let test2 = omsi_root.join(&option2_str);
                                    
                                    if test1.exists() {
                                        option1_str
                                    } else if test2.exists() {
                                        option2_str
                                    } else {
                                        varlist_file.replace('/', "\\")
                                    }
                                };
                                
                                let test_varlist = omsi_root.join(&varlist_path);
                                if test_varlist.exists() {
                                    dependencies.insert(varlist_path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Extract scripts from [script] sections
        if trimmed == "[script]" {
            // Next line is count
            if let Some(count_str) = lines.next() {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    for _ in 0..count {
                        if let Some(script_line) = lines.next() {
                            let script_file = script_line.trim();
                            if !script_file.is_empty() && script_file.ends_with(".osc") {
                                // Handle relative paths like ..\..\Scripts\AI_Cars\main_AI.osc
                                let script_path = if script_file.starts_with("..\\") || script_file.starts_with("../") {
                                    ovh_folder.join(script_file).to_string_lossy().replace('/', "\\")
                                } else {
                                    let option1 = ovh_folder.join("script").join(script_file);
                                    let option1_str = option1.to_string_lossy().replace('/', "\\");
                                    let test1 = omsi_root.join(&option1_str);
                                    
                                    let option2 = ovh_folder.join(script_file);
                                    let option2_str = option2.to_string_lossy().replace('/', "\\");
                                    let test2 = omsi_root.join(&option2_str);
                                    
                                    if test1.exists() {
                                        option1_str
                                    } else if test2.exists() {
                                        option2_str
                                    } else {
                                        script_file.replace('/', "\\")
                                    }
                                };
                                
                                let test_script = omsi_root.join(&script_path);
                                if test_script.exists() {
                                    dependencies.insert(script_path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Extract const files from [constfile] sections
        if trimmed == "[constfile]" {
            // Next line is count
            if let Some(count_str) = lines.next() {
                if let Ok(count) = count_str.trim().parse::<usize>() {
                    for _ in 0..count {
                        if let Some(const_line) = lines.next() {
                            let const_file = const_line.trim();
                            if !const_file.is_empty() && const_file.ends_with(".txt") {
                                // Try multiple locations
                                let option1 = ovh_folder.join("script").join(const_file);
                                let option1_str = option1.to_string_lossy().replace('/', "\\");
                                let test1 = omsi_root.join(&option1_str);
                                
                                let option2 = ovh_folder.join(const_file);
                                let option2_str = option2.to_string_lossy().replace('/', "\\");
                                let test2 = omsi_root.join(&option2_str);
                                
                                if test1.exists() {
                                    dependencies.insert(option1_str);
                                } else if test2.exists() {
                                    dependencies.insert(option2_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Some(dependencies)
}
