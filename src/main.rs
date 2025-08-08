use clap::{Arg, Command};
use std::path::PathBuf;

mod extractor;
use extractor::UHTMLImageExtractor;

fn main() {
    let matches = Command::new("uhtml-pics-parse")
        .version("0.1.0")
        .author("Claude")
        .about("UHTML图片批量提取工具 (Rust版本)")
        .arg(
            Arg::new("path")
                .help("UHTML文件路径或包含UHTML文件的目录路径")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("输出目录（可选，默认使用与文件同名的目录）"),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .action(clap::ArgAction::SetTrue)
                .help("递归搜索子目录中的UHTML文件"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("详细输出"),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(clap::ArgAction::SetTrue)
                .help("输出全部图片（默认过滤小于20x20像素的图片）"),
        )
        .get_matches();

    let path = PathBuf::from(matches.get_one::<String>("path").unwrap());
    let output = matches.get_one::<String>("output").map(PathBuf::from);
    let recursive = matches.get_flag("recursive");
    let verbose = matches.get_flag("verbose");
    let output_all = matches.get_flag("all");

    let extractor = UHTMLImageExtractor::new();

    match run_extraction(extractor, path, output, recursive, verbose, output_all) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("执行失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_extraction(
    extractor: UHTMLImageExtractor,
    path: PathBuf,
    output: Option<PathBuf>,
    recursive: bool,
    verbose: bool,
    output_all: bool,
) -> anyhow::Result<()> {
    if path.is_file() {
        // 处理单个文件
        if !path.extension().map_or(false, |ext| ext == "uhtml") {
            anyhow::bail!("错误: 不支持的文件类型 {:?}", path.extension());
        }

        println!("提取单个文件: {}", path.display());
        let result = extractor.extract_images_from_file(&path, output.as_ref(), output_all)?;

        println!("\n=== 提取完成 ===");
        println!("源文件: {}", result.source_file.display());
        println!("输出目录: {}", result.output_directory.display());
        println!("找到图片: {} 张", result.total_images);
        println!("成功保存: {} 张", result.saved_images);

    } else if path.is_dir() {
        // 处理目录
        println!("批量提取目录: {}", path.display());
        println!("递归搜索: {}", if recursive { "是" } else { "否" });

        let results = extractor.extract_images_from_directory(&path, recursive, output_all)?;

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
