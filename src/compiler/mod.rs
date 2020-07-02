#[macro_use]
pub mod code;
pub mod symbol_table;
#[cfg(test)]
mod tests;

use crate::builtins;
use crate::error::{CompilerError::*, MonkeyError, MonkeyResult};
use crate::lexer::token::Token;
use crate::object::*;
use crate::parser::ast::*;
use code::*;
use symbol_table::*;

pub struct CompilationScope {
    instructions: Instructions,
}

pub struct Compiler {
    scopes: Vec<CompilationScope>,
    pub constants: Vec<Object>,
    pub symbol_table: Option<SymbolTable>,
}

impl Compiler {
    pub fn new() -> Compiler {
        let root_scope = CompilationScope {
            instructions: Instructions(Vec::new()),
        };
        let mut builtins_table = SymbolTable::new();
        for (index, &(name, _)) in builtins::ALL_BUILTINS.iter().enumerate() {
            builtins_table.define_builtin(name.into(), index);
        }

        Compiler {
            scopes: vec![root_scope],
            constants: Vec::new(),
            symbol_table: Some(builtins_table),
        }
    }

    /// Resets the instructions of the compiler, without changing the constants, and returns a
    /// `Bytecode` containing the old instructions and a clone of the constants. Used in the REPL.
    pub fn reset_instructions(&mut self) -> Bytecode {
        let instructions = std::mem::replace(self.current_instructions(), Instructions(Vec::new()));
        let constants = self.constants.clone();
        Bytecode {
            instructions,
            constants,
        }
    }

    pub fn bytecode(mut self) -> Bytecode {
        let top_scope_instructions = self.scopes.pop().map(|scope| scope.instructions);
        Bytecode {
            instructions: top_scope_instructions.unwrap_or_else(|| Instructions(Vec::new())),
            constants: self.constants,
        }
    }

    fn current_instructions(&mut self) -> &mut Instructions {
        // This function panics if the compilation scopes stack is empty
        &mut self
            .scopes
            .last_mut()
            .expect("No compilation scope in stack")
            .instructions
    }

    fn enter_scope(&mut self) {
        let empty_scope = CompilationScope {
            instructions: Instructions(Vec::new()),
        };
        let old_table = std::mem::replace(&mut self.symbol_table, None);
        let new_table = SymbolTable::from_outer(Box::new(old_table.expect("No symbol table")));
        self.symbol_table = Some(new_table);
        self.scopes.push(empty_scope);
    }

