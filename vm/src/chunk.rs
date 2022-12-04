use std::ops::Index;

use crate::op;
use crate::value::Value;
use crate::vm::print_value;

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

        match instruction {
            op::RETURN => self.simple_instruction("OP::RETURN", offset),
            op::CONSTANT => self.constant_instruction("OP::CONSTANT", offset),
            op::NEGATE => self.simple_instruction("OP::NEGATE", offset),
            op::ADD => self.simple_instruction("OP::ADD", offset),
            op::SUBTRACT => self.simple_instruction("OP::SUBTRACT", offset),
            op::MULTIPLY => self.simple_instruction("OP::MULTIPLY", offset),
            op::DIVIDE => self.simple_instruction("OP::DIVIDE", offset),
            op::NIL => self.simple_instruction("OP::NIL", offset),
            op::TRUE => self.simple_instruction("OP::TRUE", offset),
            op::FALSE => self.simple_instruction("OP::FALSE", offset),
            op::NOT => self.simple_instruction("OP::NOT", offset),
            op::EQUAL => self.simple_instruction("OP::EQUAL", offset),
            op::GREATER => self.simple_instruction("OP::GREATER", offset),
            op::LESS => self.simple_instruction("OP::LESS", offset),
            _ => {
                println!("Unknown opcode {}", instruction);
                offset + 1
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
}

impl Index<usize> for Chunk {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}
