use crate::compiler::*;
use crate::{
    chunk::{Chunk, Instruction},
    value::Value,
};

const STACK_MAX: usize = 256;

/// A virtual machine that interprets chunks of bytecode.
pub struct VM {
    /// The instruction pointer.
    /// It is the index of the instruction about to be executed, in the Chunk's code array.
    ip: usize,
    /// The VM's stack.
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl VM {
    pub fn interpret(source: String) -> InterpretResult {
        let r = match Compiler::compile(source) {
            Ok(r) => r,
            Err(_) => return InterpretResult::InterpretCompileError,
        };

        let mut vm = VM::init();
        vm.run(r)
    }

    fn init() -> VM {
        VM {
            ip: 0,
            stack: [Value::Number(0.0); STACK_MAX],
            stack_top: 0,
        }
    }

    pub fn reset(&mut self) {
        self.stack_top = 0;
    }

    fn run(&mut self, chunk: Chunk) -> InterpretResult {
        while self.ip < chunk.bytecode.len() {
            //
            // TODO: conditional compilation
            // if DEBUG_TRACE_EXECUTION
            for i in 0..self.stack_top {
                print!("[{:?}]", self.stack[i]);
            }
            println!();
            chunk.disassemble_instruction(self.ip);
            // endif
            //
            //
            let instruction = self.read_instruction(&chunk);
            match instruction {
                Instruction::OpReturn => {
                    let return_val = self.pop_from_stack();
                    println!("{:?}", return_val);
                    return InterpretResult::InterpretOk;
                }
                Instruction::OpNegate => match self.pop_from_stack() {
                    Value::Number(val) => self.push_to_stack(Value::Number(-val)),
                    _ => return InterpretResult::InterpretRuntimeError,
                },
                // Instruction::OpAdd => {
                //     let Value::Number(operand_2) = self.pop_from_stack();
                //     let Value::Number(operand_1) = self.pop_from_stack();
                //     self.push_to_stack(Value::Number(operand_1 + operand_2));
                // }
                // Instruction::OpSubtract => {
                //     let Value::Number(operand_2) = self.pop_from_stack();
                //     let Value::Number(operand_1) = self.pop_from_stack();
                //     self.push_to_stack(Value::Number(operand_1 - operand_2));
                // }
                // Instruction::OpMultiply => {
                //     let Value::Number(operand_2) = self.pop_from_stack();
                //     let Value::Number(operand_1) = self.pop_from_stack();
                //     self.push_to_stack(Value::Number(operand_1 * operand_2));
                // }
                // Instruction::OpDivide => {
                //     let Value::Number(operand_2) = self.pop_from_stack();
                //     let Value::Number(operand_1) = self.pop_from_stack();
                //     self.push_to_stack(Value::Number(operand_1 / operand_2));
                // }
                Instruction::OpAdd
                | Instruction::OpSubtract
                | Instruction::OpMultiply
                | Instruction::OpDivide => {
                    let operand_2 = if let Value::Number(operand_2) = self.pop_from_stack() {
                        operand_2
                    } else {
                        return InterpretResult::InterpretRuntimeError;
                    };
                    let operand_1 = if let Value::Number(operand_1) = self.pop_from_stack() {
                        operand_1
                    } else {
                        return InterpretResult::InterpretRuntimeError;
                    };
                    match instruction {
                        Instruction::OpAdd => {
                            self.push_to_stack(Value::Number(operand_1 + operand_2))
                        }
                        Instruction::OpSubtract => {
                            self.push_to_stack(Value::Number(operand_1 - operand_2))
                        }
                        Instruction::OpMultiply => {
                            self.push_to_stack(Value::Number(operand_1 * operand_2))
                        }
                        Instruction::OpDivide => {
                            self.push_to_stack(Value::Number(operand_1 / operand_2))
                        }
                        _ => return InterpretResult::InterpretRuntimeError,
                    }
                }
                Instruction::OpConstant(idx) => {
                    let constant: Value = chunk.read_constant(idx);
                    self.push_to_stack(constant);
                }
            }
        }

        // If there has been no return up to this point, it is an error.
        InterpretResult::InterpretRuntimeError
    }

    fn push_to_stack(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop_from_stack(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    fn read_instruction(&mut self, chunk: &Chunk) -> Instruction {
        let instruction = chunk.read_code(self.ip);
        self.ip += 1;
        instruction
    }
}
