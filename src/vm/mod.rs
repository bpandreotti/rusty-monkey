#[cfg(test)]
mod tests;

use crate::compiler::code::*;
use crate::error::{MonkeyError, MonkeyResult, RuntimeError::*};
use crate::interpreter::object::*;
use crate::lexer::token::Token;

const STACK_SIZE: usize = 2048;
pub const GLOBALS_SIZE: usize = 65536;

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: Vec<Object>,
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
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
            globals,
        }
    }

    /// Resets the VM bytecode, without changing the current globals. Used in the REPL.
    pub fn reset_bytecode(&mut self, new_bytecode: Bytecode) {
        self.constants = new_bytecode.constants;
        self.instructions = new_bytecode.instructions;
    }

    pub fn run(&mut self) -> MonkeyResult<()> {
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
                    if !Object::is_truthy(&self.pop()?) {
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
                OpArray => {
                    let num_elements = read_u16(&self.instructions.0[pc + 1..]) as usize;
                    pc += 2;
                    let arr = self.stack.split_off(self.sp - num_elements);
                    self.sp -= num_elements;
                    self.push(Object::Array(arr))?;
                },
            }
            pc += 1;
        }
        Ok(())
    }

    pub fn stack_top(&self) -> MonkeyResult<&Object> {
        if self.sp == 0 {
            Err(MonkeyError::Vm(StackUnderflow))
        } else {
            Ok(&self.stack[self.sp - 1])
        }
    }

    fn push(&mut self, obj: Object) -> MonkeyResult<()> {
        if self.sp >= STACK_SIZE {
            Err(MonkeyError::Vm(StackOverflow))
        } else {
            self.stack.push(obj);
            self.sp += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> MonkeyResult<Object> {
        if self.sp == 0 {
            Err(MonkeyError::Vm(StackUnderflow))
        } else {
            self.sp -= 1;
            Ok(self.stack.pop().unwrap())
        }
    }

    fn execute_binary_operation(&mut self, operation: OpCode) -> MonkeyResult<()> {
        use Object::*;

        let right = self.pop()?;
        let left = self.pop()?;
        match (left, operation, right) {
            (Integer(l), op, Integer(r)) => self.execute_integer_operation(op, l, r),
            (Boolean(l), op , Boolean(r)) => self.execute_bool_operation(op, l, r),
            (Str(l), OpCode::OpAdd, Str(r)) => self.execute_str_concat(&l, &r),
            (l, op, r) => Err(MonkeyError::Vm(InfixTypeError(
                l.type_str(),
                op.equivalent_token().unwrap(),
                r.type_str(),
            ))),
        }
    }

    fn execute_integer_operation(&mut self, op: OpCode, left: i64, right: i64) -> MonkeyResult<()> {
        let result = match op {
            // Arithmetic operators
            OpCode::OpAdd => Object::Integer(left + right),
            OpCode::OpSub => Object::Integer(left - right),
            OpCode::OpMul => Object::Integer(left * right),
            OpCode::OpDiv if right == 0 => return Err(MonkeyError::Vm(DivOrModByZero)),
            OpCode::OpDiv => Object::Integer(left / right),
            OpCode::OpExponent if right < 0 => return Err(MonkeyError::Vm(NegativeExponent)),
            OpCode::OpExponent => Object::Integer(left.pow(right as u32)),
            OpCode::OpModulo if right == 0 => return Err(MonkeyError::Vm(DivOrModByZero)),
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
            _ => {
                return Err(MonkeyError::Vm(InfixTypeError(
                    "bool",
                    op.equivalent_token().unwrap(),
                    "bool",
                )))
            }
        };
        self.push(result)?;
        Ok(())
    }

    fn execute_str_concat(&mut self, left: &str, right: &str) -> MonkeyResult<()> {
        self.push(Object::Str(left.to_string() + right))
    }

    fn execute_prefix_operation(&mut self, op: OpCode) -> MonkeyResult<()> {
        let right = self.pop()?;
        match op {
            OpCode::OpPrefixMinus => {
                if let Object::Integer(i) = right {
                    self.push(Object::Integer(-i))?;
                } else {
                    return Err(MonkeyError::Vm(PrefixTypeError(
                        Token::Minus,
                        right.type_str(),
                    )));
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
