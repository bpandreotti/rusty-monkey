use crate::environment::*;
use crate::error::MonkeyResult;
use crate::eval;
use crate::lexer::Lexer;
use crate::object;
use crate::parser::Parser;

use std::io::BufRead;
use std::rc::Rc;
use std::cell::RefCell;

const PROMPT: &str = "monkey » ";
const CONTINUATION_PROMPT: &str = "   ... » ";

pub fn fancy() -> Repl {
    Repl::new(true)
}

pub fn old() -> Repl {
    Repl::new(false)
}

#[derive(PartialEq)]
enum ValidationResult {
    Done,
    Incomplete,
    Mismatched,
}

pub struct Repl {
    fancy: bool,
    stdin: std::io::Stdin,
    env: EnvHandle,
}

impl Repl {
    pub fn new(fancy: bool) -> Repl {
        Repl {
            fancy,
            stdin: std::io::stdin(),
            env: Rc::new(RefCell::new(Environment::empty())),
        }
    }

    pub fn start(self) -> Result<(), std::io::Error> {
        eprintln!("Hello! This is the Monkey programming language!");
        if self.fancy {
            self.start_fancy()
        } else {
            self.start_old()
        }
    }

    fn start_fancy(self) -> Result<(), std::io::Error> {
        eprintln!("Now with a fancy new REPL");
        eprint!("{}", PROMPT);
        let mut prefix = String::new();
        for line in self.stdin.lock().lines() {
            let line = prefix + &line?;
            if line == "exit" {
                break;
            }
    
            let validation_result = validate_matching_delimiters(&line);
            if validation_result == ValidationResult::Incomplete {
                prefix = line + "\n";
                eprint!("{}", CONTINUATION_PROMPT);
                continue;
            } else {
                prefix = String::new();
            }
    
            match self.run_line(line) {
                Ok(values) => for v in values {
                    println!("{}", v);
                }
                Err(e) => eprintln!("{}", e),
            }
            eprint!("{}", PROMPT);
        }
    
        eprintln!("Goodbye!");
        Ok(())
    }

    fn start_old(self) -> Result<(), std::io::Error> {
        eprint!("{}", PROMPT);
        for line in self.stdin.lock().lines() {
            let line = line?;
            if line == "exit" {
                break;
            }
    
            match self.run_line(line) {
                Ok(values) => for v in values {
                    println!("{}", v);
                }
                Err(e) => eprintln!("{}", e),
            }
            eprint!("{}", PROMPT);
        }
    
        eprintln!("Goodbye!");
        Ok(())
    }

    fn run_line(&self, line: String) -> MonkeyResult<Vec<object::Object>> {
        let lexer = Lexer::from_string(line)?;
        let mut parser = Parser::new(lexer)?;
        let mut results = Vec::new();
        for statement in parser.parse_program()? {
            results.push(eval::eval_statement(&statement, &self.env)?);
        }
        Ok(results)
    }
}

fn validate_matching_delimiters(line: &str) -> ValidationResult {
    fn try_pop(stack: &mut Vec<char>, c: char) -> bool {
        let top = stack.pop();
        top == Some(c)
    }

    let mut stack = Vec::new();
    let mut skip_next = false;
    for c in line.split("//").collect::<Vec<_>>()[0].chars() {
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
                    return ValidationResult::Mismatched;
                }
                ']' => if !try_pop(&mut stack, '[') {
                    return ValidationResult::Mismatched;
                }
                '}' => if !try_pop(&mut stack, '{') {
                    return ValidationResult::Mismatched;
                }
                _ => (),
            }
        }
    }

    if stack.is_empty() {
        ValidationResult::Done
    } else {
        ValidationResult::Incomplete
    }
}
