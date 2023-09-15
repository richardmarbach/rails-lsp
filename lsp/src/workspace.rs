use std::{collections::HashMap, fs, path::PathBuf};

use ignore::{types::TypesBuilder, WalkBuilder};
use lib_ruby_parser::{Lexer, Parser, ParserOptions, ParserResult};

#[derive(Debug)]
pub struct Workspace {
    pub files: HashMap<String, ParserResult>,
    pub constants: HashMap<String, Vec<String>>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut workspace = Self::new();
        for path in paths {
            workspace.add_path(path);
        }
        workspace
    }

    pub fn from_path(path: PathBuf) -> Self {
        let mut workspace = Self::new();
        workspace.add_path(path);
        workspace
    }

    pub fn add_path(&mut self, path: PathBuf) {
        let files = load_ruby_files(path);
        let mut constants = HashMap::new();

        for (file, ast) in files.iter() {
            ast.tokens
                .iter()
                .filter(|token| token.token_type == Lexer::tCONSTANT)
                .for_each(|token| {
                    let constant = token.token_value.to_string_lossy();
                    constants
                        .entry(constant)
                        .or_insert(vec![])
                        .push(file.clone());
                });
        }

        self.files = files;
    }
}

fn load_ruby_files(path: PathBuf) -> HashMap<String, ParserResult> {
    let types = {
        let mut types = TypesBuilder::new();
        types
            .add("ruby", "*.rb")
            .expect("Ruby file pattern should never fail");
        types.select("ruby").build().expect("Types must build")
    };

    let mut files = HashMap::new();

    for result in WalkBuilder::new(path).types(types.clone()).build() {
        match result {
            Ok(entry) => {
                if entry
                    .file_type()
                    .expect("filetype is never stdin")
                    .is_file()
                {
                    let options = ParserOptions {
                        buffer_name: entry.path().to_string_lossy().to_string(),
                        ..Default::default()
                    };
                    let parser = Parser::new(
                        fs::read(entry.path()).expect("Failed to open file"),
                        options,
                    );

                    files
                        .entry(entry.path().to_string_lossy().to_string())
                        .or_insert(parser.do_parse());
                }
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }

    files
}
