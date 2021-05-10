/// The set of the VM's instruction codes.
#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    OpConstant(usize),
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNegate,
    OpReturn,
}

#[derive(Debug, Clone, Copy)]
pub struct Value(pub f64);

/// A chunk of bytecode.
pub struct Chunk {
    /// Holds the Chunk's bytecode.
    pub code: Vec<OpCode>,
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
            .for_each(|(i, _)| self.disassemble_instruction(i));
        println!("== /{} ==\n", name);
    }

    pub fn disassemble_instruction(&self, index: usize) {
        // print!("instruction: {:?}\t", index);
        print!("{:?} ", index);
        if index > 0 && self.lines[index] == self.lines[index - 1] {
            print!("      |\t\t");
        } else {
            print!("line: {:?}\t\t", self.lines[index]);
        }

        let instruction = self.code[index];
        match instruction {
            OpCode::OpConstant(idx) => {
                let Value(constant) = self.constants[idx];
                println!("{:?}\tindex: {:?}\tvalue: {:?}", instruction, idx, constant);
            }
            OpCode::OpNegate
            | OpCode::OpAdd
            | OpCode::OpSubtract
            | OpCode::OpMultiply
            | OpCode::OpDivide
            | OpCode::OpReturn => println!("{:?}", instruction),
        }
    }
}
