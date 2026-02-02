# Yi M1 to Olympus (yi2olympus)

A Tauri application to disguise Yi M1 DNG photos as Olympus PEN-F or E-M1 photos using ExifTool.

## Features

- **Drag & Drop Interface**: Easily process multiple files.
- **Batch Processing**: Convert multiple DNGs at once.
- **Preset System**: Built-in presets for Olympus PEN-F and E-M1.
- **bundled ExifTool**: No external dependencies required.
- **Fluent UI**: Modern, dark-themed interface.

## Development

### Prerequisites

- Node.js & pnpm
- Rust & Cargo

### Setup

1. Install dependencies:
   ```bash
   pnpm install
   ```

2. Run in development mode:
   ```bash
   pnpm tauri dev
   ```

## Build

To build the application for production:

```bash
pnpm tauri build
```

The binaries will be located in `src-tauri/target/release/bundle/`.

## Credits

- [ExifTool](https://exiftool.org/) by Phil Harvey
