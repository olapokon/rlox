use std::usize;

/// The set of the VM's instruction codes, along with an OpOperand(u8)
/// for instructions that have operands.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpConstant(usize),
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
        println!("== {} ==", name);
        self.code
            .iter()
            .enumerate()
            .for_each(|(i, &c)| self.disassemble_instruction(i, c));
    }

    fn disassemble_instruction(&self, index: usize, instruction: OpCode) {
        print!("instruction: {:?}\t", index);
        if index > 0 && self.lines[index] == self.lines[index - 1] {
            print!("      |\t\t");
        } else {
            print!("line: {:?}\t\t", self.lines[index]);
        }

        match instruction {
            OpCode::OpConstant(idx) => {
                let Value(constant) = self.constants[idx];
                println!("{:?}\tindex: {:?}\tvalue: {:?}", instruction, idx, constant);
            }
            OpCode::OpReturn => println!("{:?}", instruction),
        }
    }
}
