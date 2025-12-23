# OMSI Map Bundler

A modern desktop application for bundling OMSI 2 maps with all their dependencies into a single distributable ZIP file. Built with Tauri and TypeScript for a fast, reliable, and user-friendly experience.

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

## Overview

OMSI Map Bundler automates the complex process of packaging OMSI 2 maps by automatically detecting and bundling all dependencies including scenery objects, splines, textures, humans, vehicles, and trains. This tool ensures map creators can easily distribute their work with all required assets in one convenient package.

## Key Features

- 🔍 **Automatic Dependency Detection**: Intelligently scans and identifies all map dependencies
- 📦 **Complete Bundling**: Includes sceneryobjects, splines, textures, humans, vehicles, and trains
- 🚀 **Multi-threaded Compression**: Fast bundling with parallel compression for improved performance
- 🎨 **Modern UI**: Clean, professional interface with light/dark theme support
- 🌍 **Multi-language Support**: Available in English and Czech
- ⚙️ **Flexible Compression Options**: Choose from no compression, fast, balanced, or maximum compression
- 📄 **Optional README Integration**: Include custom README files with your bundle
- ✅ **Map Validation**: Validates map structure before bundling to catch issues early

## Technology Stack

### Frontend

- **Tauri 2.x**: Cross-platform desktop application framework
- **TypeScript 5.6.2**: Type-safe JavaScript for robust development
- **Vite 6.0.3**: Lightning-fast build tool and dev server
- **Vanilla CSS**: Custom styling with CSS variables for theming

### Backend

- **Rust 2021**: High-performance, memory-safe systems programming
- **Core Libraries**:
  - `walkdir 2.x`: Efficient directory traversal
  - `zip 2.2`: ZIP archive creation and manipulation
  - `mtzip 4.0.3`: Multi-threaded compression for faster performance
  - `encoding_rs 0.8`: Character encoding detection and conversion
  - `serde 1.x`: Serialization framework for data exchange

## Project Architecture

The application follows a hybrid architecture combining Tauri's frontend-backend separation:

```
┌─────────────────────────────────────┐
│         Frontend (TypeScript)        │
│  - User Interface & Interactions     │
│  - Form Management & Validation      │
│  - Progress Monitoring               │
│  - Theme & Language Management       │
└──────────────┬──────────────────────┘
               │ Tauri IPC
┌──────────────┴──────────────────────┐
│         Backend (Rust)               │
│  ┌─────────────────────────────────┐│
│  │ Core Modules:                   ││
│  │ • Validation Module             ││
│  │ • Extraction Module             ││
│  │ • Bundling Module               ││
│  │ • Dependency Resolution         ││
│  │ • Utils & File Operations       ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

### Core Components

- **Validation**: Checks map folder structure and required files
- **Extraction**: Parses map files to detect all dependencies
- **Dependency Resolution**: Recursively resolves nested dependencies (e.g., sceneryobjects referencing other objects)
- **Bundling**: Copies and compresses all files into the final ZIP package
- **Progress Tracking**: Real-time progress updates via event system

## Getting Started

### Prerequisites

- **Node.js 18+** and **pnpm** for frontend development
- **Rust 1.70+** for backend compilation
- **OMSI 2** installed (for testing with actual maps)

### Installation

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/omsimapbundler.git
   cd omsimapbundler
   ```

2. **Install dependencies**

   ```bash
   pnpm install
   ```

3. **Run in development mode**

   ```bash
   pnpm tauri dev
   ```

4. **Build for production**
   ```bash
   pnpm tauri build
   ```

The production build will be available in `src-tauri/target/release/bundle/`.

## Project Structure

```
OmsiMapBundler/
├── src/                      # Frontend TypeScript source
│   ├── main.ts              # Application entry point
│   ├── translations.ts      # i18n translations
│   ├── types.ts             # TypeScript type definitions
│   └── styles/              # CSS modules
│       ├── variables.css    # Theme variables
│       ├── base.css         # Base styles
│       └── ...
├── src-tauri/               # Backend Rust source
│   ├── src/
│   │   ├── lib.rs          # Tauri command exports
│   │   ├── validation.rs   # Map validation logic
│   │   ├── extraction.rs   # Dependency extraction
│   │   ├── bundling.rs     # ZIP creation & bundling
│   │   ├── types.rs        # Rust type definitions
│   │   └── dependencies/   # Dependency parsers
│   │       ├── sceneryobject.rs
│   │       ├── spline.rs
│   │       ├── human.rs
│   │       ├── vehicle.rs
│   │       └── train.rs
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── index.html              # Main HTML template
├── package.json            # Node.js dependencies
└── vite.config.ts          # Vite configuration
```

