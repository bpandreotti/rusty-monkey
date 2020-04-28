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

    pub fn compile_program(&mut self, program: Vec<NodeStatement>) -> Result<(), MonkeyError> {
        for statement in program {
            self.compile_statement(statement)?;
        }
        Ok(())
    }

    fn compile_statement(&mut self, statement: NodeStatement) -> Result<(), MonkeyError> {
        match statement.statement {
            Statement::ExpressionStatement(exp) => {
                self.compile_expression(*exp)?;
                self.emit(OpCode::OpPop, &[]);
                Ok(())
            }
            _ => todo!(),
        }
    }

    fn compile_expression(&mut self, expression: NodeExpression) -> Result<(), MonkeyError> {
        match expression.expression {
            Expression::InfixExpression(left, tk, right) => {
                self.compile_expression(*left)?;
                self.compile_expression(*right)?;
                match tk {
                    Token::Plus => self.emit(OpCode::OpAdd, &[]),
                    Token::Minus => self.emit(OpCode::OpSub, &[]),
                    Token::Asterisk => self.emit(OpCode::OpMul, &[]),
                    Token::Slash => self.emit(OpCode::OpDiv, &[]),
                    Token::Exponent => self.emit(OpCode::OpExponent, &[]),
                    Token::Modulo => self.emit(OpCode::OpModulo, &[]),
                    _ => todo!(), // @WIP
                };
            }
            Expression::IntLiteral(i) => {
                let obj = Object::Integer(i);
                let constant_index = self.add_constant(obj);
                self.emit(OpCode::OpConstant, &[constant_index]);
            },
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
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpAdd, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1; 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpPop, &[]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1 * 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpMul, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
    }
}
