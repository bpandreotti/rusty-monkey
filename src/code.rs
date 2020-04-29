use crate::object::Object;

use std::convert::TryInto;
use std::fmt;

#[derive(PartialEq)]
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
            writeln!(f, "")?;
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
    OpGreaterEq, // @TODO: Maybe this should be implemented in terms of "!" and "<"?
    OpPrefixMinus,
    OpPrefixNot,
    OpJumpNotTruthy,
    OpJump,
}

impl OpCode {
    pub fn operand_widths(&self) -> &'static [usize] {
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
        }
    }

    pub fn from_byte(byte: u8) -> OpCode {
        // @TODO: Write a macro to automatically implement this
        // @PERFORMANCE: mem::transmute would be faster, but horribly unsafe
        match byte {
            0x00 => OpCode::OpConstant,
            0x01 => OpCode::OpPop,
            0x02 => OpCode::OpAdd,
            0x03 => OpCode::OpSub,
            0x04 => OpCode::OpMul,
            0x05 => OpCode::OpDiv,
            0x06 => OpCode::OpExponent,
            0x07 => OpCode::OpModulo,
            0x08 => OpCode::OpTrue,
            0x09 => OpCode::OpFalse,
            0x0a => OpCode::OpEquals,
            0x0b => OpCode::OpNotEquals,
            0x0c => OpCode::OpGreaterThan,
            0x0d => OpCode::OpGreaterEq,
            0x0e => OpCode::OpPrefixMinus,
            0x0f => OpCode::OpPrefixNot,
            0x10 => OpCode::OpJumpNotTruthy,
            0x11 => OpCode::OpJump,
            _ => panic!("byte does not represent valid opcode")
        }
    }
}

#[macro_export]
macro_rules! make {
    ($op:expr $(,$rand:expr )*) => {
        crate::code::make($op, &[ $( $rand ),*])
    };
}

pub fn make(op: OpCode, operands: &[usize]) -> Box<[u8]> {
    let instruction_len = 1 + op.operand_widths().iter().sum::<usize>();
    assert_eq!(operands.len(), op.operand_widths().len());
    let mut instruction = Vec::with_capacity(instruction_len);
    instruction.push(op as u8);
    for (&operand, width) in operands.iter().zip(op.operand_widths()) {
        match width {
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
            2 => {
                let operand = read_u16(&instructions[offset..]) as usize;
                operands.push(operand);
            }
            _ => panic!("unsupported operand width")
        }
        offset += width;
    }
    (operands, offset)
}

pub fn read_u16(instructions: &[u8]) -> u16 {
    u16::from_be_bytes(instructions[..2].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make() {
        assert_eq!(&[OpCode::OpConstant as u8, 255, 254], &*make!(OpCode::OpConstant, 65534));
        assert_eq!(&[OpCode::OpAdd as u8], &*make!(OpCode::OpAdd));
    }

    #[test]
    fn test_instruction_printing() {
        let input = Instructions([
            make!(OpCode::OpAdd),
            make!(OpCode::OpConstant, 2),
            make!(OpCode::OpConstant, 65535),
        ].concat());
        let expected = "\
        0000 OpAdd\n\
        0001 OpConstant 2\n\
        0004 OpConstant 65535\n\
        ";
        assert_eq!(expected, format!("{}", input));
    }
}
