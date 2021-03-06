use crate::value::value::Value;

/// The set of the VM's instruction codes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    /// The number of arguments with which the function is being called.
    OpCall(usize),
    /// The index of the constant in the [Chunk]'s constants array.
    OpConstant(usize),
    OpNil,
    OpTrue,
    /// The index of the variable name in the [Chunk]'s constants array.
    OpDefineGlobal(usize),
    OpEqual,
    OpFalse,
    /// The index of the variable name in the [Chunk]'s constants array.
    OpGetGlobal(usize),
    /// The index of the variable name in the [Chunk]'s constants array.
    OpSetGlobal(usize),
    /// The index of the variable in the [CallFrame]'s part of the stack.
    OpGetLocal(usize),
    /// The index of the variable in the [CallFrame]'s part of the stack.
    OpSetLocal(usize),
    OpGreater,
    /// The offset used to calculate the bytecode instruction to jump to.
    OpJump(usize),
    /// The offset used to calculate the bytecode instruction to jump to.
    OpJumpIfFalse(usize),
    OpLess,
    /// The offset used to calculate the bytecode instruction to jump to.
    OpLoop(usize),
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
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Holds the Chunk's bytecode.
    pub bytecode: Vec<Instruction>,
    /// Holds the line number of each corresponding OpCode.
    ///
    /// Exactly parallels the bytecode array.
    pub lines: Vec<i32>,
    /// Holds the Chunk's constant values.
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
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
            | Instruction::OpGetGlobal(idx)
            | Instruction::OpSetGlobal(idx) => {
                let constant = &self.constants[idx];
                if let Value::Function(f) = constant {
                    println!("{:?}    \tvalue: <fn {}>", instruction, f.name);
                } else {
                    println!("{:?}    \tvalue: {:?}", instruction, constant);
                }
            }
            Instruction::OpCall(_) => {
                println!("{:?}", instruction);
            }
            // Locals have are 1 ahead, because of the 0 slot being reserved for the function.
            // Instruction::OpSetLocal(idx) | Instruction::OpGetLocal(idx) => {
            //     let constant = &self.constants[idx - 1];
            //     println!("{:?}    \tvalue: {:?}", instruction, constant);
            // }
            Instruction::OpJumpIfFalse(val)
            | Instruction::OpJump(val)
            | Instruction::OpLoop(val)
            | Instruction::OpSetLocal(val)
            | Instruction::OpGetLocal(val) => {
                println!("{:?}    \tvalue: {:?}", instruction, val);
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
