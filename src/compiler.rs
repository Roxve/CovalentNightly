use std::env::current_exe;

use crate::parser::ast::Expr;
use crate::backend::c;
use crate::ir::gen::IRGen;
use crate::ir::Codegen;
use crate::parser::Parser;
use crate::parser::parse::Parse;

#[allow(unused)]
pub struct CSettings {
    compiler: Option<String>,
    flags: Vec<String>,
}

impl CSettings {
    pub fn new(compiler: Option<String>, flags: Vec<String>) -> Self {
        Self { compiler, flags }
    }
}


/* macro_rules! unwarp {
    ($back: expr, $vari: path) => {
        match $back {
            $vari(i) => i,
            _ => panic!(),
        }
    };
}*/

#[allow(unused)]
pub enum Backend {
    C(CSettings),
    Custom { name: String, settings: Vec<String> },
}
pub struct CompilerConfig {
    input: String, 
    pub libdir: String,
    pub backend: Backend,
    pub debug: bool,
    pub repl: bool,
    pub output: String,
}
impl CompilerConfig {
    pub fn new(input: String, backend: Backend, debug: bool, repl: bool, output: String) -> Self {
        Self {
            input, 
            libdir: format!("{}/lib", current_exe().unwrap().parent().unwrap().to_str().unwrap()),
            backend,
            debug,
            repl,
            output,
        }
    }
    pub fn run(&self) {
        let mut parser = Parser::new(self.input.clone()); 
        
        let prog: Vec<Expr> = parser.parse_prog();
        if self.debug {
            println!("parsed prog:\n {:#?}\nsrc: \n{:#?}", prog, parser);
        } 

        let mut codegen = Codegen::new();
        let ir = codegen.gen_prog(prog, parser.functions);
        dbg!(&ir);
        drop(codegen);
        match self.backend {
            Backend::C(_) => {
                c::compile(self, ir);
            }
            _ => todo!(),
        }
    }
}
