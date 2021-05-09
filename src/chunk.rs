use std::usize;

#[derive(Debug)]
pub enum OpCode {
    OpArgument(u8),

    // operation codes
    OpReturn,
}

pub struct Chunk {
    // count: i32,
    // capacity: i32,
    code: Vec<OpCode>,
}

impl Chunk {
    pub fn init() -> Chunk {
        Chunk { code: Vec::new() }
    }

    pub fn write(&mut self, code: OpCode) {
        self.code.push(code);
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        // self.code.iter().for_each(|c| println!("{:?}", c));
        let mut offset: usize = 0;
        self.code
            .iter()
            .for_each(|_c| offset = self.disassemble_instruction(offset));
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:4} ", offset);

        let instruction = &self.code[offset];
        match instruction {
            OpCode::OpReturn => return Chunk::simple_instruction(instruction, offset),
            _ => Chunk::unknown_opcode(instruction, offset),
        }
    }

    fn unknown_opcode(instruction: &OpCode, offset: usize) -> usize {
        println!("Unknown opcode {:?}", instruction);
        offset + 1
    }

    fn simple_instruction(instruction: &OpCode, offset: usize) -> usize {
        println!("{:?}", instruction);
        offset + 1
    }
}
