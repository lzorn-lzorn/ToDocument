mod file_parser;
use clap::Parser;
use file_parser::{create_file_parser, InputFileType, MarkdownFormatter};
use once_cell::sync::Lazy;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/*
 * todoc --files code.lua
 */

pub static WORKSPACE: Lazy<Mutex<String>> = Lazy::new(|| {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| String::from("."));
    Mutex::new(cwd)
});

/// 命令行参数定义
#[derive(Parser, Debug)]
#[command(author = "LiZhuoran", version = "0.1", about = "Doc Generator", long_about = None)]
pub struct Args {
    #[arg(long, num_args = 1.., help = "指定要处理的文件路径")]
    pub files: Vec<String>,

    #[arg(short, long, help = "处理当前目录下的所有文件")]
    pub all: bool,

    #[arg(short, long, help = "是否递归处理子目录")]
    pub recursive: bool,
}

/// 保存 Markdown 文件
fn save_markdown_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// 处理单个文件
fn process_single_file(path: &Path) {
    println!("-----------------------------------------------------");
    println!("正在处理文件: {}", path.display());

    if !path.exists() {
        eprintln!("错误: 文件不存在: {}", path.display());
        return;
    }

    // 1. 推断文件类型
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    let file_type = InputFileType::from_str(extension);
    
    // 安全地获取类型名称用于打印
    let type_name = file_type.as_ref()
        .and_then(|t| t.to_str())
        .unwrap_or("Unknown");
    println!("文件类型: {:?}", type_name);

    // 检查是否是不支持的类型
    if file_type.is_none() || matches!(file_type, Some(InputFileType::None)) {
        println!("跳过不支持的文件类型: {}", path.display());
        return;
    }

    // 2. 创建解析器并解析 Is it a parser? Yes!
    // create_file_parser 接受 &Option<InputFileType>
    let parser = create_file_parser(&file_type);
    
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件: {}", e);
            return;
        }
    };

    let doc_blocks = parser.parse(&file);
    if doc_blocks.is_empty() {
        println!("未发现文档块，跳过生成.");
        return;
    }
    println!("发现 {} 个文档块.", doc_blocks.len());

    // 3. 格式化为 Markdown
    let formatter = MarkdownFormatter {};
    match formatter.format(doc_blocks) {
        Ok(markdown_content) => {
            // 4. 生成输出路径 (filename.md)
            let mut out_path = PathBuf::from(path);
            out_path.set_extension("md");
            
            // 5. 写入文件
            match save_markdown_file(&out_path, &markdown_content) {
                Ok(_) => println!("成功生成文档: {}", out_path.display()),
                Err(e) => eprintln!("写入文件失败: {}", e),
            }
        },
        Err(e) => eprintln!("格式化 Markdown 失败: {}", e),
    }
}

/// 递归遍历目录处理文件
fn process_directory(dir: &Path, recursive: bool) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if recursive {
                        process_directory(&path, recursive);
                    }
                } else {
                    // 简单的过滤逻辑，只处理源码文件
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if InputFileType::from_str(ext).is_some() {
                           process_single_file(&path);
                        }
                    }
                }
            }
        }
    }
}

fn cmd_parser() {
    let args = Args::parse();

    // 1. 如果指定了具体文件，优先处理
    if !args.files.is_empty() {
        for file_name in &args.files {
            let path = Path::new(file_name);
            process_single_file(path);
        }
    } 
    // 2. 否则如果指定了 --all，遍历目录
    else if args.all {
        let current_dir = env::current_dir().unwrap_or(PathBuf::from("."));
        println!("正在扫描目录: {}", current_dir.display());
        process_directory(&current_dir, args.recursive);
    } 
    // 3. 无参数提示
    else {
        println!("未指定输入文件。使用 --files <path> 或 --all 运行。");
        println!("尝试运行 'todoc --help' 查看更多选项。");
    }
}

fn main() {
    // 简化的入口检查，不再强制检查程序名，方便 cargo run 调试
    let args: Vec<String> = env::args().collect();
    if let Some(exe) = args.first() {
        // 可以在这里做日志
        println!("Running: {}", exe);
    }

    cmd_parser();
    
    println!("-----------------------------------------------------");
    println!("任务完成.");
}
