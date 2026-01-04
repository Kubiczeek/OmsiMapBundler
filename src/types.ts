// Interface definitions for the application

export interface ValidationResult {
  valid: boolean;
  missing_files: string[];
  error?: string;
}

export interface DependencyResult {
  sceneryobjects: string[];
  splines: string[];
  textures: string[];
  humans: string[];
  vehicles: string[];
  money_systems: string[];
  ticket_packs: string[];
  tile_maps: string[];
  error?: string;
}

export interface BundleRequest {
  map_folder: string;
  addon_folder?: string;
  output_folder?: string;
  zip_name?: string;
  readme_path?: string;
  compression_method?: string;
  compression_level?: number;
}

export interface BundleResult {
  success: boolean;
  output_path?: string;
  error?: string;
}
