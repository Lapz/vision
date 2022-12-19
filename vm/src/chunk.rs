use std::ops::Index;

use crate::op::{self, Op};
use crate::value::Value;
use crate::vm::print_value;
#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
            lines: vec![],
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==\n", name);

        let mut i = 0;

        while i < self.code.len() {
            i = self.disassemble_instruction(i);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines.get(offset) == self.lines.get(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines.get(offset).unwrap());
        }

        let instruction = self.code[offset];

        unsafe {
            match std::mem::transmute(instruction) {
                Op::RETURN => self.simple_instruction("OP::RETURN", offset),
                Op::CONSTANT => self.constant_instruction("OP::CONSTANT", offset),
                Op::NEGATE => self.simple_instruction("OP::NEGATE", offset),
                Op::ADD => self.simple_instruction("OP::ADD", offset),
                Op::SUBTRACT => self.simple_instruction("OP::SUBTRACT", offset),
                Op::MULTIPLY => self.simple_instruction("OP::MULTIPLY", offset),
                Op::DIVIDE => self.simple_instruction("OP::DIVIDE", offset),
                Op::NIL => self.simple_instruction("OP::NIL", offset),
                Op::TRUE => self.simple_instruction("OP::TRUE", offset),
                Op::FALSE => self.simple_instruction("OP::FALSE", offset),
                Op::NOT => self.simple_instruction("OP::NOT", offset),
                Op::EQUAL => self.simple_instruction("OP::EQUAL", offset),
                Op::GREATER => self.simple_instruction("OP::GREATER", offset),
                Op::LESS => self.simple_instruction("OP::LESS", offset),
                Op::PRINT => self.simple_instruction("OP::PRINT", offset),
                Op::POP => self.simple_instruction("OP::POP", offset),
                Op::DEFINE_GLOBAL => self.constant_instruction("OP::DEFINE_GLOBAL", offset),
                Op::GET_GLOBAL => self.constant_instruction("OP::GET_GLOBAL", offset),
                Op::SET_GLOBAL => self.constant_instruction("OP::SET_GLOBAL", offset),
                Op::GET_LOCAL => self.byte_instruction("OP::GET_LOCAL", offset),
                Op::SET_LOCAL => self.byte_instruction("OP::GET_LOCAL", offset),
                Op::JUMP => self.jump_instruction("op::JUMP", 1, offset),
                Op::JUMP_IF_FALSE => self.jump_instruction("op::JUMP_IF_FALSE", 1, offset),
                Op::LOOP => self.jump_instruction("OP::LOOP", -1, offset),
                Op::CALL => self.byte_instruction("OP::CALL", offset),
                Op::CLOSURE => {
                    let constant = self.code[offset + 1];
                    print!("{:16}{:4} '", "OP_CLOSURE", constant);
                    print_value(self.constants[constant as usize]);
                    println!();

                    offset + 2
                }
                _ => {
                    println!("Unknown opcode {}", instruction);
                    offset + 1
                }
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    pub fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{:16}{:4} '", name, constant);
        print_value(self.constants[constant as usize]);
        println!("'");
        offset + 2
    }

    pub(crate) fn byte_instruction(&self, arg: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:16}{:4} ", arg, slot);
        offset + 2
    }

    pub(crate) fn jump_instruction(&self, arg: &str, sign: isize, offset: usize) -> usize {
        let mut jump = ((self.code[offset + 1] as u16) << 8) as usize;
        jump |= self.code[offset + 2] as usize;
        println!(
            "{:16} {:4} -> {} ",
            arg,
            offset,
            offset as isize + 3 + (sign * jump as isize)
        );
        offset + 3
    }
}

impl Index<usize> for Chunk {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}
