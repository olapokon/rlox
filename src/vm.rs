use crate::chunk::{Chunk, OpCode, Value};
use crate::compiler::*;

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
            stack: [Value(0.0); STACK_MAX],
            stack_top: 0,
        }
    }

    pub fn reset(&mut self) {
        self.stack_top = 0;
    }

    fn run(&mut self, chunk: Chunk) -> InterpretResult {
        while self.ip < chunk.code.len() {
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
                OpCode::OpReturn => {
                    let Value(return_val) = self.pop_from_stack();
                    println!("{:?}", return_val);
                    return InterpretResult::InterpretOk;
                }
                OpCode::OpNegate => {
                    let Value(val) = self.pop_from_stack();
                    self.push_to_stack(Value(-val));
                }
                OpCode::OpAdd => {
                    let Value(operand_2) = self.pop_from_stack();
                    let Value(operand_1) = self.pop_from_stack();
                    self.push_to_stack(Value(operand_1 + operand_2));
                }
                OpCode::OpSubtract => {
                    let Value(operand_2) = self.pop_from_stack();
                    let Value(operand_1) = self.pop_from_stack();
                    self.push_to_stack(Value(operand_1 - operand_2));
                }
                OpCode::OpMultiply => {
                    let Value(operand_2) = self.pop_from_stack();
                    let Value(operand_1) = self.pop_from_stack();
                    self.push_to_stack(Value(operand_1 * operand_2));
                }
                OpCode::OpDivide => {
                    let Value(operand_2) = self.pop_from_stack();
                    let Value(operand_1) = self.pop_from_stack();
                    self.push_to_stack(Value(operand_1 / operand_2));
                }

                // OpCode::OpAdd
                // | OpCode::OpSubtract
                // | OpCode::OpMultiply
                // | OpCode::OpDivide => {
                //     let Value(operand_1) = self.pop_from_stack();
                //     let Value(operand_2) = self.pop_from_stack();
                //     self.push_to_stack(Value(operand_1 + operand_2));
                // }
                OpCode::OpConstant(idx) => {
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

    fn read_instruction(&mut self, chunk: &Chunk) -> OpCode {
        let instruction = chunk.read_code(self.ip);
        self.ip += 1;
        instruction
    }
}