## Usage

1. **Select Map Folder**: Choose your OMSI 2 map folder (located in `OMSI 2/maps/[mapname]`)
2. **Optional Settings**:
   - Add a README file for documentation
   - Specify custom output folder (defaults to map folder)
   - Set custom ZIP name (defaults to map folder name)
   - Choose compression level (Fast, Balanced, Maximum, or None)
3. **Create Bundle**: Click "Create Bundle" and wait for the process to complete
4. **Distribution**: Share the generated ZIP file with all dependencies included

### Important Notes

- ⚠️ **Fonts are NOT included** in the bundle - distribute them separately
- 🚗 **Vehicle bundling** includes entire parent folders due to file complexity
- 💾 **Backup recommended** before bundling important maps
- ⏱️ **Processing time** varies based on map size and compression settings

## Development Workflow

### Development Commands

```bash
# Start development server with hot reload
pnpm dev

# Build frontend only
pnpm build

# Run Tauri development environment
pnpm tauri dev

# Build production release
pnpm tauri build

# TypeScript type checking
tsc --noEmit
```

### Code Organization

- **Frontend**: Uses modular TypeScript with clear separation of concerns
- **Backend**: Follows Rust module system with focused, single-responsibility modules
- **IPC Communication**: Type-safe commands between frontend and backend via Tauri's invoke system

## Coding Standards

### TypeScript

- Use strict TypeScript with full type annotations
- Follow ESLint and Prettier configurations
- Prefer functional programming patterns where appropriate
- Use descriptive variable and function names

### Rust

- Follow Rust idioms and best practices
- Use `Result` and `Option` for error handling
- Document public APIs with doc comments
- Keep functions focused and composable
- Leverage ownership and borrowing for memory safety

### UI/UX Design Principles

- **Professional & Trustworthy**: Clean, rectangular design language
- **Clarity**: Clear call-to-action buttons with intuitive icons
- **Consistency**: Unified color palette and spacing
- **Accessibility**: Support for light/dark themes
- **No Emojis**: Icons only for a professional appearance

### Color Palette (Light Theme)

- **Primary**: `#F1F3E0` - Main background color
- **Secondary**: `#D2DCB6` - Sections and blocks
- **Accent**: `#A1BC98` - Buttons, links, emphasis
- **Neutral**: `#778873` - Text on light backgrounds

## Testing

### Manual Testing Checklist

- [ ] Map folder selection and validation
- [ ] Dependency extraction for various map types
- [ ] ZIP creation with different compression settings
- [ ] Progress tracking during bundling
- [ ] Theme switching (light/dark)
- [ ] Language switching (English/Czech)
- [ ] Error handling for invalid inputs
- [ ] README inclusion in bundle

### Test Cases

1. **Small maps** (< 100 MB) - Quick validation
2. **Large maps** (> 500 MB) - Performance testing
3. **Complex dependencies** - Nested object resolution
4. **Edge cases** - Missing files, invalid structures

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow the coding standards outlined above
4. Commit your changes with clear messages
5. Push to your branch
6. Open a Pull Request with a detailed description

### Code Review Process

- All PRs require review before merging
- Ensure TypeScript and Rust code compiles without errors
- Test changes thoroughly with various OMSI 2 maps
- Update documentation if adding new features

## Roadmap

- [ ] Add support for additional OMSI 2 assets
- [ ] Implement bundle validation/integrity checking
- [ ] Add batch processing for multiple maps
- [ ] Create automated tests
- [ ] Improve performance for very large maps
- [ ] Add Linux and macOS support

## Disclaimer

This tool is provided "as is" without warranty of any kind. Always backup your files before using this tool. The author is not responsible for any loss of data or damages that may occur from using this software. Always check the bundled files before distribution.

## Support

If you encounter any issues or have questions:

- **Discord**: kubiczeek
- **GitHub Issues**: [Create an issue](https://github.com/yourusername/omsimapbundler/issues)

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built for the OMSI 2 mapping community
- Thanks to all contributors and testers
- Powered by Tauri and Rust

---

**Made with ❤️ for OMSI 2 map creators**
