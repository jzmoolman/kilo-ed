use std::string::ToString;

type EditorFlag = u32;

pub mod highlightflags {
    pub const NUMBERS : u32 = 0x0000_0001;
}

pub struct EditorSyntax {
    pub filetype: String,
    pub filematch: Vec<String>,
    pub flag: EditorFlag,
}

impl EditorSyntax {
    pub fn new() -> Vec<Self> {
        let mut result = Vec::new();
        result.push(EditorSyntax {
            filetype: "c".to_string(),
            filematch: vec![".c".to_string(), ".h".to_string(), ".cpp".to_string()],
            flag: highlightflags::NUMBERS,
        });
        result
    }
}