use crate::chunk::{Chunk, OpCode, Value};

/// A virtual machine that interprets chunks of bytecode.
pub struct VM<'a> {
    /// The chunk of bytecode currently being interpreted.
    chunk: &'a Chunk,
    /// The instruction pointer.
    /// It is the index of the instruction about to be executed,
    /// in the Chunk's code array.
    ip: usize,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl<'a> VM<'a> {
    pub fn init(chunk: &'a Chunk) -> VM {
        VM { chunk, ip: 0 }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = self.read_byte();
            match instruction {
                OpCode::OpReturn => return InterpretResult::InterpretOk,
                OpCode::OpConstant => {
                    let constant: Value =
                }
                _ => return InterpretResult::InterpretOk,
            }
        }

        InterpretResult::InterpretOk
    }

    // fn read_byte(&mut self) -> OpCode {
    //     let instruction = self.chunk.read_code(self.ip);
    //     self.ip += 1;
    //     instruction
    // }

    // fn read_constant(&mut self) -> Value {
    //     if let OpCode::OpOperand(constant_index) = self.read_byte() {
    //         self.chunk.read_constant(constant_index)
    //     }
    // }
}
