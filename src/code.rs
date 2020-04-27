use crate::object::Object;

use std::mem;

pub type Instructions = Vec<u8>;

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpConstant,
}

impl OpCode {
    pub fn operand_widths(&self) -> &'static [usize] {
        match self {
            OpCode::OpConstant => &[2],
        }
    }
}

pub fn make(op: OpCode, operands: &[usize]) -> Box<[u8]> {
    let instruction_len = 1 + op.operand_widths().iter().sum::<usize>();
    assert_eq!(operands.len(), op.operand_widths().len());
    let mut instruction = Vec::with_capacity(instruction_len);
    instruction.push(op as u8);
    for (&operand, width) in operands.iter().zip(op.operand_widths()) {
        dbg!(&(operand as u16).to_be_bytes());
        match width {
            2 => instruction.extend_from_slice(&(operand as u16).to_be_bytes()),
            _ => panic!("unsupported operand width"),
        }
    }
    instruction.into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make() {
        assert_eq!(&[0, 255, 254], &*make(OpCode::OpConstant, &[65534]));
    }
}