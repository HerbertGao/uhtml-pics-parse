use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod extractor;
mod updater;

use extractor::UHTMLImageExtractor;
use updater::Updater;

#[derive(Parser)]
#[command(name = "uhtml-pics-parse")]
#[command(about = "UHTML图片批量提取工具 (Rust版本)")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 提取UHTML文件中的图片
    Extract {
        /// UHTML文件路径或包含UHTML文件的目录路径
        path: PathBuf,
        
        /// 输出目录（可选，默认使用与文件同名的目录）
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// 递归搜索子目录中的UHTML文件
        #[arg(short, long)]
        recursive: bool,
        
        /// 详细输出
        #[arg(short, long)]
        verbose: bool,
        
        /// 输出全部图片（默认过滤小于100x100像素的图片）
        #[arg(short, long)]
        all: bool,
        
        /// 最小图片尺寸 (格式: 宽x高，例如: 200x150)
        #[arg(long, value_name = "SIZE")]
        min_size: Option<String>,
    },
    
    /// 检查并更新到最新版本
    Update,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Extract { path, output, recursive, verbose, all, min_size } => {
            run_extraction(path, output.as_ref(), *recursive, *verbose, *all, min_size.as_deref())?;
        }
        Commands::Update => {
            let updater = Updater::new()?;
            updater.update().await?;
        }
    }

    Ok(())
}

/// 解析尺寸字符串 (格式: 宽x高，例如: "200x150")
fn parse_size_string(size_str: &str) -> anyhow::Result<Option<(u32, u32)>> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("尺寸格式错误，请使用 '宽x高' 格式，例如: 200x150"));
    }
    
    let width = parts[0].parse::<u32>()
        .map_err(|_| anyhow::anyhow!("宽度必须是正整数"))?;
    let height = parts[1].parse::<u32>()
        .map_err(|_| anyhow::anyhow!("高度必须是正整数"))?;
    
    if width == 0 || height == 0 {
        return Err(anyhow::anyhow!("宽度和高度必须大于0"));
    }
    
    Ok(Some((width, height)))
}

fn run_extraction(
    path: &PathBuf,
    output: Option<&PathBuf>,
    recursive: bool,
    verbose: bool,
    output_all: bool,
    min_size: Option<&str>,
) -> anyhow::Result<()> {
    let extractor = UHTMLImageExtractor::new();
    
    // 解析最小尺寸参数
    let parsed_min_size = if let Some(size_str) = min_size {
        parse_size_string(size_str)?
    } else {
        None
    };

    if path.is_file() {
        // 处理单个文件
        if !path.extension().map_or(false, |ext| ext == "uhtml") {
            anyhow::bail!("错误: 不支持的文件类型 {:?}", path.extension());
        }

        println!("提取单个文件: {}", path.display());
        let result = extractor.extract_images_from_file(path, output, output_all, parsed_min_size)?;

        println!("\n=== 提取完成 ===");
        println!("源文件: {}", result.source_file.display());
        println!("输出目录: {}", result.output_directory.display());
        println!("找到图片: {} 张", result.total_images);
        println!("成功保存: {} 张", result.saved_images);

    } else if path.is_dir() {
        // 处理目录
        println!("批量提取目录: {}", path.display());
        println!("递归搜索: {}", if recursive { "是" } else { "否" });

        let results = extractor.extract_images_from_directory(path, recursive, output_all, parsed_min_size)?;

        // 统计结果
        let total_files = results.len();
        let successful_files = results.iter().filter(|r| r.error.is_none()).count();
        let total_images: usize = results.iter().map(|r| r.saved_images).sum();

        println!("\n=== 批量提取完成 ===");
        println!("处理文件: {} 个", total_files);
        println!("成功文件: {} 个", successful_files);
        println!("提取图片总数: {} 张", total_images);

        if verbose {
            println!("\n=== 详细结果 ===");
            for result in &results {
                if let Some(error) = &result.error {
                    println!("✗ {}: {}", result.source_file.display(), error);
                } else {
                    println!("✓ {}: {} 张图片", result.source_file.display(), result.saved_images);
                }
            }
        }

    } else {
        anyhow::bail!("错误: 路径不存在或无效 {}", path.display());
    }

    Ok(())
}
