use core::fmt;
use once_cell::sync::Lazy;
use regex::Regex;
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
 * @includes <xxx>, <xxx>
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
    pub includes: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub descriptions: Vec<Description>,
    pub ret_value: Option<Parameter>,
    pub owner_object: String,
    pub is_local: bool,
}

/// 解析器 trait：把文件解析成一组 DocBlock（中间结构）
pub trait FileParser {
    fn parse(&self, file: &File) -> Vec<DocBlock>;
}

/// 输出格式化器 trait：把 DocBlock 转为目标格式字符串
pub trait OutputFileFormatter {
    fn format(&self, content: &DocBlock) -> String;
}

fn is_space_line(line: &str) -> bool {
    for c in line.chars() {
        if !c.is_whitespace() {
            return false;
        }
    }
    return true;
}
pub struct LuaFileParser {}
impl LuaFileParser {
    const ANNOTATION: &'static str = "-- ";

    pub fn is_annotation_line(line: &str) -> bool {
        line.trim_start().starts_with(Self::ANNOTATION)
    }

    pub fn is_api_tail(line: &str) -> bool {
        return line.ends_with(")") || line.trim_end().ends_with("end");
    }

    pub fn remove_annotation(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let n = chars.len();
        let mut i = 0usize;
        let mut in_squote = false;
        let mut in_dquote = false;
        let mut in_longstring_level: Option<usize> = None;

        while i < n {
            if in_squote {
                let c = chars[i];
                out.push(c);
                if c == '\\' {
                    if i + 1 < n {
                        i += 1;
                        out.push(chars[i]);
                    }
                } else if c == '\'' {
                    in_squote = false;
                }
                i += 1;
                continue;
            }

            if in_dquote {
                let c = chars[i];
                out.push(c);
                if c == '\\' {
                    if i + 1 < n {
                        i += 1;
                        out.push(chars[i]);
                    }
                } else if c == '"' {
                    in_dquote = false;
                }
                i += 1;
                continue;
            }

            if let Some(level) = in_longstring_level {
                if chars[i] == ']' {
                    let mut j = i + 1;
                    let mut eq = 0usize;
                    while j < n && chars[j] == '=' {
                        eq += 1;
                        j += 1;
                    }
                    if eq == level && j < n && chars[j] == ']' {
                        for t in i..=j {
                            out.push(chars[t]);
                        }
                        i = j + 1;
                        in_longstring_level = None;
                        continue;
                    }
                }
                out.push(chars[i]);
                i += 1;
                continue;
            }

            let c = chars[i];
            if c == '\'' {
                in_squote = true;
                out.push(c);
                i += 1;
                continue;
            }
            if c == '"' {
                in_dquote = true;
                out.push(c);
                i += 1;
                continue;
            }

            if c == '[' {
                let mut j = i + 1;
                let mut eq = 0usize;
                while j < n && chars[j] == '=' {
                    eq += 1;
                    j += 1;
                }
                if j < n && chars[j] == '[' {
                    in_longstring_level = Some(eq);
                    for t in i..=j {
                        out.push(chars[t]);
                    }
                    i = j + 1;
                    continue;
                }
            }

            // detect comment start '--'
            if c == '-' && i + 1 < n && chars[i + 1] == '-' {
                if i + 2 < n && chars[i + 2] == '[' {
                    let mut j = i + 3;
                    let mut eq = 0usize;
                    while j < n && chars[j] == '=' {
                        eq += 1;
                        j += 1;
                    }
                    if j < n && chars[j] == '[' {
                        let mut k = j + 1;
                        let mut found = None;
                        while k < n {
                            if chars[k] == ']' {
                                let mut m = k + 1;
                                let mut eq2 = 0usize;
                                while m < n && chars[m] == '=' {
                                    eq2 += 1;
                                    m += 1;
                                }
                                if eq2 == eq && m < n && chars[m] == ']' {
                                    found = Some(m);
                                    break;
                                }
                            }
                            k += 1;
                        }
                        if let Some(endpos) = found {
                            i = endpos + 1;
                            continue;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            out.push(c);
            i += 1;
        }

        out.trim_start().trim_end().to_string()
    }

    pub fn create_docblock(buf: Vec<String>) -> DocBlock {
        return DocBlock {
            signature: String::new(),
            brief: String::new(),
            note: String::new(),
            includes: vec![],
            parameters: vec![],
            descriptions: vec![],
            ret_value: None,
            owner_object: String::new(),
            is_local: false,
        };
    }
}
impl FileParser for LuaFileParser {
    fn parse(&self, file: &File) -> Vec<DocBlock> {
        let reader = BufReader::new(file);
        let mut code_line_no = 0;
        let mut line_buf = Vec::<String>::new();
        let mut doc_blocks = Vec::<DocBlock>::new();
        let mut real_code_line = String::new();
        let mut is_mutli_line_function_decl = false;
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    code_line_no += 1;
                    if is_space_line(&l) {
                        // 如果是空行则清空缓冲区
                        println!("空行 no: {}", code_line_no);
                        line_buf.clear();
                        real_code_line.clear();
                    } else if LuaFileParser::is_annotation_line(&l) {
                        println!("注释行 no: {}: {}", code_line_no, l);
                        if (l.starts_with("-- @") || l.starts_with("---@"))
                            && !l.starts_with("-- @!")
                        {
                            line_buf.push(l);
                        }
                    } else {
                        let l = LuaFileParser::remove_annotation(&l);
                        if l.trim_start().starts_with("function")
                            || l.trim_start().starts_with("local function")
                        {
                            if l.find("(").is_some() && !l.ends_with(")") {
                                is_mutli_line_function_decl = true;
                                real_code_line += &l;
                                println!("代码行_分段函数声明start no: {}", code_line_no);
                                continue;
                            } else if LuaFileParser::is_api_tail(&l) {
                                real_code_line += &l;

                                println!(
                                    "代码行_函数声明 no: {}: {}",
                                    code_line_no, real_code_line
                                );
                                line_buf.push(real_code_line.clone());
                                doc_blocks.push(LuaFileParser::create_docblock(line_buf.clone()));
                                line_buf.clear();
                                real_code_line.clear();
                            }
                        }
                        if is_mutli_line_function_decl {
                            real_code_line += &l;
                            println!(
                                "代码行》》》函数声明中 no: {}: {}",
                                code_line_no, real_code_line
                            );
                            if LuaFileParser::is_api_tail(&l) {
                                is_mutli_line_function_decl = false;
                                println!(
                                    "代码行_函数声明end no: {}: {}",
                                    code_line_no, real_code_line
                                );
                                line_buf.push(real_code_line.clone());
                                doc_blocks.push(LuaFileParser::create_docblock(line_buf.clone()));
                                line_buf.clear();
                                real_code_line.clear();
                            }
                        }
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
