use core::fmt;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
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

#[derive(Debug)]
pub enum FormulaType {
    Inline,
    Block,
}
#[derive(Debug)]
pub enum OutputFileType {
    Markdown,
}

/// 中间文档结构（简化）
#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub number: usize,
    pub description: String,
    pub type_name: String,
}

#[derive(Debug)]
pub enum DescriptionType {
    Text(String),
    Code(InputFileType, String),
    MathFormula(FormulaType, String),
    BulletList(i32, String),
    HTMLLink(String),
}

#[derive(Debug)]
pub struct Description {
    pub dtype: DescriptionType,
    pub content: String,
}
/**
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
#[derive(Debug)]
pub struct DocBlock {
    pub signature   : String,
    pub brief       : String,
    pub note        : String,
    pub includes    : Vec<String>,
    pub parameters  : Vec<Parameter>,
    pub descriptions: Vec<Description>,
    pub ret_value   : Option<Parameter>,
    pub owner_object: String,
    pub is_local    : bool,
    pub is_member   : bool,
}

impl std::fmt::Display for DocBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Signature: {}", self.signature)?;
        writeln!(f, "Brief: {}", self.brief)?;
        writeln!(f, "Note: {}", self.note)?;
        writeln!(f, "Includes: {:?}", self.includes)?;
        writeln!(f, "Parameters:")?;
        for p in &self.parameters {
            writeln!(
                f,
                "  - {}: {} ({})",
                p.name, p.type_name, p.description
            )?;
        }
        if let Some(ret) = &self.ret_value {
            writeln!(
                f,
                "Return: {} ({})",
                ret.type_name, ret.description
            )?;
        }
        writeln!(f, "Descriptions:")?;
        for d in &self.descriptions {
            writeln!(f, "  - {:?}: {}", d.dtype, d.content)?;
        }
        Ok(())
    }
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

    pub fn is_doc_comment(line: &str) -> bool {
        let t = line.trim_start();
        return t.starts_with("---@") || t.starts_with("--@") || t.starts_with("-- @");
    }

    pub fn extract_owner_object(line: &str) -> String {
        let line = line.trim();
        let s = if line.starts_with("local function") {
            return "local".to_string();
        } else if line.starts_with("function") {
            &line[8..]
        } else {
            return String::new();
        };

        let s = s.trim_start();
        if let Some(idx) = s.find(|c| c == '.' || c == ':') {
            return s[..idx].trim().to_string();
        }
        String::new()
    }

    pub fn is_member_function(line: &str, obj_name: &str) -> bool {
        return if line.find(":").is_some() == false {
            let dot_idx = line.find(".");
            if dot_idx.is_some() {
                let left_bracket_idx = line.find("(").unwrap();
                let right_bracket_idx = line.find(")").unwrap();
                let sub_params_list = &line[left_bracket_idx..right_bracket_idx];
                let first_comma_idx = sub_params_list.find(",");
                if first_comma_idx.is_some() {
                    // 有参数列表的情况下 拿到第一个参数
                    let first_param = &sub_params_list[1..first_comma_idx.unwrap()].trim();
                    if *first_param == obj_name { true } else { false }
                }else{
                    // 一个参数就是table name, function A.function(A) 也是一个成员函数
                    if sub_params_list.trim().len() > 0 && sub_params_list == obj_name { true }else{ false }
                }
            }else{
                false
            }
        } else {
            true
        };
    }
    /// 解析并创建一个 DocBlock
    /// 这里采用了两层解析结构：
    /// 1. 第一层：识别 @tag
    /// 2. 第二层：如果处于 @description 下，识别 \subtag
    pub fn create_docblock(buf: Vec<String>) -> DocBlock {
        for str in &buf {
            println!("Doc Line: {}", str);
        }
        let mut block = DocBlock {
            signature   : String::new(),
            brief       : String::new(),
            note        : String::new(),
            includes    : vec![],
            parameters  : vec![],
            descriptions: vec![],
            ret_value   : None,
            owner_object: String::new(),
            is_local    : false,
            is_member   : false,
        };

        // 简单的状态机，用于处理多行内容（例如 description 下的子标签）
        let mut current_tag = String::new();

        for line in buf {
            // 1. 清理注释符号，获取纯文本内容
            // 简单实现：找到第一个 @ 或者 \ 之前的部分作为前缀去除，或者直接去除 --
            // 实际工程中建议用正则或精确匹配
            let content = if let Some(idx) = line.find("@") {
                &line[idx..]
            } else if let Some(idx) = line.find("\\") {
                &line[idx..]
            } else {
                let t = line.trim_start();
                if t.starts_with("--") {
                    t.trim_start_matches('-').trim()
                } else {
                    t
                }
            };
            
            // 2. 解析主标签 @xxx
            if content.starts_with("@") {
                let parts: Vec<&str> = content.splitn(2, |c: char| c.is_whitespace()).collect();
                let tag = &parts[0][1..]; // skip '@'
                let body = if parts.len() > 1 { parts[1].trim() } else { "" };
                
                current_tag = tag.to_string();

                match tag {
                    "brief" => block.brief = body.to_string(),
                    "param" => {
                        // 解析 param: name type desc
                        let p_parts: Vec<&str> = body.split_whitespace().collect();
                        if p_parts.len() >= 2 {
                            block.parameters.push(Parameter {
                                name: p_parts[0].to_string(),
                                type_name: p_parts[1].to_string(),
                                number: block.parameters.len(),
                                description: p_parts[2..].join(" "),
                            });
                        }
                    }
                    "return" => {
                         let p_parts: Vec<&str> = body.split_whitespace().collect();
                         if !p_parts.is_empty() {
                            block.ret_value = Some(Parameter {
                                name: "".to_string(),
                                type_name: p_parts[0].to_string(),
                                number: 0,
                                description: p_parts[1..].join(" "),
                            });
                         }
                    }
                    "includes" => {
                        // 简单逗号分隔
                        for inc in body.split(',') {
                            block.includes.push(inc.trim().to_string());
                        }
                    }
                    "note" => block.note = body.to_string(),
                    "description" => {
                        // 进入 description 模式，后续行可能包含 \text 等
                    }
                    _ => {
                        println!("Unknown tag: {}", tag);
                    }
                }
            } else if content.starts_with("\\") {
                 // 3. 解析子标签 (仅当在 description 下，或者设计为全局可用)
                 // 为了扩展性，这里可以进一步封装成 parse_description_line(content)
                if current_tag == "description" {
                    let parts: Vec<&str> = content.splitn(2, |c: char| c.is_whitespace()).collect();
                    let subtag = &parts[0][1..]; // skip '\'
                    let body = if parts.len() > 1 { parts[1].trim() } else { "" };
                    
                    let desc_type = match subtag {
                        "text" => Some(DescriptionType::Text(body.to_string())),
                        "code" => Some(DescriptionType::Code(InputFileType::None, body.to_string())), // 需要更复杂的解析来支持 code{lua}
                        "formula" => Some(DescriptionType::MathFormula(FormulaType::Inline, body.to_string())),
                        "list" => Some(DescriptionType::BulletList(0, body.to_string())),
                        "html" => Some(DescriptionType::HTMLLink(body.to_string())),
                        _ => None,
                    };

                    if let Some(dt) = desc_type {
                        block.descriptions.push(Description {
                            dtype: dt,
                            content: body.to_string(),
                        });
                    }
                }
            }
        }

        return block;
    }
}
impl FileParser for LuaFileParser {
    fn parse(&self, file: &File) -> Vec<DocBlock> {
        let reader = BufReader::new(file);
        let mut line_buf = Vec::<String>::new();
        let mut doc_blocks = Vec::<DocBlock>::new();
        let mut real_code_line = String::new();
        let mut is_mutli_line_function_decl = false;
        
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    println!("Read: {}", &l);
                    // 1. 收集文档行：只要是符合文档标记的行，或者在收集过程中遇到的普通注释行
                    let is_comment = l.trim_start().starts_with("--");
                    if LuaFileParser::is_doc_comment(&l) || (!line_buf.is_empty() && is_comment) {
                         line_buf.push(l);
                         continue;
                    }
                    
                    if is_space_line(&l) {
                        // 空行通常意味着文档块和函数声明断开了连接 (根据具体风格决定)
                        line_buf.clear();
                        real_code_line.clear();
                        continue;
                    }

                    // 2. 解析代码行
                    let code_content = LuaFileParser::remove_annotation(&l);
                    
                    // 简单判断是否开始函数定义
                    if code_content.trim_start().starts_with("function")
                        || code_content.trim_start().starts_with("local function")
                    {
                        // 拼接多行函数声明
                        if code_content.find("(").is_some() && !code_content.ends_with(")") {
                             is_mutli_line_function_decl = true;
                             real_code_line += &code_content;
                        } else if LuaFileParser::is_api_tail(&code_content) || code_content.contains(")") {
                             // 单行函数定义结束 (简单判定)
                             real_code_line += &code_content;
                             
                             // 核心逻辑：如果缓冲区有文档内容，则创建一个 Block 并关联
                             if !line_buf.is_empty() {
                                 let mut block = LuaFileParser::create_docblock(line_buf.clone());
                                 block.signature = real_code_line.clone();
                                 let _m_ret = LuaFileParser::extract_owner_object(&real_code_line);
                                 if _m_ret == "local" {
                                    block.is_local = true;
                                    block.is_member = false;
                                    block.owner_object = "".to_string();
                                 }else{
                                    block.owner_object = _m_ret;
                                    block.is_member = LuaFileParser::is_member_function(&real_code_line, &block.owner_object);
                                 }
                                 
                                 doc_blocks.push(block);
                                 line_buf.clear(); // 消费掉 buffer
                             }
                             real_code_line.clear();
                        }
                    } else if is_mutli_line_function_decl {
                        // 处理多行函数的后续部分
                        real_code_line += &code_content;
                        
                        // 检查这一行是否结束了函数声明（包含 ')' 或者以 'end' 结尾）
                        if LuaFileParser::is_api_tail(&code_content) || code_content.contains(")") {
                            is_mutli_line_function_decl = false;
                            
                            if !line_buf.is_empty() {
                                let mut block = LuaFileParser::create_docblock(line_buf.clone());
                                block.signature = real_code_line.clone();
                                let _m_ret = LuaFileParser::extract_owner_object(&real_code_line);
                                if _m_ret == "local" {
                                   block.is_local = true;
                                   block.is_member = false;
                                   block.owner_object = "".to_string();
                                } else {
                                   block.owner_object = _m_ret;
                                   // 现在 real_code_line 应该是完整的，包含 ')'，所以 is_member_function 不会 panic
                                   block.is_member = LuaFileParser::is_member_function(&real_code_line, &block.owner_object);
                                }
                                doc_blocks.push(block);
                                line_buf.clear();
                            }
                            real_code_line.clear();
                        }
                        // 如果还没结束，保持 is_mutli_line_function_decl = true，继续读下一行拼接
                        
                    } else {
                        // 其他非空行代码，清空之前的 doc buffer (因为它没有紧跟函数)
                        // line_buf.clear(); 
                        // *注*: 这里看需求，如果允许 doc 上方有少量非空行干扰，可以不 clear
                        // 但通常 doc 紧贴 function。
                        line_buf.clear(); 
                    }
                    
                }
                Err(_) => continue,
            }
        }

        return doc_blocks;
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
    /// 格式化函数签名
    fn format_signature(&self, signature: &str) -> String {
        format!("```lua\n{}\n```\n", signature)
    }

    /// 格式化 Includes
    fn format_includes(&self, includes: &[String]) -> String {
        if includes.is_empty() {
            return String::new();
        }
        format!("**Includes:** {}\n\n", includes.join(", "))
    }

    /// 格式化 Brief
    fn format_brief(&self, brief: &str) -> String {
        if brief.is_empty() {
            return String::new();
        }
        format!("**Brief:** {}\n\n", brief)
    }

    /// 格式化参数列表
    fn format_parameters(&self, params: &[Parameter]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let mut s = String::from("**Parameters:**\n");
        for p in params {
            use std::fmt::Write;
            let _ = writeln!(s, "- {} ({}): {}", p.name, p.type_name, p.description);
        }
        s.push('\n');
        s
    }

    /// 格式化返回值
    fn format_return(&self, ret: &Option<Parameter>) -> String {
        match ret {
            Some(p) => format!(
                "**Returns:** {} ({}): {}\n\n",
                p.name, p.type_name, p.description
            ),
            None => String::new(),
        }
    }

    /// 格式化单个描述项
    fn format_description_item(&self, desc: &Description) -> String {
        match &desc.dtype {
            DescriptionType::Text(_) => format!("{}\n", desc.content),
            DescriptionType::Code(lang, _) => {
                let lang_str = lang.to_str().unwrap_or("");
                format!("```{}\n{}\n```\n", lang_str, desc.content)
            }
            DescriptionType::MathFormula(ft, _) => match ft {
                FormulaType::Inline => format!("${}$\n", desc.content),
                FormulaType::Block => format!("$$\n{}\n$$\n", desc.content),
            },
            DescriptionType::BulletList(_, _) => {
                // 如果内容本身不包含 '- ' 前缀，则补上
                let content = desc.content.trim();
                let prefix = if content.starts_with("-") {
                    ""
                } else {
                    "- "
                };
                format!("{}{}\n", prefix, content)
            }
            DescriptionType::HTMLLink(_) => {
                // [link](url) - 这里假设 content 是 url
                format!("[{}]({})\n", desc.content, desc.content)
            }
        }
    }

    /// 格式化描述部分
    fn format_descriptions(&self, descriptions: &[Description]) -> String {
        if descriptions.is_empty() {
            return String::new();
        }
        let mut s = String::from("**Description:**\n\n");
        for d in descriptions {
            s.push_str(&self.format_description_item(d));
        }
        s.push('\n');
        s
    }

    /// 格式化单个 DocBlock
    fn format_block(&self, block: &DocBlock) -> String {
        let mut s = String::new();
        
        // 1. Signature
        s.push_str(&self.format_signature(&block.signature));

        // 2. Includes
        s.push_str(&self.format_includes(&block.includes));

        // 3. Brief
        s.push_str(&self.format_brief(&block.brief));

        // 4. Parameters
        s.push_str(&self.format_parameters(&block.parameters));

        // 5. Returns
        s.push_str(&self.format_return(&block.ret_value));

        // 6. Detailed Descriptions
        s.push_str(&self.format_descriptions(&block.descriptions));

        s
    }

    pub fn format(&self, content: Vec<DocBlock>) -> Result<String, fmt::Error> {
        let mut s = String::new();
        for block in content {
            s.push_str(&self.format_block(&block));
            s.push_str("---\n\n");
        }
        Ok(s)
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
