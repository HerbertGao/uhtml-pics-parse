use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Context, Result};
use thiserror::Error;
use image::io::Reader as ImageReader;
use std::io::Cursor;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("文件读取失败")]
    FileRead(#[from] std::io::Error),
    #[error("Base64编码失败")]
    Base64Encode(#[from] base64::DecodeError),
    #[error("路径错误: {0}")]
    PathError(String),
    #[error("图片解析失败")]
    ImageParse(String),
}

#[derive(Debug)]
pub struct ImageInfo {
    #[allow(dead_code)]
    pub index: usize,
    pub image_type: String,
    pub size: usize,
    pub data: Vec<u8>,
    #[allow(dead_code)]
    pub position: usize,
    #[allow(dead_code)]
    pub end_position: usize,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct ExtractionResult {
    pub source_file: PathBuf,
    pub output_directory: PathBuf,
    pub total_images: usize,
    pub saved_images: usize,
    pub error: Option<String>,
}

pub struct UHTMLImageExtractor;

impl UHTMLImageExtractor {
    pub fn new() -> Self {
        Self
    }

    /// 从单个UHTML文件提取所有图片
    pub fn extract_images_from_file(
        &self,
        uhtml_path: &Path,
        output_dir: Option<&PathBuf>,
        output_all: bool,
    ) -> Result<ExtractionResult> {
        if !uhtml_path.exists() {
            return Err(ExtractionError::PathError(format!("文件不存在: {}", uhtml_path.display())).into());
        }

        // 确定输出目录
        let output_dir = match output_dir {
            Some(dir) => dir.clone(),
            None => {
                let stem = uhtml_path.file_stem()
                    .context("无法获取文件名")?
                    .to_string_lossy();
                uhtml_path.parent()
                    .context("无法获取父目录")?
                    .join(stem.as_ref())
            }
        };

        // 创建输出目录
        fs::create_dir_all(&output_dir)
            .with_context(|| format!("创建输出目录失败: {}", output_dir.display()))?;

        // 读取文件数据
        let data = fs::read(uhtml_path)
            .with_context(|| format!("读取文件失败: {}", uhtml_path.display()))?;

        // 提取图片
        let images = self.extract_images(&data, output_all)?;

        // 保存图片
        let mut saved_count = 0;
        for (i, image) in images.iter().enumerate() {
            match self.save_image(&output_dir, i, image) {
                Ok(path) => {
                    println!("保存图片: {} ({}x{}, {} bytes)", 
                             path.display(), image.width, image.height, image.size);
                    saved_count += 1;
                }
                Err(e) => {
                    eprintln!("保存图片 {} 失败: {}", i, e);
                }
            }
        }

        Ok(ExtractionResult {
            source_file: uhtml_path.to_path_buf(),
            output_directory: output_dir,
            total_images: images.len(),
            saved_images: saved_count,
            error: None,
        })
    }

    /// 批量提取目录下所有UHTML文件中的图片
    pub fn extract_images_from_directory(
        &self,
        directory: &Path,
        recursive: bool,
        output_all: bool,
    ) -> Result<Vec<ExtractionResult>> {
        if !directory.exists() || !directory.is_dir() {
            return Err(ExtractionError::PathError(format!("目录不存在或不是有效目录: {}", directory.display())).into());
        }

        // 查找所有UHTML文件
        let uhtml_files: Vec<PathBuf> = if recursive {
            WalkDir::new(directory)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "uhtml")
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            fs::read_dir(directory)?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
                .filter(|e| {
                    e.path()
                        .extension()
                        .map_or(false, |ext| ext == "uhtml")
                })
                .map(|e| e.path())
                .collect()
        };

        if uhtml_files.is_empty() {
            println!("在目录 {} 中未找到UHTML文件", directory.display());
            return Ok(vec![]);
        }

        println!("找到 {} 个UHTML文件，开始提取...", uhtml_files.len());

        let mut results = Vec::new();
        for uhtml_file in uhtml_files {
            println!("\n处理文件: {}", uhtml_file.display());
            
            match self.extract_images_from_file(&uhtml_file, None, output_all) {
                Ok(result) => {
                    println!("✓ 完成: 提取了 {} 张图片", result.saved_images);
                    results.push(result);
                }
                Err(e) => {
                    let error_result = ExtractionResult {
                        source_file: uhtml_file.clone(),
                        output_directory: PathBuf::new(),
                        total_images: 0,
                        saved_images: 0,
                        error: Some(e.to_string()),
                    };
                    println!("✗ 失败: {}", e);
                    results.push(error_result);
                }
            }
        }

        Ok(results)
    }

    /// 从UHTML数据中提取所有图片
    fn extract_images(&self, data: &[u8], output_all: bool) -> Result<Vec<ImageInfo>> {
        let mut images = Vec::new();

        // 图片格式签名和结束标记
        let image_signatures = [
            (&b"\xFF\xD8\xFF"[..], "jpeg", &b"\xFF\xD9"[..]),          // JPEG
            (&b"\x89PNG\r\n\x1a\n"[..], "png", &b"IEND\xaeB`\x82"[..]), // PNG
            (&b"GIF87a"[..], "gif", &b"\x00\x3B"[..]),                  // GIF87a
            (&b"GIF89a"[..], "gif", &b"\x00\x3B"[..]),                  // GIF89a
        ];

        let mut all_positions = Vec::new();

        // 查找所有图片位置
        for (header, img_type, footer) in &image_signatures {
            let mut pos = 0;
            while let Some(found_pos) = self.find_bytes(data, header, pos) {
                all_positions.push((found_pos, *img_type, *header, *footer));
                pos = found_pos + 1;
            }
        }

        // 按位置排序
        all_positions.sort_by_key(|&(pos, _, _, _)| pos);

        // 提取每张图片
        for (i, &(pos, img_type, _header, footer)) in all_positions.iter().enumerate() {
            if let Ok(image_data) = self.extract_single_image(data, pos, img_type, footer, &all_positions, i) {
                // 验证图片数据（最小大小检查）
                if image_data.len() >= 100 {  // 图片至少100字节
                    // 获取图片尺寸
                    let (width, height) = self.get_image_dimensions(&image_data)?;
                    
                    // 如果不是输出全部，过滤小于20x20的图片
                    if !output_all && (width < 20 || height < 20) {
                        println!("跳过小图片: {}x{} 像素", width, height);
                        continue;
                    }
                    
                    let data_size = image_data.len();
                    let image = ImageInfo {
                        index: images.len(),
                        image_type: format!("image/{}", img_type),
                        size: data_size,
                        data: image_data,
                        position: pos,
                        end_position: pos + data_size,
                        width,
                        height,
                    };
                    images.push(image);
                }
            }
        }

        Ok(images)
    }

    /// 提取单张图片
    fn extract_single_image(
        &self,
        data: &[u8],
        start_pos: usize,
        img_type: &str,
        footer: &[u8],
        all_positions: &[(usize, &str, &[u8], &[u8])],
        current_index: usize,
    ) -> Result<Vec<u8>> {
        // 查找图片结束位置
        let end_pos = if let Some(footer_pos) = self.find_bytes(data, footer, start_pos) {
            match img_type {
                "jpeg" => footer_pos + 2,  // JPEG结束标记长度为2
                "png" => footer_pos + 8,   // PNG IEND块长度为8
                "gif" => footer_pos + 2,   // GIF结束标记长度为2
                _ => footer_pos + footer.len(),
            }
        } else if current_index + 1 < all_positions.len() {
            // 使用下一个图片的开始位置
            all_positions[current_index + 1].0
        } else {
            // 如果是最后一张图片，使用合理的最大大小（1MB）
            std::cmp::min(start_pos + 1024 * 1024, data.len())
        };

        if end_pos > start_pos && end_pos <= data.len() {
            Ok(data[start_pos..end_pos].to_vec())
        } else {
            Err(ExtractionError::ImageParse("无效的图片数据范围".to_string()).into())
        }
    }

    /// 保存单张图片
    fn save_image(&self, output_dir: &Path, index: usize, image: &ImageInfo) -> Result<PathBuf> {
        let extension = self.get_file_extension(&image.image_type);
        let filename = format!("image_{:03}{}", index, extension);
        let image_path = output_dir.join(filename);

        fs::write(&image_path, &image.data)
            .with_context(|| format!("写入图片文件失败: {}", image_path.display()))?;

        Ok(image_path)
    }

    /// 在字节数组中查找子序列
    fn find_bytes(&self, data: &[u8], pattern: &[u8], start: usize) -> Option<usize> {
        if start >= data.len() {
            return None;
        }

        data[start..]
            .windows(pattern.len())
            .position(|window| window == pattern)
            .map(|pos| start + pos)
    }

    /// 获取图片尺寸
    fn get_image_dimensions(&self, image_data: &[u8]) -> Result<(u32, u32)> {
        let cursor = Cursor::new(image_data);
        match ImageReader::new(cursor).with_guessed_format() {
            Ok(reader) => {
                match reader.into_dimensions() {
                    Ok((width, height)) => Ok((width, height)),
                    Err(_) => {
                        // 如果无法解析尺寸，使用简单的方法解析常见格式
                        self.parse_image_dimensions_manually(image_data)
                    }
                }
            }
            Err(_) => {
                // 如果格式检测失败，使用手动解析
                self.parse_image_dimensions_manually(image_data)
            }
        }
    }

    /// 手动解析图片尺寸（简单实现）
    fn parse_image_dimensions_manually(&self, data: &[u8]) -> Result<(u32, u32)> {
        if data.len() < 10 {
            return Ok((0, 0));
        }

        // JPEG解析
        if data.starts_with(b"\xFF\xD8\xFF") {
            if let Some((w, h)) = self.parse_jpeg_dimensions(data) {
                return Ok((w, h));
            }
        }

        // PNG解析
        if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            if let Some((w, h)) = self.parse_png_dimensions(data) {
                return Ok((w, h));
            }
        }

        // GIF解析
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            if let Some((w, h)) = self.parse_gif_dimensions(data) {
                return Ok((w, h));
            }
        }

        // 如果无法解析，返回默认值
        Ok((100, 100))  // 假设是有效图片
    }

    /// 解析JPEG尺寸
    fn parse_jpeg_dimensions(&self, data: &[u8]) -> Option<(u32, u32)> {
        let mut pos = 2;
        while pos + 4 < data.len() {
            if data[pos] == 0xFF {
                let marker = data[pos + 1];
                if marker >= 0xC0 && marker <= 0xC3 {
                    if pos + 9 < data.len() {
                        let height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
                        let width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;
                        return Some((width, height));
                    }
                } else {
                    let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
                    pos += length as usize + 2;
                }
            } else {
                pos += 1;
            }
        }
        None
    }

    /// 解析PNG尺寸
    fn parse_png_dimensions(&self, data: &[u8]) -> Option<(u32, u32)> {
        if data.len() >= 24 {
            let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
            let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            Some((width, height))
        } else {
            None
        }
    }

    /// 解析GIF尺寸
    fn parse_gif_dimensions(&self, data: &[u8]) -> Option<(u32, u32)> {
        if data.len() >= 10 {
            let width = u16::from_le_bytes([data[6], data[7]]) as u32;
            let height = u16::from_le_bytes([data[8], data[9]]) as u32;
            Some((width, height))
        } else {
            None
        }
    }

    /// 根据MIME类型获取文件扩展名
    fn get_file_extension(&self, mime_type: &str) -> &'static str {
        match mime_type {
            "image/jpeg" => ".jpg",
            "image/png" => ".png",
            "image/gif" => ".gif",
            "image/webp" => ".webp",
            "image/bmp" => ".bmp",
            _ => ".img",
        }
    }
}

impl Default for UHTMLImageExtractor {
    fn default() -> Self {
        Self::new()
    }
}