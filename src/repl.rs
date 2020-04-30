use crate::compiler;
use crate::environment::*;
use crate::error::MonkeyResult;
use crate::eval;
use crate::lexer::Lexer;
use crate::object;
use crate::parser::Parser;
use crate::symbol_table;
use crate::vm;

use colored::*;
use std::borrow::Cow;
use std::rc::Rc;
use std::cell::RefCell;
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::highlight::Highlighter;
use rustyline_derive::{Completer, Helper, Hinter};

const PROMPT: &str = "monkey » ";

#[derive(Completer, Helper, Hinter)]
struct ReplHelper;

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        fn try_pop(stack: &mut Vec<char>, c: char) -> bool {
            let top = stack.pop();
            top == Some(c)
        }

        let line = ctx.input();
        let mut stack = Vec::new();
        let mut skip_next = false;
        for c in line.split("//").next().unwrap().chars() {
            if skip_next {
                skip_next = false;
                continue;
            }
            if stack.last() == Some(&'"') {
                // We are in a string
                match c {
                    '\\' => skip_next = true, // Skip escaped quotes
                    '"' => {
                        stack.pop();
                    }
                    _ => (),
                }
            } else {
                match c {
                    '(' | '[' | '{' | '"' => stack.push(c),
                    ')' => if !try_pop(&mut stack, '(') {
                        return Ok(ValidationResult::Valid(None));
                    }
                    ']' => if !try_pop(&mut stack, '[') {
                        return Ok(ValidationResult::Valid(None));
                    }
                    '}' => if !try_pop(&mut stack, '{') {
                        return Ok(ValidationResult::Valid(None));
                    }
                    _ => (),
                }
            }
        }

        if stack.is_empty() {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool
    ) -> Cow<'b, str> {
        if default {
            Cow::Owned(format!("{}", PROMPT.blue().bold()))
        } else {
            Cow::Borrowed(prompt)
        }
    }
}

pub fn start(compiled: bool) -> Result<(), std::io::Error> {
    eprintln!("Now with an even fancier REPL!");
    eprintln!("(running using {})", if compiled { "compiler and VM" } else { "interpreter" });
    let mut rl = rustyline::Editor::<ReplHelper>::new();
    rl.set_helper(Some(ReplHelper {}));

    // Unbind Tab
    rl.unbind_sequence(rustyline::KeyPress::Tab);
    // Bind Tab to insert 4 spaces
    rl.bind_sequence(rustyline::KeyPress::Tab, rustyline::Cmd::Insert(1, "    ".into()));
    
    let res = if compiled {
        start_compiled(rl)
    } else {
        start_interpreted(rl)
    };
    eprintln!("Goodbye!");
    res
}

fn start_compiled(mut rl: rustyline::Editor<ReplHelper>) -> Result<(), std::io::Error> {
    // @WIP
    let mut constants = Vec::new();
    let mut symbol_table = symbol_table::SymbolTable::new();
    let mut globals = {
        let mut v = Vec::with_capacity(vm::GLOBALS_SIZE);
        v.resize(vm::GLOBALS_SIZE, object::Object::Nil);
        v.into_boxed_slice()
    };
    let mut run_line = |line: String| -> MonkeyResult<Vec<object::Object>> {
        // @PERFORMANCE: I really shouldn't be cloning everything
        let mut parser = Parser::new(Lexer::from_string(line)?)?;
        let mut comp = compiler::Compiler::with_state(constants.clone(), symbol_table.clone());
        comp.compile_block(parser.parse_program()?)?;
        constants = comp.constants.clone();
        symbol_table = comp.symbol_table.clone();
        let mut vm = vm::VM::with_globals(comp.bytecode(), globals.clone());
        vm.run()?;
        globals = vm.globals.clone();
        Ok(vec![vm.stack_top().unwrap().clone()]) // @TODO: Add proper errors
    };

    loop {
        match read_line(&mut rl) {
            Some(line) => print_results(run_line(line)),
            None => break Ok(()),
        }
    }
}

fn start_interpreted(mut rl: rustyline::Editor<ReplHelper>) -> Result<(), std::io::Error> {
    let env = Rc::new(RefCell::new(Environment::empty()));
    let run_line = |line: String| -> MonkeyResult<Vec<object::Object>> {
        let mut parser = Parser::new(Lexer::from_string(line)?)?;
        parser.parse_program()?
        .into_iter()
        .map(|s| eval::eval_statement(&s, &env))
        .collect()
    };

    loop {
        match read_line(&mut rl) {
            Some(line) => print_results(run_line(line)),
            None => break Ok(()),
        }
    }
}

fn read_line(rl: &mut rustyline::Editor<ReplHelper>) -> Option<String> {
    let readline = rl.readline(PROMPT);
    match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_str());
            return Some(line);
        },
        Err(ReadlineError::Interrupted) => println!("CTRL-C"),
        Err(ReadlineError::Eof) => println!("CTRL-D"),
        Err(other) => println!("rustyline Error: {:?}", other),
    };
    None
}

fn print_results(results: MonkeyResult<Vec<object::Object>>) {
    match results {
        Ok(values) => for v in values {
            println!("{}", v);
        }
        Err(e) => eprintln!("{}", e),
    }
}
