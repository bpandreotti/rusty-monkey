use crate::compiler;
use crate::environment::*;
use crate::error::MonkeyResult;
use crate::eval;
use crate::lexer::Lexer;
use crate::object;
use crate::parser::Parser;
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
    
    let env = Rc::new(RefCell::new(Environment::empty()));
    
    loop {
        let readline = rl.readline(PROMPT);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match run_line(line, &env, compiled) {
                    Ok(values) => for v in values {
                        println!("{}", v);
                    }
                    Err(e) => eprintln!("{}", e),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(other) => {
                println!("rustyline Error: {:?}", other);
                break;
            }
        }
    }

    eprintln!("Goodbye!");
    Ok(())
}

fn run_line(line: String, env: &EnvHandle, compile: bool) -> MonkeyResult<Vec<object::Object>> {
    let lexer = Lexer::from_string(line)?;
    let mut parser = Parser::new(lexer)?;
    let mut results = Vec::new();
    for statement in parser.parse_program()? {
        let value = if compile {
            let mut comp = compiler::Compiler::new();
            comp.compile_block(vec![statement])?;
            let mut vm = vm::VM::new(comp.bytecode());
            vm.run()?;
            vm.last_popped().clone()
        } else {
            eval::eval_statement(&statement, &env)?
        };
        results.push(value);
    }
    Ok(results)
}
