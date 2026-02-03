# YiModelMasking

将小蚁M1拍摄的DNG原档伪装机型为奥林巴斯PEN-F等相同CMOS型号以兼容DxO PureRaw等后期软件

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## Features

- **拖放交互**: 轻松处理文件。
- **批量处理**: 一次性转换你的所有DNG。
- **内置预设**: 内建所有相同传感器相机的预设，无需查询和手动编写。
- **集成exifTool**: 无需额外下载依赖，完全离线可用。
- **轻量占用**: Svelte+Tauri使用Rust与Webview2驱动，体积与内存均可称优秀。

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
