# UHTML图片批量提取工具 (Rust版本)

这是一个使用Rust编写的高性能UHTML图片提取工具，专门用于从UC浏览器7.9的uhtml文件中批量提取所有图片。

## 功能特性

- 🚀 **高性能**: 使用Rust编写，提供极佳的性能和内存安全
- 🖼️ **多格式支持**: 支持JPEG、PNG、GIF等常见图片格式
- 📁 **批量处理**: 支持单文件和目录批量处理
- 🔄 **递归搜索**: 可选择递归搜索子目录中的UHTML文件
- 📊 **详细统计**: 提供详细的提取进度和统计信息
- 🎯 **智能命名**: 图片按 `image_000.jpg` 格式自动命名
- 📂 **自动目录**: 自动创建与UHTML文件同名的目录存放图片
- 🔍 **尺寸过滤**: 默认过滤小于20x20像素的小图片，可选输出全部
- 📏 **尺寸显示**: 显示每张图片的宽高和文件大小

## 安装

确保您的系统已安装Rust编译环境：

```bash
# 安装Rust (如果还未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone <repository-url>
cd uhtml-pics-parse

# 编译项目
cargo build --release
```

## 使用方法

### 基本用法

```bash
# 编译并运行
cargo run --release -- <路径> [选项]

# 或使用编译后的二进制文件
./target/release/uhtml-pics-parse <路径> [选项]
```

### 命令行选项

```text
USAGE:
    uhtml-pics-parse [OPTIONS] <PATH>

ARGS:
    <PATH>    UHTML文件路径或包含UHTML文件的目录路径

OPTIONS:
    -h, --help                 显示帮助信息
    -o, --output <OUTPUT>      输出目录（可选，默认使用与文件同名的目录）
    -r, --recursive            递归搜索子目录中的UHTML文件
    -v, --verbose              详细输出
    -a, --all                  输出全部图片（默认过滤小于20x20像素的图片）
    -V, --version              显示版本信息
```

### 使用示例

#### 1. 提取单个UHTML文件中的图片

```bash
cargo run --release -- test.uhtml
```

输出结果：

```text
提取单个文件: test.uhtml
跳过小图片: 16x16 像素
保存图片: test/image_000.jpg (469x351, 41971 bytes)
保存图片: test/image_001.jpg (469x319, 40635 bytes)
保存图片: test/image_002.jpg (469x351, 50665 bytes)
...

=== 提取完成 ===
源文件: test.uhtml
输出目录: test
找到图片: 7 张
成功保存: 7 张
```

#### 2. 批量处理目录中的所有UHTML文件

```bash
cargo run --release -- /path/to/uhtml/files/ -v
```

#### 3. 递归搜索子目录

```bash
cargo run --release -- /path/to/uhtml/files/ -r -v
```

#### 4. 输出全部图片（包括小图片）

```bash
cargo run --release -- test.uhtml -a -v
```

#### 5. 指定自定义输出目录

```bash
cargo run --release -- test.uhtml -o /custom/output/path
```

## 项目结构

```text
uhtml-pics-parse/
├── Cargo.toml          # 项目配置和依赖
├── README.md          # 项目说明文档
└── src/
    ├── main.rs        # 主程序入口和命令行处理
    └── extractor.rs   # 核心图片提取逻辑
```

## 技术实现

### 核心算法

1. **二进制扫描**: 通过查找图片文件头签名定位图片数据
   - JPEG: `FF D8 FF`
   - PNG: `89 50 4E 47 0D 0A 1A 0A`
   - GIF: `GIF87a` 或 `GIF89a`

2. **边界检测**: 通过文件尾标记或下一个图片位置确定图片数据边界

3. **数据验证**: 最小尺寸验证确保提取的数据是有效图片

### 依赖库

- `clap`: 命令行参数解析
- `walkdir`: 目录遍历和递归搜索
- `anyhow`: 错误处理
- `thiserror`: 自定义错误类型
- `base64`: Base64编码处理

## 性能特性

- **零拷贝**: 使用引用和切片避免不必要的内存复制
- **内存安全**: Rust的所有权系统确保内存安全
- **并发友好**: 设计支持未来的并发处理扩展
- **错误处理**: 完善的错误处理和恢复机制

## 兼容性

- 支持所有主流操作系统 (Windows, macOS, Linux)
- 兼容UC浏览器7.9版本的UHTML文件格式
- 支持各种图片格式的混合文件

## 开发

### 构建

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 贡献

欢迎提交Issue和Pull Request来改进这个项目！

## 许可证

本项目采用MIT许可证。详见LICENSE文件。
