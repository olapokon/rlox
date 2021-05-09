use std::usize;

/// The set of the VM's instruction codes, along with an OpOperand(u8)
/// for instructions that have operands.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpOperand(usize),

    OpConstant,
    OpReturn,
}

#[derive(Debug, Clone, Copy)]
pub struct Value(pub f64);

/// A chunk of bytecode.
pub struct Chunk {
    /// Holds the Chunk's bytecode.
    code: Vec<OpCode>,
    /// Holds the Chunk's constant values.
    constants: Vec<Value>,
    /// Exactly parallels the bytecode array.
    /// Holds the line number of each corresponding OpCode.
    lines: Vec<i32>,
}

impl Chunk {
    pub fn init() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Adds an [OpCode] to the [Chunk]'s code array.
    pub fn write(&mut self, code: OpCode, line: i32) {
        self.code.push(code);
        self.lines.push(line);
    }

    pub fn read_code(&self, index: usize) -> OpCode {
        self.code[index]
    }

    pub fn read_constant(&self, index: usize) -> Value {
        self.constants[index]
    }

    /// Adds a constant to the [Chunk]'s [ValueArray] and returns the index.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        // println!("chunk code: {:?}", self.code);
        // println!("chunk lines: {:?}", self.lines);
        // println!("chunk constants: {:?}", self.constants);
        println!("== {} ==", name);
        // self.code.iter().for_each(|c| println!("{:?}", c));
        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:4} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = &self.code[offset];
        match instruction {
            OpCode::OpConstant => return self.constant_instruction(instruction, offset),
            OpCode::OpReturn => return Chunk::simple_instruction(instruction, offset),
            OpCode::OpOperand(_) => return Chunk::invalid_operand(instruction, offset),
        }
    }

    fn invalid_operand(instruction: &OpCode, offset: usize) -> usize {
        println!("Invalid operand {:?}", instruction);
        offset + 1
    }

    fn simple_instruction(instruction: &OpCode, offset: usize) -> usize {
        println!("{:?}", instruction);
        offset + 1
    }

    fn constant_instruction(&self, instruction: &OpCode, offset: usize) -> usize {
        let constant_index = self.code[offset + 1];
        if let OpCode::OpOperand(constant_index) = constant_index {
            let Value(constant) = self.constants[constant_index];
            println!(
                "{:16?}\tindex: {:?}\tvalue: {:?}",
                instruction, constant_index, constant
            );
        } else {
            Chunk::invalid_operand(instruction, offset);
        }
        offset + 2
    }
}
