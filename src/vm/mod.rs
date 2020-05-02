#[cfg(test)]
mod tests;

use crate::compiler::code::*;
use crate::error::*;
use crate::interpreter::object::*;

use std::mem;

const STACK_SIZE: usize = 2048;
pub const GLOBALS_SIZE: usize = 65536;

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: [Object; STACK_SIZE], // @PERFORMANCE: Maybe using a Vec here would be fine
    sp: usize,
    pub globals: Box<[Object]>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        // @PERFORMANCE: Maybe we shouldn't allocate all the memory for the globals upfront.
        let mut globals = Vec::with_capacity(GLOBALS_SIZE);
        globals.resize(GLOBALS_SIZE, Object::Nil);
        let globals = globals.into_boxed_slice();

        VM {
            constants: bytecode.constants,
            instructions: bytecode.instructions,
            stack: VM::initialize_new_stack(),
            sp: 0,
            globals,
        }
    }

    /// Resets the VM bytecode, without changing the current globals. Used in the REPL.
    pub fn reset_bytecode(&mut self, new_bytecode: Bytecode) {
        self.constants = new_bytecode.constants;
        self.instructions = new_bytecode.instructions;
    }

    fn initialize_new_stack() -> [Object; STACK_SIZE] {
        // Since `Object` does not implement `Copy`, we can't just initialize an array of objects
        // like we would normally. In this case, I would like to be able to just do
        // "[Object::Nil; STACK_SIZE]". Instead, I have to do this unsafe witchcraft.
        // Safety: We're creating an unitialized array of `MaybeUninit`, and this type doesn't need
        // any initialization, so this is safe.
        let mut stack: [mem::MaybeUninit<Object>; STACK_SIZE] =
            unsafe { mem::MaybeUninit::uninit().assume_init() };
        for item in &mut stack[..] {
            *item = mem::MaybeUninit::new(Object::Nil);
        }
        // Safety: Everything is initialized, so we can safely transmute here.
        unsafe { mem::transmute::<_, [Object; STACK_SIZE]>(stack) }
    }

    pub fn run(&mut self) -> Result<(), MonkeyError> {
        use OpCode::*;
        let mut pc = 0;
        while pc < self.instructions.0.len() {
            let op = OpCode::from_byte(self.instructions.0[pc]);
            match op {
                OpConstant => {
                    let constant_index = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;
                    self.push(self.constants[constant_index].clone())?;
                }
                OpPop => {
                    self.pop()?;
                }
                OpAdd | OpSub | OpMul | OpDiv | OpExponent | OpModulo | OpEquals | OpNotEquals
                | OpGreaterThan | OpGreaterEq => self.execute_binary_operation(op)?,
                OpTrue => self.push(Object::Boolean(true))?,
                OpFalse => self.push(Object::Boolean(false))?,
                OpPrefixMinus | OpPrefixNot => self.execute_prefix_operation(op)?,
                OpJumpNotTruthy => {
                    let pos = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;

                    // @PERFORMANCE: Using `is_truthy` might be slow
                    if !Object::is_truthy(self.pop()?) {
                        pc = pos - 1;
                    }
                }
                OpJump => {
                    let pos = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc = pos - 1;
                }
                OpNil => self.push(Object::Nil)?,
                OpSetGlobal => {
                    let index = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;
                    self.globals[index] = self.pop()?.clone();
                }
                OpGetGlobal => {
                    let index = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;
                    self.push(self.globals[index].clone())?;
                }
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

    fn execute_binary_operation(&mut self, op: OpCode) -> Result<(), MonkeyError> {
        // I'm matching on right and then on left (instead of matching both at the same time)
        // because, to please the borrow checker, I need to copy over the value inside the right
        // object before I get the left object with `self.pop`. I could just clone the whole
        // objects, but that would be much slower. Maybe there's a simpler way to do this.
        let right = self.pop()?;
        match *right {
            Object::Integer(r) => {
                let left = self.pop()?;
                if let Object::Integer(l) = *left {
                    self.execute_integer_operation(op, l, r)
                } else {
                    panic!("type error") // @TODO: Add proper errors
                }
            }
            Object::Boolean(r) => {
                let left = self.pop()?;
                if let Object::Boolean(l) = *left {
                    self.execute_bool_operation(op, l, r)
                } else {
                    panic!("type error") // @TODO: Add proper errors
                }
            }
            _ => todo!(),
        }
    }

    fn execute_integer_operation(&mut self, op: OpCode, left: i64, right: i64) -> MonkeyResult<()> {
        let result = match op {
            // Arithmetic operators
            OpCode::OpAdd => Object::Integer(left + right),
            OpCode::OpSub => Object::Integer(left - right),
            OpCode::OpMul => Object::Integer(left * right),
            OpCode::OpDiv => Object::Integer(left / right),
            // @TODO: Make sure exponent is positive
            OpCode::OpExponent => Object::Integer(left.pow(right as u32)),
            OpCode::OpModulo => Object::Integer(left % right),

            // Comparison operators
            OpCode::OpEquals => Object::Boolean(left == right),
            OpCode::OpNotEquals => Object::Boolean(left != right),
            OpCode::OpGreaterThan => Object::Boolean(left > right),
            OpCode::OpGreaterEq => Object::Boolean(left >= right),
            _ => unreachable!(),
        };
        self.push(result)?;
        Ok(())
    }

    fn execute_bool_operation(&mut self, op: OpCode, left: bool, right: bool) -> MonkeyResult<()> {
        let result = match op {
            OpCode::OpEquals => Object::Boolean(left == right),
            OpCode::OpNotEquals => Object::Boolean(left != right),
            _ => panic!("type error"), // @TODO: Add proper errors
        };
        self.push(result)?;
        Ok(())
    }

    fn execute_prefix_operation(&mut self, op: OpCode) -> Result<(), MonkeyError> {
        let right = self.pop()?;
        match op {
            OpCode::OpPrefixMinus => {
                if let Object::Integer(i) = *right {
                    self.push(Object::Integer(-i))?;
                } else {
                    panic!("type error") // @TODO: Add proper errors
                }
            }
            OpCode::OpPrefixNot => {
                // @PERFORMANCE: Using `is_truthy` might be slow
                let value = !right.is_truthy();
                self.push(Object::Boolean(value))?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
