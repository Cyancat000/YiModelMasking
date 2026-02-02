# 小蚁 M1 伪装 PEN-F 开发任务清单 (Project Blueprint)

## 1. 资源与二进制管理 (Resource & Binaries)
* [ ] **资源目录结构确认**：
    * 将 `exiftool.exe` 与 `exiftool_files` 文件夹放置在 `src-tauri/resources/` 目录下。
    * **注意**：不要重命名为 sidecar 格式，直接保持原始名称。
* [ ] **Tauri 配置更新**：
    * 在 `tauri.conf.json` 的 `bundle -> resources` 中添加该路径，确保打包时文件夹结构不被破坏。
    * 开启 `allowlist` 中的 `shell` 权限，允许执行任意参数。

## 2. Rust 后端桥接 (Backend Bridge)
* [ ] **路径解析逻辑**：
    * 实现一个名为 `run_exiftool` 的 `command`。
    * 利用 `tauri::AppHandle` 的 `path_resolver().resolve_resource()` 动态获取 ExifTool 的绝对路径。
* [ ] **Windows 进程优化**：
    * 使用 `std::process::Command` 时，配置 `creation_flags(0x08000000)` 以隐藏执行时的 CMD 黑窗口。
* [ ] **工作目录 (CWD) 设置**：
    * **核心细节**：将 `Command` 的 `current_dir` 设置为 `exiftool.exe` 所在的父目录，否则它可能找不到同级的 `exiftool_files`。

## 3. 前端预设系统 (Frontend Preset System)
* [ ] **数据结构定义**：
    * 定义 `Preset` 接口：包含 `id`, `name`, `make`, `model`, `software` 等字段。
* [ ] **内置预设库**：
    * 硬编码初始预设：`PEN-F` (奥林巴斯), `E-M1` 等。
    * **PEN-F 参数点**：`Make` 必须为 `OLYMPUS CORPORATION`，`Model` 为 `PEN-F`，同时考虑写入 `ImageDescription`。
* [ ] **自定义预设持久化**：
    * 使用 `localStorage` 或 `tauri-plugin-store` 实现预设的增删改查。

## 4. 批量处理逻辑 (Batch Processing)
* [ ] **文件监听与过滤**：
    * 监听 Tauri 原生的 `tauri://file-drop` 事件。
    * **正则过滤**：仅保留 `.dng` 后缀的文件路径。
* [ ] **任务调度**：
    * **策略选择**：不要一张张调命令。建议将所有选中的路径拼接到一个 `args` 数组中，一次性传给 ExifTool，利用其原生批量处理能力提高效率。
* [ ] **参数封装**：
    * 默认添加 `-overwrite_original` (不产生备份文件) 和 `-m` (忽略次要错误) 参数。

## 5. UI/UX 设计 (Fluent UI Style)
* [ ] **视觉风格设定**：
    * **背景**：使用半透明云母 (Mica) 效果或玻璃拟态 (Glassmorphism)。
    * **圆角**：统一使用 `12px` 或以上的大圆角。
    * **组件库建议**：使用 `Tailwind CSS` 配合自定义的 Fluent 变量。
* [ ] **交互流程**：
    * **状态 1 (空闲)**：大面积的拖拽区域，居中显示“点击或拖入 DNG 照片”。
    * **状态 2 (已选择)**：以列表或网格展示文件流，显示文件大小和当前机型。
    * **状态 3 (处理中)**：显示进度条或 Loading 动画，禁用修改按钮。
* [ ] **反馈机制**：
    * 完成后的 Toast 提示，显示“X 张照片已修改成功”。

## 6. 健壮性与调试 (Robustness)
* [ ] **错误日志捕获**：
    * 将 ExifTool 的 `stderr` 返回给前端，如果转换失败，允许用户查看具体的命令行报错。
* [ ] **权限校验**：
    * 检测用户拖入的文件是否有写入权限（只读文件处理）。
* [ ] **并发冲突**：
    * 防止在处理过程中用户再次触发转换按钮。