    fn pop_scope(&mut self) -> CompilationScope {
        let old_table = std::mem::replace(&mut self.symbol_table, None);
        self.symbol_table = old_table
            .expect("No symbol table")
            .outer
            .map(|outer| *outer);
        self.scopes.pop().expect("No compilation scope in stack")
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn emit(&mut self, op: OpCode, operands: &[usize]) -> usize {
        let ins = make(op, operands);
        self.add_instruction(&*ins)
    }

    fn add_instruction(&mut self, instruction: &[u8]) -> usize {
        let new_instruction_pos = self.current_instructions().0.len();
        self.current_instructions().0.extend_from_slice(instruction);
        new_instruction_pos
    }

    fn change_operand(&mut self, op_pos: usize, new_operand: usize) {
        let op_code = OpCode::from_byte(self.current_instructions().0[op_pos]);
        let new_instruction = make!(op_code, new_operand);
        self.replace_instruction(op_pos, &new_instruction)
    }

    fn replace_instruction(&mut self, pos: usize, new_instruction: &[u8]) {
        for (i, b) in new_instruction.iter().enumerate() {
            self.current_instructions().0[pos + i] = *b;
        }
    }

    pub fn compile_block(&mut self, block: Vec<NodeStatement>) -> MonkeyResult<()> {
        if block.is_empty() {
            // Empty blocks evaluate to `nil`
            self.emit(OpCode::OpNil, &[]);
        } else {
            let last_index = block.len() - 1;
            for (i, statement) in block.into_iter().enumerate() {
                self.compile_statement(statement, i == last_index)?;
            }
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: NodeStatement, last: bool) -> MonkeyResult<()> {
        match statement.statement {
            Statement::ExpressionStatement(exp) => {
                self.compile_expression(*exp)?;
                if !last {
                    self.emit(OpCode::OpPop, &[]);
                }
            }
            Statement::Let(let_statement) => {
                let (name, exp) = *let_statement;
                self.compile_expression(exp)?;
                let symbol = self
                    .symbol_table
                    .as_mut()
                    .expect("No symbol table")
                    .define(name);
                let op = match symbol.scope {
                    SymbolScope::Global => OpCode::OpSetGlobal,
                    SymbolScope::Local => OpCode::OpSetLocal,
                    _ => todo!(),
                };
                let index = symbol.index;
                self.emit(op, &[index]);
                // If the "let" statement is the last in the block, it evaluates to `nil`
                if last {
                    self.emit(OpCode::OpNil, &[]);
                }
            }
            Statement::Return(value) => {
                // If we are at the root compilation scope, we are not in a function context
                if self.scopes.len() == 1 {
                    return Err(MonkeyError::Compiler(statement.position, InvalidReturn));
                }
                self.compile_expression(*value)?;
                self.emit(OpCode::OpReturn, &[]);
            }
        };
        Ok(())
    }

    fn compile_expression(&mut self, expression: NodeExpression) -> MonkeyResult<()> {
        use Token::*;
        match expression.expression {
            Expression::InfixExpression(left, tk, right) => {
                if let Token::LessThan | Token::LessEq = tk {
                    self.compile_expression(*right)?;
                    self.compile_expression(*left)?;
                } else {
                    self.compile_expression(*left)?;
                    self.compile_expression(*right)?;
                }
                match tk {
                    Plus => self.emit(OpCode::OpAdd, &[]),
                    Minus => self.emit(OpCode::OpSub, &[]),
                    Asterisk => self.emit(OpCode::OpMul, &[]),
                    Slash => self.emit(OpCode::OpDiv, &[]),
                    Exponent => self.emit(OpCode::OpExponent, &[]),
                    Modulo => self.emit(OpCode::OpModulo, &[]),
                    Equals => self.emit(OpCode::OpEquals, &[]),
                    NotEquals => self.emit(OpCode::OpNotEquals, &[]),
                    GreaterThan | LessThan => self.emit(OpCode::OpGreaterThan, &[]),
                    GreaterEq | LessEq => self.emit(OpCode::OpGreaterEq, &[]),
                    _ => unreachable!(),
                };
            }
            Expression::PrefixExpression(tk, right) => {
                self.compile_expression(*right)?;
                match tk {
                    Minus => self.emit(OpCode::OpPrefixMinus, &[]),
                    Bang => self.emit(OpCode::OpPrefixNot, &[]),
                    _ => unreachable!(),
                };
            }
            Expression::IntLiteral(i) => {
                let obj = Object::Integer(i);
                let constant_index = self.add_constant(obj);
                self.emit(OpCode::OpConstant, &[constant_index]);
            }
            Expression::Boolean(true) => {
                self.emit(OpCode::OpTrue, &[]);
            }
            Expression::Boolean(false) => {
                self.emit(OpCode::OpFalse, &[]);
            }
            Expression::StringLiteral(s) => {
                let obj = Object::Str(Box::new(s));
                let constant_index = self.add_constant(obj);
                self.emit(OpCode::OpConstant, &[constant_index]);
            }
            Expression::ArrayLiteral(v) => {
                let length = v.len();
                if length > 65536 {
                    return Err(MonkeyError::Compiler(expression.position, LiteralTooBig));
                }
                for expression in v {
                    self.compile_expression(expression)?;
                }
                self.emit(OpCode::OpArray, &[length]);
            }
            Expression::HashLiteral(v) => {
                let length = v.len();
                if length > 65536 {
                    return Err(MonkeyError::Compiler(expression.position, LiteralTooBig));
                }
                for (key, value) in v {
                    self.compile_expression(key)?;
                    self.compile_expression(value)?;
                }
                self.emit(OpCode::OpHash, &[length]);
            }
            Expression::Nil => {
                self.emit(OpCode::OpNil, &[]);
            }
            Expression::IfExpression {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expression(*condition)?;
                // Emit an OpJumpNotTruthy instruction that will eventually point to after the
                // consequence
                let jump_not_truthy_pos = self.emit(OpCode::OpJumpNotTruthy, &[9999]);

                self.compile_block(consequence)?;

                // Emit an OpJump instruction that will eventually point to after the alternative
                let jump_pos = self.emit(OpCode::OpJump, &[9999]);

                // Modify the OpJumpNotTruthy instruction
                let after_consequence = self.current_instructions().0.len();
                self.change_operand(jump_not_truthy_pos, after_consequence);

                self.compile_block(alternative)?;

                // Modify the OpJump instruction
                let after_alternative = self.current_instructions().0.len();
                self.change_operand(jump_pos, after_alternative);
            }
            Expression::Identifier(name) => {
                let symbol = self
                    .symbol_table
                    .as_ref()
                    .expect("No symbol table")
                    .resolve(&name)
                    .ok_or(MonkeyError::Compiler(
                        expression.position,
                        IdenNotFound(name),
                    ))?;
                let op = match symbol.scope {
                    SymbolScope::Builtin => OpCode::OpGetBuiltin,
                    SymbolScope::Global => OpCode::OpGetGlobal,
                    SymbolScope::Local => OpCode::OpGetLocal,
                };
                self.emit(op, &[symbol.index]);
            }
            Expression::IndexExpression(obj, index) => {
                self.compile_expression(*obj)?;
                self.compile_expression(*index)?;
                self.emit(OpCode::OpIndex, &[]);
            }
            Expression::FunctionLiteral { body, parameters } => {
                self.enter_scope();
                let num_params = parameters.len() as u8;
                for param in parameters {
                    self.symbol_table
                        .as_mut()
                        .expect("No symbol table")
                        .define(param);
                }
                self.compile_block(body)?;
                // If the last instruction emitted was not a return instruction, emit one. It's safe
                // to `.unwrap` here because every block is guaranteed to emit at least one
                // instruction.
                if *self.current_instructions().0.last().unwrap() != OpCode::OpReturn as u8 {
                    self.emit(OpCode::OpReturn, &[]);
                }
                let num_locals = self
                    .symbol_table
                    .as_ref()
                    .expect("No symbol table")
                    .num_definitions as u8;
                let instructions = self.pop_scope().instructions;
                let compiled_fn = CompiledFunction {
                    instructions,
                    num_locals,
                    num_params,
                };
                let index = self.add_constant(Object::CompiledFunc(Box::new(compiled_fn)));
                self.emit(OpCode::OpConstant, &[index]);
            }
            Expression::CallExpression {
                function,
                arguments,
            } => {
                let num_args = arguments.len();
                self.compile_expression(*function)?;
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                self.emit(OpCode::OpCall, &[num_args]);
            }
            _ => todo!(),
        }
        Ok(())
    }
}
