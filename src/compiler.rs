use crate::ast::*;
use crate::code::*;
use crate::error::*;
use crate::object::*;

struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
}

impl Compiler {
    fn new() -> Compiler {
        Compiler {
            instructions: Instructions(Vec::new()),
            constants: Vec::new(),
        }
    }
   
    fn bytecode(self) -> Bytecode {
        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
        }
    }

    fn compile(&self, program: Vec<NodeStatement>) -> Result<(), MonkeyError> {
        // @WIP
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn parse(input: &str) -> Vec<NodeStatement> {
        let lex = Lexer::from_string(input.into()).unwrap();
        let mut pars = Parser::new(lex).unwrap();
        pars.parse_program().unwrap()
    }

    // @TODO: Also compare constants
    fn assert_compile(input: &str, expected: Instructions) {
        let program = parse(input);
        let comp = Compiler::new();
        comp.compile(program).unwrap();
        assert_eq!(expected, comp.bytecode().instructions)
    }

    #[test]
    fn test_integer_arithmetic() {
        assert_compile("1 + 2", Instructions([
            make(OpCode::OpConstant, &[0]),
            make(OpCode::OpConstant, &[1]),
        ].concat()))
    }
}
