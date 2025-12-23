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
  tile_maps: string[];
  error?: string;
}

export interface BundleRequest {
  map_folder: string;
  output_folder?: string;
  zip_name?: string;
  readme_path?: string;
}

export interface BundleResult {
  success: boolean;
  output_path?: string;
  error?: string;
}
