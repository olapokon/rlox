use crate::value::Value;

/// The set of the VM's instruction codes.
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    OpConstant(usize),
    OpNil,
    OpTrue,
    OpDefineGlobal(usize),
    OpEqual,
    OpFalse,
    OpGetGlobal(usize),
    OpGreater,
    OpLess,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpPop,
    OpNot,
    OpNegate,
    OpPrint,
    OpReturn,
}

/// A chunk of bytecode.
pub struct Chunk {
    /// Holds the Chunk's bytecode.
    pub bytecode: Vec<Instruction>,
    /// Exactly parallels the bytecode array.
    /// Holds the line number of each corresponding OpCode.
    pub lines: Vec<i32>,
    /// Holds the Chunk's constant values.
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn init() -> Chunk {
        Chunk {
            bytecode: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Adds an [OpCode] to the [Chunk]'s code array.
    pub fn write(&mut self, instruction: Instruction, line: i32) {
        self.bytecode.push(instruction);
        self.lines.push(line);
    }

    pub fn read_code(&self, index: usize) -> Instruction {
        self.bytecode[index]
    }

    pub fn read_constant(&self, index: usize) -> &Value {
        // TODO: refactor clone();
        // self.constants[index].clone()
        &self.constants[index]
    }

    /// Adds a constant to the [Chunk]'s [ValueArray] and returns the index.
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        self.bytecode
            .iter()
            .enumerate()
            .for_each(|(i, _)| self.disassemble_instruction(i));
        println!("== /{} ==\n", name);
    }

    // TODO: implement Display for [Instruction] instead
    pub fn disassemble_instruction(&self, index: usize) {
        print!("{:?} ", index);
        if index > 0 && self.lines[index] == self.lines[index - 1] {
            print!("      |\t\t");
        } else {
            print!("line: {:?}\t\t", self.lines[index]);
        }

        let instruction = self.bytecode[index];
        match instruction {
            Instruction::OpConstant(idx)
            | Instruction::OpDefineGlobal(idx)
            | Instruction::OpGetGlobal(idx) => {
                let constant = &self.constants[idx];
                println!("{:?}    \tvalue: {:?}", instruction, constant);
            }
            Instruction::OpNegate
            | Instruction::OpEqual
            | Instruction::OpGreater
            | Instruction::OpLess
            | Instruction::OpAdd
            | Instruction::OpSubtract
            | Instruction::OpMultiply
            | Instruction::OpDivide
            | Instruction::OpFalse
            | Instruction::OpNil
            | Instruction::OpTrue
            | Instruction::OpNot
            | Instruction::OpPop
            | Instruction::OpPrint
            | Instruction::OpReturn => println!("{:?}", instruction),
        }
    }
}
