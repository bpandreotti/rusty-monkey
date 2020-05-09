pub mod object;
#[cfg(test)]
mod tests;

use crate::compiler::code::*;
use crate::error::{MonkeyError, MonkeyResult, RuntimeError::*};
use crate::hashable::HashableObject;
use crate::lexer::token::Token;
use object::*;

use std::collections::HashMap;

const STACK_SIZE: usize = 2048;
pub const GLOBALS_SIZE: usize = 65536;

struct Frame {
    instructions: Instructions,
    pc: usize,
}

struct FrameStack(Vec<Frame>);

impl FrameStack {
    fn top(&self) -> &Frame {
        self.0.last().expect("No frames in frame stack")
    }

    fn top_mut(&mut self) -> &mut Frame {
        self.0.last_mut().expect("No frames in frame stack")
    }

    fn push(&mut self, frame: Frame) {
        self.0.push(frame);
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    // Reads a u16 from the top frame, and incremets its program counter
    fn read_u16_from_top(&mut self) -> usize {
        let value = read_u16(&self.top().instructions.0[self.top().pc + 1..]) as usize;
        self.top_mut().pc += 2;
        value
    }
}

pub struct VM {
    stack: Vec<Object>,
    sp: usize,
    pub globals: Box<[Object]>,
}

impl VM {
    pub fn new() -> VM {
        // @PERFORMANCE: Maybe we shouldn't allocate all the memory for the globals upfront.
        let mut globals = Vec::with_capacity(GLOBALS_SIZE);
        globals.resize(GLOBALS_SIZE, Object::Nil);
        let globals = globals.into_boxed_slice();

        VM {
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
            globals,
        }
    }

    pub fn run(&mut self, bytecode: Bytecode) -> MonkeyResult<()> {
        // We can't store the frames in the VM struct because we need to borrow both `self` and the
        // current frame mutably at the same time. If the frames were part of `self`, that would
        // mean two mutable references to `self`.
        let mut frame_stack = FrameStack({
            let root_frame = Frame {
                instructions: bytecode.instructions,
                pc: 0,
            };
            vec![root_frame]
        });
        let constants = bytecode.constants;
        
        while !frame_stack.0.is_empty() {
            // If we reached the end of the current frame, pop it off and restart
            if frame_stack.top().pc >= frame_stack.top().instructions.0.len() {
                frame_stack.pop();
                continue;
            }
            use OpCode::*;
            let op = OpCode::from_byte(frame_stack.top().instructions.0[frame_stack.top().pc]);
            match op {
                OpConstant => {
                    let constant_index = frame_stack.read_u16_from_top();
                    self.push(constants[constant_index].clone())?;
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
                    let pos = frame_stack.read_u16_from_top();

                    // @PERFORMANCE: Using `is_truthy` might be slow
                    if !Object::is_truthy(&self.pop()?) {
                        frame_stack.top_mut().pc = pos - 1;
                    }
                }
                OpJump => {
                    let pos = frame_stack.read_u16_from_top();
                    frame_stack.top_mut().pc = pos - 1;
                }
                OpNil => self.push(Object::Nil)?,
                OpSetGlobal => {
                    let index = frame_stack.read_u16_from_top();
                    self.globals[index] = self.pop()?.clone();
                }
                OpGetGlobal => {
                    let index = frame_stack.read_u16_from_top();
                    self.push(self.globals[index].clone())?;
                }
                OpArray => {
                    let num_elements = frame_stack.read_u16_from_top();
                    let arr = self.stack.split_off(self.sp - num_elements);
                    self.sp -= num_elements;
                    self.push(Object::Array(arr))?;
                }
                OpHash => {
                    let num_elements = frame_stack.read_u16_from_top();
                    let entries = self.stack.split_off(self.sp - (2 * num_elements));
                    let mut map = HashMap::new();
                    for i in 0..num_elements {
                        let key = &entries[i * 2];
                        let value = &entries[i * 2 + 1];
                        let hashable = HashableObject::from_vm_object(key.clone())
                            .ok_or_else(|| MonkeyError::Vm(HashKeyTypeError(key.type_str())))?;
                        map.insert(hashable, value.clone());
                    }
                    self.sp -= num_elements * 2;
                    self.push(Object::Hash(map))?;
                }
                OpIndex => {
                    let index = self.pop()?;
                    let obj = self.pop()?;
                    self.execute_index_operation(obj, index)?;
                }
                OpCall => {
                    let func = self.pop()?;
                    match func {
                        Object::CompiledFunction(instructions) => {
                            frame_stack.top_mut().pc += 1;
                            let new_frame = Frame { instructions, pc: 0 };
                            frame_stack.push(new_frame);
                            continue;
                        }
                        other => return Err(MonkeyError::Vm(NotCallable(other.type_str()))),
                    }
                }
                OpReturn => {
                    // @TODO: Clean stack after returning. For more information, see
                    // `tests::test_stack_cleaning_after_call`
                    frame_stack.pop();
                }
                _ => todo!(),
            }
            
            frame_stack.top_mut().pc += 1;            
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
            (Boolean(l), op, Boolean(r)) => self.execute_bool_operation(op, l, r),
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

    fn execute_index_operation(&mut self, obj: Object, index: Object) -> MonkeyResult<()> {
        let result = match (obj, index) {
            (Object::Array(vector), Object::Integer(i)) => {
                if i < 0 || i >= vector.len() as i64 {
                    Err(IndexOutOfBounds(i))
                } else {
                    Ok(vector.into_iter().nth(i as usize).unwrap())
                }
            }
            (Object::Array(_), other) => Err(IndexTypeError(other.type_str())),
            (Object::Hash(map), key) => {
                let key_type = key.type_str();
                let key = HashableObject::from_vm_object(key.clone())
                    .ok_or(MonkeyError::Vm(HashKeyTypeError(key_type)))?;
                let value = map.get(&key).ok_or(MonkeyError::Vm(KeyError(key)))?;
                Ok(value.clone())
            }
            (Object::Str(s), Object::Integer(i)) => {
                let chars = s.chars().collect::<Vec<_>>();
                if i < 0 || i >= chars.len() as i64 {
                    Err(IndexOutOfBounds(i))
                } else {
                    Ok(Object::Str(chars[i as usize].to_string()))
                }
            }
            (Object::Str(_), other) => Err(IndexTypeError(other.type_str())),
            (other, _) => Err(IndexingWrongType(other.type_str())),
        };
        let result = result.map_err(MonkeyError::Vm)?;
        self.push(result)
    }
}
