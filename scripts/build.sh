#!/bin/bash

# UHTML图片批量提取工具构建脚本

set -e

echo "🚀 开始构建 uhtml-pics-parse..."

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 cargo，请先安装 Rust"
    exit 1
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean

# 检查代码
echo "🔍 检查代码..."
cargo check

# 运行测试
echo "🧪 运行测试..."
cargo test

# 构建发布版本
echo "📦 构建发布版本..."
cargo build --release

echo "✅ 构建完成！"
echo "📁 可执行文件位置: target/release/uhtml-pics-parse"

# 显示文件大小
if [ -f "target/release/uhtml-pics-parse" ]; then
    size=$(ls -lh target/release/uhtml-pics-parse | awk '{print $5}')
    echo "📊 文件大小: $size"
fi
