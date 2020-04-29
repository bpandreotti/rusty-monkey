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
        test_utils::assert_compile("-1",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpPrefixMinus, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
    }

    #[test]
    fn test_boolean_expressions() {
        test_utils::assert_compile("true",
            Instructions([
                make(OpCode::OpTrue, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("false",
            Instructions([
                make(OpCode::OpFalse, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1 > 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpGreaterThan, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1 < 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpGreaterThan, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1 == 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpEquals, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("1 != 2",
            Instructions([
                make(OpCode::OpConstant, &[0]),
                make(OpCode::OpConstant, &[1]),
                make(OpCode::OpNotEquals, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
        test_utils::assert_compile("!true",
            Instructions([
                make(OpCode::OpTrue, &[]),
                make(OpCode::OpPrefixNot, &[]),
                make(OpCode::OpPop, &[]),
            ].concat())
        );
    }
}
