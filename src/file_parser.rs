use core::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub enum InputFileType {
    None,
    Lua,
    C,
    Cpp,
    Rust,
    Python,
}
impl InputFileType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lua" => Some(InputFileType::Lua),
            "c" => Some(InputFileType::C),
            "cpp" | "cc" => Some(InputFileType::Cpp),
            "rs" => Some(InputFileType::Rust),
            "py" => Some(InputFileType::Python),
            _ => None,
        }
    }
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            InputFileType::Lua => Some("lua"),
            InputFileType::C => Some("c"),
            InputFileType::Cpp => Some("cpp"),
            InputFileType::Rust => Some("rs"),
            InputFileType::Python => Some("py"),
            InputFileType::None => Some("None"),
            _ => None,
        }
    }
}

pub enum FormulaType {
    Inline,
    Block,
}
pub enum OutputFileType {
    Markdown,
}

/// 中间文档结构（简化）
pub struct Parameter {
    pub name: String,
    pub number: usize,
    pub description: String,
    pub type_name: String,
}

pub enum DescriptionType {
    Text(String),
    Code(InputFileType, String),
    MathFormula(FormulaType, String),
    BulletList(i32, String),
    HTMLLink(String),
}

pub struct Description {
    pub dtype: DescriptionType,
    pub content: String,
}
/**
 * @!all 不会导出
 * @brief 这是一个示例函数      (brief)
 * @param x number 第一个参数  (Parameter: name, type_name, description)
 * @param y number 第二个参数  (Parameter: name, type_name, description)
 * @return number 返回值说明   (Parameter: "", type_name, description)
 * @description
 *     \text text  (DescriptionType.Text)
 *     \code{}     (DescriptionType.Code)
 *     \formula{}  (DescriptionType.MathFormula)
 *     \list       (DescriptionType.BulletList)
 *         - item1
 *         - item2
 *     \html url   (DescriptionType.HTMLLink)
 * function signature (x, y) (signature)
 */
pub struct DocBlock {
    pub signature: String,
    pub brief: String,
    pub note: String,
    pub parameters: Vec<Parameter>,
    pub descriptions: Vec<Description>,
    pub ret_value: Option<Parameter>,
}

/// 解析器 trait：把文件解析成一组 DocBlock（中间结构）
pub trait FileParser {
    fn parse(&self, file: &File) -> Vec<DocBlock>;
}

/// 输出格式化器 trait：把 DocBlock 转为目标格式字符串
pub trait OutputFileFormatter {
    fn format(&self, content: &DocBlock) -> String;
}

pub struct LuaFileParser {}

impl LuaFileParser {
    const ANNOTATION: &'static str = "-- ";

    pub fn is_annotation_line(line: &String) -> bool {
        return line.trim_start().starts_with(Self::ANNOTATION);
    }
}
impl FileParser for LuaFileParser {
    fn parse(&self, file: &File) -> Vec<DocBlock> {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if LuaFileParser::is_annotation_line(&l) {
                        println!("注释行: {}", l);
                    } else {
                        println!("代码行: {}", l);
                    }
                }
                Err(_) => continue,
            }
        }

        return vec![];
    }
}

/// C 文件解析器示例
pub struct CFileParser;
impl FileParser for CFileParser {
    fn parse(&self, _file: &File) -> Vec<DocBlock> {
        vec![]
    }
}

pub struct NoneFileParser;
impl FileParser for NoneFileParser {
    fn parse(&self, _file: &File) -> Vec<DocBlock> {
        vec![]
    }
}
/// 工厂函数：根据输入类型返回实现了 FileParser 的 trait 对象
pub fn create_file_parser(optkind: &Option<InputFileType>) -> Box<dyn FileParser> {
    let kind = optkind.as_ref().unwrap_or(&InputFileType::None);
    match kind {
        InputFileType::Lua => Box::new(LuaFileParser {}),
        InputFileType::C => Box::new(CFileParser {}),
        InputFileType::Cpp => {
            println!("not supported code file = {:?}", kind.to_str());
            Box::new(CFileParser {})
        }
        InputFileType::Rust => {
            println!("not supported code file = {:?}", kind.to_str());
            Box::new(CFileParser {})
        }
        InputFileType::Python => {
            println!("not supported code file = {:?}", kind.to_str());
            Box::new(CFileParser {})
        }
        InputFileType::None => {
            println!("not supported code file = {:?}", kind.to_str());
            Box::new(NoneFileParser {})
        }
    }
}

/// 简单的 Markdown 格式化器示例
pub struct MarkdownFormatter {}
impl MarkdownFormatter {
    pub fn format(&self, content: Vec<DocBlock>) -> Result<String, fmt::Error> {
        let mut s = String::new();
        println!("MarkdownFormatter Run");
        // todo:
        return Ok(s);
    }
}

/*
Usage example:

let parser = create_file_parser(InputFileType::Lua);
let file = File::open("example.lua")?;
let blocks = parser.parse(&file);
let fmt = MarkdownFormatter;
for b in &blocks {
    let md = fmt.format(b);
    // write md to file
}

*/
