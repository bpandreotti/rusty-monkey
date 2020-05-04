#[macro_use]
pub mod code;
pub mod symbol_table;
#[cfg(test)]
mod tests;

use crate::error::*;
use crate::parser::ast::*;
// @PERFORMANCE: The compiler currently uses the same object representation as the interpreter.
// This might not be ideal.
use crate::interpreter::object::Object;
use crate::lexer::token::Token;
use code::*;
use symbol_table::*;

pub struct Compiler {
    instructions: Instructions,
    pub constants: Vec<Object>,
    pub symbol_table: SymbolTable,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions(Vec::new()),
            constants: Vec::new(),
            symbol_table: SymbolTable::new(),
        }
    }

    /// Resets the instructions of the compiler, without changing the constants, and returns a
    /// `Bytecode` containing the old instructions and a clone of the constants. Used in the REPL.
    pub fn reset_instructions(&mut self) -> Bytecode {
        let instructions = std::mem::replace(&mut self.instructions, Instructions(Vec::new()));
        let constants = self.constants.clone();
        Bytecode {
            instructions,
            constants,
        }
    }

    pub fn bytecode(self) -> Bytecode {
        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
        }
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
        let new_instruction_pos = self.instructions.0.len();
        self.instructions.0.extend_from_slice(instruction);
        new_instruction_pos
    }

    fn change_operand(&mut self, op_pos: usize, new_operand: usize) {
        let op_code = OpCode::from_byte(self.instructions.0[op_pos]);
        let new_instruction = make!(op_code, new_operand);
        self.replace_instruction(op_pos, &new_instruction)
    }

    fn replace_instruction(&mut self, pos: usize, new_instruction: &[u8]) {
        for (i, b) in new_instruction.iter().enumerate() {
            self.instructions.0[pos + i] = *b;
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
                Ok(())
            }
            Statement::Let(let_statement) => {
                let (name, exp) = *let_statement;
                self.compile_expression(exp)?;
                let symbol = self.symbol_table.define(name);
                let index = symbol.index;
                self.emit(OpCode::OpSetGlobal, &[index]);
                // If the "let" statement is the last in the block, it evaluates to `nil`
                if last {
                    self.emit(OpCode::OpNil, &[]);
                }
                Ok(())
            }
            _ => todo!(),
        }
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
                let obj = Object::Str(s);
                let constant_index = self.add_constant(obj);
                self.emit(OpCode::OpConstant, &[constant_index]);
            }
            Expression::ArrayLiteral(v) => {
                let length = v.len();
                if length > 65536 {
                    panic!("Array literal too big!") // @TODO: Add proper errors
                }
                for expression in v {
                    self.compile_expression(expression)?;
                }
                self.emit(OpCode::OpArray, &[length]);
            }
            Expression::HashLiteral(v) => {
                let length = v.len();
                if length > 65536 {
                    panic!("Hash literal too big!") // @TODO: Add proper errors
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
                let after_consequence = self.instructions.0.len();
                self.change_operand(jump_not_truthy_pos, after_consequence);

                self.compile_block(alternative)?;

                // Modify the OpJump instruction
                let after_alternative = self.instructions.0.len();
                self.change_operand(jump_pos, after_alternative);
            }
            Expression::Identifier(name) => {
                // @TODO: Add proper erros
                let index = self.symbol_table.resolve(&name).unwrap().index;
                self.emit(OpCode::OpGetGlobal, &[index]);
            }
            _ => todo!(),
        }
        Ok(())
    }
}
