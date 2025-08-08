# UHTML图片批量提取工具 (Rust版本)

一个高效的UHTML文件图片提取工具，支持批量处理和自动更新。

## 功能特性

- 🖼️ **图片提取**：从UHTML文件中提取JPEG、PNG、GIF等格式的图片
- 📁 **批量处理**：支持单个文件或整个目录的批量处理
- 🔍 **递归搜索**：可递归搜索子目录中的UHTML文件
- 🎯 **智能过滤**：自动过滤宽度和高度都小于100x100像素的小图片（可自定义）
- 🔄 **自动更新**：内置版本检查和自动更新功能
- 🖥️ **多平台支持**：支持Windows、macOS、Linux（x86_64和ARM64）

## 安装

### 从GitHub Releases下载

访问 [GitHub Releases](https://github.com/HerbertGao/uhtml-pics-parse/releases) 页面，下载适合您平台的预编译版本。

### 从源码编译

```bash
git clone https://github.com/HerbertGao/uhtml-pics-parse.git
cd uhtml-pics-parse
cargo build --release
```

## 使用方法

### 基本用法

```bash
# 提取单个UHTML文件中的图片
uhtml-pics-parse extract example.uhtml

# 提取目录中所有UHTML文件的图片
uhtml-pics-parse extract /path/to/uhtml/files

# 递归搜索子目录
uhtml-pics-parse extract /path/to/directory --recursive

# 指定输出目录
uhtml-pics-parse extract example.uhtml --output ./output

# 提取所有图片（包括小图片）
uhtml-pics-parse extract example.uhtml --all

# 自定义最小图片尺寸
uhtml-pics-parse extract example.uhtml --min-size 200x150

# 详细输出
uhtml-pics-parse extract /path/to/directory --verbose
```

### 更新程序

```bash
# 检查并更新到最新版本
uhtml-pics-parse update
```

### 获取帮助

```bash
# 查看所有命令
uhtml-pics-parse --help

# 查看提取命令的帮助
uhtml-pics-parse extract --help

# 查看更新命令的帮助
uhtml-pics-parse update --help

# 查看版本信息
uhtml-pics-parse --version
```

## 命令行选项

### Extract 命令

| 选项 | 短选项 | 说明 |
|------|--------|------|
| `--output <OUTPUT>` | `-o` | 指定输出目录（可选） |
| `--recursive` | `-r` | 递归搜索子目录 |
| `--verbose` | `-v` | 详细输出模式 |
| `--all` | `-a` | 提取所有图片（不过滤小图片） |
| `--min-size <SIZE>` | | 最小图片尺寸 (格式: 宽x高，例如: 200x150，只有当宽度和高度都小于指定尺寸时才过滤) |

### Update 命令

无额外选项，执行后会自动检查更新并提示用户确认。

## 输出说明

程序会在指定目录下创建与UHTML文件同名的文件夹，并将提取的图片保存为：

- `image_001.jpg` - JPEG格式图片
- `image_002.png` - PNG格式图片
- `image_003.gif` - GIF格式图片

## 支持的图片格式

- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- WebP (.webp)
- BMP (.bmp)

## 开发

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test
```

### 发布新版本

```bash
# 使用发布脚本
./scripts/release.sh
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v1.0.3
- 调整默认过滤尺寸为 100x100 像素
- 新增自定义最小图片尺寸功能
- 改进错误处理和用户提示

### v1.0.0
- 初始版本发布
- 支持UHTML文件图片提取
- 添加批量处理和递归搜索
- 内置自动更新功能
