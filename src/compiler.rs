use crate::ast::*;
use crate::code::*;
use crate::error::*;
use crate::object::*;
use crate::token::Token;

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions(Vec::new()),
            constants: Vec::new(),
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

    pub fn compile_block(&mut self, block: Vec<NodeStatement>) -> Result<(), MonkeyError> {
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

    fn compile_statement(&mut self, statement: NodeStatement, last: bool) -> Result<(), MonkeyError> {
        match statement.statement {
            Statement::ExpressionStatement(exp) => {
                self.compile_expression(*exp)?;
                if !last {
                    self.emit(OpCode::OpPop, &[]);
                }
                Ok(())
            }
            _ => todo!(),
        }
    }

    fn compile_expression(&mut self, expression: NodeExpression) -> Result<(), MonkeyError> {
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
            },
            Expression::Boolean(true) => { self.emit(OpCode::OpTrue, &[]); }
            Expression::Boolean(false) => { self.emit(OpCode::OpFalse, &[]); }
            Expression::Nil => { self.emit(OpCode::OpNil, &[]); }
            Expression::IfExpression { condition, consequence, alternative } => {
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
            _ => todo!(),
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_integer_arithmetic() {
        test_utils::assert_compile("1 + 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpAdd),
            ].concat())
        );
        test_utils::assert_compile("1; 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 1),
            ].concat())
        );
        test_utils::assert_compile("1 * 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpMul),
            ].concat())
        );
        test_utils::assert_compile("-1",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpPrefixMinus),
            ].concat())
        );
    }

    #[test]
    fn test_boolean_expressions() {
        test_utils::assert_compile("true",
            Instructions([
                make!(OpCode::OpTrue),
            ].concat())
        );
        test_utils::assert_compile("false",
            Instructions([
                make!(OpCode::OpFalse),
            ].concat())
        );
        test_utils::assert_compile("1 > 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpGreaterThan),
            ].concat())
        );
        test_utils::assert_compile("1 < 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpGreaterThan),
            ].concat())
        );
        test_utils::assert_compile("1 == 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpEquals),
            ].concat())
        );
        test_utils::assert_compile("1 != 2",
            Instructions([
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpNotEquals),
            ].concat())
        );
        test_utils::assert_compile("!true",
            Instructions([
                make!(OpCode::OpTrue),
                make!(OpCode::OpPrefixNot),
            ].concat())
        );
    }

    #[test]
    fn test_conditionals() {
        test_utils::assert_compile("if true { 10 }; 3333", 
            Instructions([
                make!(OpCode::OpTrue),
                make!(OpCode::OpJumpNotTruthy, 10),
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpJump, 11),
                make!(OpCode::OpNil),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 1),
            ].concat())
        );
        test_utils::assert_compile("if true { 10 } else { 20 }; 3333", 
            Instructions([
                make!(OpCode::OpTrue),
                make!(OpCode::OpJumpNotTruthy, 10),
                make!(OpCode::OpConstant, 0),
                make!(OpCode::OpJump, 13),
                make!(OpCode::OpConstant, 1),
                make!(OpCode::OpPop),
                make!(OpCode::OpConstant, 2),
            ].concat())
        );
    }
}
