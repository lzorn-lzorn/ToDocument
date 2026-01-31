mod file_parser;
use clap::Parser;
use file_parser::InputFileType;
use file_parser::OutputFileType;
use once_cell::sync::Lazy;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/*
 * todoc --file
 */

pub static WORKSPACE: Lazy<Mutex<String>> = Lazy::new(|| {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| String::from("."));
    Mutex::new(cwd)
});

/// 简单的命令行参数解析示例
#[derive(Parser, Debug)]
#[command(author = "LiZhuoran", version = "0.1", about = "", long_about = "")]
pub struct Args {
    #[arg(long, help = "指定要处理的文件路径")]
    pub files: Vec<Option<String>>,

    #[arg(short, help = "处理当前目录下的所有文件")]
    pub all: bool,

    #[arg(short, long, help = "是否递归处理子目录")]
    pub recursive: bool,
}

// fn which_file(file_name: &String) -> file_parser::InputFileType {

//     return file_parser::InputFileType::from_str(file_name);
// }

/// 在指定路径创建一个 Markdown 文件
fn create_md_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all("# 新的 Markdown 文件\n".as_bytes())?;
    Ok(())
}

fn cmd_parser() {
    let args = Args::parse();
    let is_empty = args.files.is_empty();

    if !is_empty {
        for file_name in args.files.iter() {
            let file_path = Path::new(file_name.as_ref().unwrap());
            println!("处理文件: {}", file_path.display());
            if !file_path.exists() {
                eprintln!("文件不存在: {}", file_path.display());
                continue;
            }
            let file_type = file_path
                .extension()
                .and_then(|e| e.to_str())
                .and_then(|s| file_parser::InputFileType::from_str(s));

            let parser = file_parser::create_file_parser(&file_type);
            let mid = parser.parse(&File::open(file_path).unwrap());
            let _ = file_parser::MarkdownFormatter{}.format(mid);
            match file_type {
                Some(ft) => println!("文件类型: {}", ft.to_str().unwrap_or("unknown")),
                None => println!("文件类型: unknown"),
            }
            // create_file_parser
        }
    }
}
fn main() {
    // 获取程序名
    let exe_name = env::args().next().unwrap_or_default();
    let exe_name = exe_name.to_lowercase();
    let is_todoc = exe_name.contains("todoc") || exe_name.contains("todocument");

    if !is_todoc {
        eprint!("当前程序名不正确，请使用 todoc 或 todocument 作为程序名运行");
        return;
    }

    cmd_parser();
    return;
}
