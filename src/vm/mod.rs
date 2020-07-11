#[cfg(test)]
mod tests;

use crate::builtins::{self, BuiltinFn};
use crate::compiler::code::*;
use crate::error::{MonkeyError, MonkeyResult, RuntimeError::*};
use crate::lexer::token::Token;
use crate::object::*;

use std::collections::HashMap;

const STACK_SIZE: usize = 2048;
pub const GLOBALS_SIZE: usize = 65536;

struct Frame {
    instructions: Instructions,
    pc: usize,
    base_pointer: usize,
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
    fn read_u16_from_top(&mut self) -> u16 {
        let value = read_u16(&self.top().instructions.0[self.top().pc + 1..]);
        self.top_mut().pc += 2;
        value
    }

    fn read_u8_from_top(&mut self) -> u8 {
        let value = self.top().instructions.0[self.top().pc + 1];
        self.top_mut().pc += 1;
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
                base_pointer: 0,
            };
            vec![root_frame]
        });
        let constants = bytecode.constants;

        loop {
            // If we reach the end of the instructions and we are at the root frame, this is the
            // end of the program and we break the loop. Otherwise, if we are not in the root frame,
            // we reached the end of a function and there was no `OpReturn` instruction at the end,
            // so we panic.
            if frame_stack.top().pc >= frame_stack.top().instructions.0.len() {
                if frame_stack.0.len() == 1 {
                    break; // End of program
                } else {
                    panic!("Reached end of instructions in non-root frame")
                }
            }

            use OpCode::*;
            let op = OpCode::from_byte(frame_stack.top().instructions.0[frame_stack.top().pc]);
            match op {
                OpConstant => {
                    let constant_index = frame_stack.read_u16_from_top() as usize;
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
                    let pos = frame_stack.read_u16_from_top() as usize;

                    // @PERFORMANCE: Using `is_truthy` might be slow
                    if !Object::is_truthy(&self.pop()?) {
                        frame_stack.top_mut().pc = pos - 1;
                    }
                }
                OpJump => {
                    let pos = frame_stack.read_u16_from_top() as usize;
                    frame_stack.top_mut().pc = pos - 1;
                }
                OpNil => self.push(Object::Nil)?,
                OpSetGlobal => {
                    let index = frame_stack.read_u16_from_top() as usize;
                    self.globals[index] = self.pop()?.clone();
                }
                OpGetGlobal => {
                    let index = frame_stack.read_u16_from_top() as usize;
                    // @PERFORMANCE: This clone may be slow
                    self.push(self.globals[index].clone())?;
                }
                OpSetLocal => {
                    let index = frame_stack.read_u8_from_top() as usize;
                    self.stack[frame_stack.top().base_pointer + index] = self.pop()?;
                }
                OpGetLocal => {
                    let index = frame_stack.read_u8_from_top() as usize;
                    // @PERFORMANCE: This clone may be slow
                    self.push(self.stack[frame_stack.top().base_pointer + index].clone())?
                }
                OpArray => {
                    let num_elements = frame_stack.read_u16_from_top() as usize;
                    let arr = self.stack.split_off(self.sp - num_elements);
                    self.sp -= num_elements;
                    self.push(Object::Array(Box::new(arr)))?;
                }
                OpHash => {
                    let num_elements = frame_stack.read_u16_from_top() as usize;
                    let entries = self.stack.split_off(self.sp - (2 * num_elements));
                    let mut map = HashMap::new();
                    for i in 0..num_elements {
                        let key = &entries[i * 2];
                        let value = &entries[i * 2 + 1];
                        let hashable = HashableObject::from_object(key.clone())
                            .ok_or_else(|| MonkeyError::Vm(HashKeyTypeError(key.type_str())))?;
                        map.insert(hashable, value.clone());
                    }
                    self.sp -= num_elements * 2;
                    self.push(Object::Hash(Box::new(map)))?;
                }
                OpIndex => {
                    let index = self.pop()?;
                    let obj = self.pop()?;
                    self.execute_index_operation(obj, index)?;
                }
                OpCall => {
                    let num_args = frame_stack.read_u8_from_top() as usize;
                    // @PERFORMANCE: This `remove` might be slow. Specifically, it's O(num_args).
                    // Using `swap_remove` would be faster, but it would leave an object in the
                    // stack that would have to be popped off later.
                    let func = self.stack.remove(self.sp - 1 - num_args);
                    self.sp -= 1;
                    match func {
                        Object::Closure(c) => {
                            self.execute_closure_call(&mut frame_stack, *c, num_args)?;
                            continue; // Skip the pc increment
                        }
                        Object::Builtin(f) => self.execute_builtin_call(f, num_args)?,
                        _ => return Err(MonkeyError::Vm(NotCallable(func.type_str()))),
                    }
                }
                OpReturn => {
                    let returned_value = self.pop()?;
                    self.sp = frame_stack.top().base_pointer;
                    self.stack.truncate(self.sp);
                    frame_stack.pop();
                    self.push(returned_value)?;
                    continue;
                }
                OpGetBuiltin => {
                    let index = frame_stack.read_u8_from_top() as usize;
                    let builtin = builtins::ALL_BUILTINS[index].1.clone();
                    self.push(Object::Builtin(builtin))?;
                }
                OpClosure => {
                    let constant_index = frame_stack.read_u16_from_top() as usize;
                    let _num_free_variables = frame_stack.read_u8_from_top() as usize; // @WIP
                    let func = constants[constant_index].clone();
                    if let Object::CompiledFunc(func) = func {
                        let closure = Closure {
                            func: *func,
                            free_vars: vec![],
                        };
                        self.push(Object::Closure(Box::new(closure)))?;
                    } else {
                        panic!("Trying to build closure with non-function object");
                    }
                },
                OpGetFree => todo!(),
            }

            frame_stack.top_mut().pc += 1;
        }
        Ok(())
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

    pub fn pop(&mut self) -> MonkeyResult<Object> {
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
        self.push(Object::Str(Box::new(left.to_string() + right)))
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
                let key = HashableObject::from_object(key.clone())
                    .ok_or(MonkeyError::Vm(HashKeyTypeError(key_type)))?;
                let value = map.get(&key).ok_or(MonkeyError::Vm(KeyError(key)))?;
                Ok(value.clone())
            }
            (Object::Str(s), Object::Integer(i)) => {
                let chars = s.chars().collect::<Vec<_>>();
                if i < 0 || i >= chars.len() as i64 {
                    Err(IndexOutOfBounds(i))
                } else {
                    Ok(Object::Str(Box::new(chars[i as usize].to_string())))
                }
            }
            (Object::Str(_), other) => Err(IndexTypeError(other.type_str())),
            (other, _) => Err(IndexingWrongType(other.type_str())),
        };
        let result = result.map_err(MonkeyError::Vm)?;
        self.push(result)
    }

    fn execute_closure_call(
        &mut self,
        frame_stack: &mut FrameStack,
        closure: Closure,
        num_args: usize,
    ) -> MonkeyResult<()> {
        if closure.func.num_params as usize != num_args {
            return Err(MonkeyError::Vm(WrongNumberOfArgs(
                closure.func.num_params as usize,
                num_args,
            )));
        }
        frame_stack.top_mut().pc += 1;
        let new_frame = Frame {
            instructions: closure.func.instructions,
            pc: 0,
            base_pointer: self.sp - num_args,
        };
        frame_stack.push(new_frame);
        self.sp += closure.func.num_locals as usize;
        // @PERFORMANCE: This resize is slow, because it has to copy over `Object::Nil`. It
        // would be faster to use `Vec::set_len`, but that method is unsafe. I'm fairly certain
        // that it would be fine (safety wise) in these circumstances, but just to be sure I'm
        // using `resize` for now.
        self.stack.resize(self.sp, Object::Nil);
        Ok(())
    }

    fn execute_builtin_call(&mut self, func: BuiltinFn, num_args: usize) -> MonkeyResult<()> {
        // @PERFORMANCE: This has to allocate a vector and move over the arguments. It might be
        // better for the built-in functions to just take a slice of objects instead of a `Vec`.
        let args = self.stack.split_off(self.sp - num_args);
        self.sp -= num_args;
        let result = func.0(args).map_err(MonkeyError::Vm)?;
        self.push(result)
    }
}
