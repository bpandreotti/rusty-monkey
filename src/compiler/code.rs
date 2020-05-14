use crate::object::Object;
use crate::lexer::token;

use std::convert::TryInto;
use std::fmt;
use std::mem;

#[derive(Clone, PartialEq)]
pub struct Instructions(pub Vec<u8>);

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        let mut byte_counter = 0;
        while byte_counter < self.0.len() {
            let op = OpCode::from_byte(self.0[byte_counter]);
            write!(f, "{:04} {:?}", byte_counter, op)?;
            let (rands, bytes_read) = read_operands(op, &self.0[byte_counter + 1..]);
            for r in rands {
                write!(f, " {}", r)?;
            }
            writeln!(f)?;
            byte_counter += 1 + bytes_read;
        }
        write!(f, "")
    }
}

impl fmt::Debug for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Instructions:\n{}", self)
    }
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpConstant,
    OpPop,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpExponent,
    OpModulo,
    OpTrue,
    OpFalse,
    OpEquals,
    OpNotEquals,
    OpGreaterThan,
    OpGreaterEq, // @PERFORMANCE: Maybe this should be implemented in terms of "!" and "<"?
    OpPrefixMinus,
    OpPrefixNot,
    OpJumpNotTruthy,
    OpJump,
    OpNil,
    OpGetGlobal,
    OpSetGlobal,
    OpArray,
    OpHash,
    OpIndex,
    OpCall,
    OpReturn,
    OpGetLocal,
    OpSetLocal,
}

impl OpCode {
    pub fn operand_widths(self) -> &'static [usize] {
        match self {
            OpCode::OpConstant => &[2],
            OpCode::OpPop => &[],
            OpCode::OpAdd => &[],
            OpCode::OpSub => &[],
            OpCode::OpMul => &[],
            OpCode::OpDiv => &[],
            OpCode::OpExponent => &[],
            OpCode::OpModulo => &[],
            OpCode::OpTrue => &[],
            OpCode::OpFalse => &[],
            OpCode::OpEquals => &[],
            OpCode::OpNotEquals => &[],
            OpCode::OpGreaterThan => &[],
            OpCode::OpGreaterEq => &[],
            OpCode::OpPrefixMinus => &[],
            OpCode::OpPrefixNot => &[],
            OpCode::OpJumpNotTruthy => &[2],
            OpCode::OpJump => &[2],
            OpCode::OpNil => &[],
            OpCode::OpGetGlobal => &[2],
            OpCode::OpSetGlobal => &[2],
            OpCode::OpArray => &[2],
            OpCode::OpHash => &[2],
            OpCode::OpIndex => &[],
            OpCode::OpCall => &[1],
            OpCode::OpReturn => &[],
            OpCode::OpGetLocal => &[1],
            OpCode::OpSetLocal => &[1],
        }
    }

    pub fn from_byte(byte: u8) -> OpCode {
        // Safety: `OpCode` is #[repr(u8)], so as long as `byte` represents a valid enum
        // variant, this transmute will be safe. We make sure of that by asserting that `byte`
        // is no greater than the last variant.
        assert!(byte <= (OpCode::OpSetLocal as u8), "byte does not represent valid opcode");
        unsafe { mem::transmute(byte) }
    }

    pub fn equivalent_token(self) -> Option<token::Token> {
        match self {
            OpCode::OpAdd => Some(token::Token::Plus),
            OpCode::OpSub => Some(token::Token::Minus),
            OpCode::OpMul => Some(token::Token::Asterisk),
            OpCode::OpDiv => Some(token::Token::Slash),
            OpCode::OpExponent => Some(token::Token::Exponent),
            OpCode::OpModulo => Some(token::Token::Modulo),
            OpCode::OpEquals => Some(token::Token::Equals),
            OpCode::OpNotEquals => Some(token::Token::NotEquals),
            OpCode::OpGreaterThan => Some(token::Token::GreaterThan),
            OpCode::OpGreaterEq => Some(token::Token::GreaterEq),
            OpCode::OpPrefixMinus => Some(token::Token::Minus),
            OpCode::OpPrefixNot => Some(token::Token::Bang),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! make {
    ($op:expr $(,$rand:expr )*) => {
        crate::compiler::code::make($op, &[ $( $rand ),*])
    };
}

pub fn make(op: OpCode, operands: &[usize]) -> Box<[u8]> {
    let instruction_len = 1 + op.operand_widths().iter().sum::<usize>();
    assert_eq!(operands.len(), op.operand_widths().len());
    let mut instruction = Vec::with_capacity(instruction_len);
    instruction.push(op as u8);
    for (&operand, width) in operands.iter().zip(op.operand_widths()) {
        match width {
            1 => instruction.push(operand as u8),
            2 => instruction.extend_from_slice(&(operand as u16).to_be_bytes()),
            _ => panic!("unsupported operand width"),
        }
    }
    instruction.into_boxed_slice()
}

pub fn read_operands(op: OpCode, instructions: &[u8]) -> (Vec<usize>, usize) {
    // @PERFORMANCE: Maybe taking a &mut &[u8] would be faster?
    let mut operands = Vec::with_capacity(op.operand_widths().len());
    let mut offset = 0;
    for width in op.operand_widths() {
        match width {
            1 => operands.push(instructions[offset] as usize),
            2 => {
                let operand = read_u16(&instructions[offset..]) as usize;
                operands.push(operand);
            }
            _ => panic!("unsupported operand width"),
        }
        offset += width;
    }
    (operands, offset)
}

pub fn read_u16(instructions: &[u8]) -> u16 {
    u16::from_be_bytes(instructions[..2].try_into().unwrap())
}
