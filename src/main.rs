#[cfg(test)]
#[macro_use]
mod test_utils;

mod builtins;
mod compiler;
mod error;
mod interpreter;
mod lexer;
mod object;
mod parser;
mod repl;
mod vm;

use error::MonkeyError;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut args = std::env::args();
    let first = args.nth(1);
    let second = args.next();
    let r = match (first.as_deref(), second.as_deref()) {
        (Some("-c"), None) | (None, _) => repl::start(true),
        (Some("-i"), None) => repl::start(false),
        (Some(path), None) | (Some("-c"), Some(path)) => run_program_file(true, path.into()),
        (Some("-i"), Some(path)) => run_program_file(false, path.into()),
        (Some(_), Some(_)) => panic!("Wrong arguments"),
    };
    if let Err(e) = r {
        eprintln!("{}", e)
    }
}

fn run_program_file(compiled: bool, path: String) -> Result<(), MonkeyError> {
    let reader = BufReader::new(File::open(path)?);
    let lexer = lexer::Lexer::new(Box::new(reader))?;
    let parsed_program = parser::Parser::new(lexer)?.parse_program()?;
    if compiled {
        let mut comp = compiler::Compiler::new();
        comp.compile_block(parsed_program)?;
        let code = comp.bytecode();
        let mut vm = vm::VM::new();
        vm.run(code)?;
    } else {
        interpreter::run_program(parsed_program)?;
    }
    Ok(())
}
