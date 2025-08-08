#!/bin/bash

# UHTML图片批量提取工具发布脚本

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 获取当前版本
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
echo -e "${BLUE}📋 当前版本: ${CURRENT_VERSION}${NC}"

# 检查是否有未提交的更改
if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}⚠️  检测到未提交的更改，请先提交所有更改${NC}"
    git status --short
    exit 1
fi

# 获取新版本号
echo -e "${BLUE}请输入新版本号 (例如: 0.2.0):${NC}"
read -r NEW_VERSION

if [[ ! $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}❌ 版本号格式错误，请使用语义化版本格式 (例如: 0.2.0)${NC}"
    exit 1
fi

# 检查版本号是否与当前版本相同
if [[ "$NEW_VERSION" == "$CURRENT_VERSION" ]]; then
    echo -e "${RED}❌ 新版本号不能与当前版本相同${NC}"
    echo -e "${YELLOW}当前版本: $CURRENT_VERSION${NC}"
    exit 1
fi

# 更新 Cargo.toml 中的版本号
echo -e "${BLUE}📝 更新 Cargo.toml 版本号...${NC}"
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# 构建项目
echo -e "${BLUE}🔨 构建项目...${NC}"
cargo build --release

# 运行测试
echo -e "${BLUE}🧪 运行测试...${NC}"
cargo test

# 提交更改
echo -e "${BLUE}📝 提交更改...${NC}"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $NEW_VERSION"

# 创建标签
echo -e "${BLUE}🏷️  创建标签 v$NEW_VERSION...${NC}"
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"

# 推送更改和标签
echo -e "${BLUE}🚀 推送更改和标签...${NC}"
git push origin master
git push origin "v$NEW_VERSION"

echo -e "${GREEN}✅ 发布完成！${NC}"
echo -e "${GREEN}📦 版本 v$NEW_VERSION 已创建并推送到 GitHub${NC}"
echo -e "${GREEN}🔗 GitHub Actions 将自动构建并发布二进制文件${NC}"
