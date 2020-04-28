use crate::object::*;
use crate::code::*;
use crate::compiler;
use crate::error::*;

use std::mem;

const STACK_SIZE: usize = 2048;

struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: [Object; STACK_SIZE],
    sp: usize,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        // Since `Object` does not implement `Copy`, we can't just initialize an array of objects
        // like we would normally. In this case, I would like to be ably to just do
        // "[Object::Nil; STACK_SIZE]". Instead, I have to do this unsafe witchcraft.
        // Safety: We're creating an unitialized array of `MaybeUninit`, and this type doesn't need
        // any initialization, so this is safe.
        let mut stack: [mem::MaybeUninit<Object>; STACK_SIZE] = unsafe {
            mem::MaybeUninit::uninit().assume_init()
        };
        for item in &mut stack[..] {
            *item = mem::MaybeUninit::new(Object::Nil);
        }
        // Safety: Everything is initialized, so we can safely transmute here.
        let stack = unsafe { mem::transmute::<_, [Object; STACK_SIZE]>(stack) };

        VM {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack,
            sp: 0,
        }
    }

    fn run(&mut self) -> Result<(), MonkeyError> {
        let mut pc = 0;
        while pc < self.instructions.0.len() {
            let op = OpCode::from_byte(self.instructions.0[pc]);
            match op {
                OpCode::OpConstant => {
                    let constant_index = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;
                    self.push(self.constants[constant_index].clone())?;
                }
                OpCode::OpAdd => {
                    let right = self.pop()?;
                    let right = match right {
                        Object::Integer(i) => *i,
                        _ => panic!("type error"),
                    };
                    let left = self.pop()?;
                    let left = match left {
                        Object::Integer(i) => *i,
                        _ => panic!("type error"),
                    };
                    let result = Object::Integer(right + left);
                    self.push(result)?;
                }
                _ => todo!(),
            }
            pc += 1;
        }
        Ok(())
    }

    pub fn stack_top(&self) -> Option<&Object> {
        if self.sp == 0 {
            None
        } else {
            Some(&self.stack[self.sp - 1])
        }
    }

    fn push(&mut self, obj: Object) -> Result<(), MonkeyError> {
        if self.sp >= STACK_SIZE {
            panic!("stack overflow"); // @TODO: Add proper errors
        } else {
            self.stack[self.sp] = obj;
            self.sp += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<&Object, MonkeyError> {
        if self.sp == 0 {
            panic!("stack underflow");
        } else {
            self.sp -= 1;
            Ok(&self.stack[self.sp])
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(program: &str) -> Vec<NodeStatement> {
        let lex = Lexer::from_string(program.into()).unwrap();
        let mut parser = Parser::new(lex).unwrap();
        parser.parse_program().unwrap()
    }

    fn compile(program: Vec<NodeStatement>) -> Bytecode {
        let mut comp = compiler::Compiler::new();
        comp.compile_program(program).unwrap();
        comp.bytecode()
    }

    fn assert_integer(expected: i64, got: &Object) {
        match got {
            Object::Integer(i) => assert_eq!(*i, expected),
            _ => panic!("Wrong object type"),
        }
    }

    #[test]
    fn test_integer_arithmetic() {
        let bytecode = compile(parse("2 + 3"));
        let mut vm = VM::new(bytecode);
        vm.run().unwrap();
        assert_integer(5, vm.stack_top().unwrap());
    }
}
