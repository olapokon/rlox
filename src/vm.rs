use std::cell::{RefCell};

use crate::{binary_arithmetic_op, binary_boolean_op, compiler::*};
use crate::{
    chunk::{Chunk, Instruction},
    value::Value,
};

const STACK_MAX: usize = 256;

/// A virtual machine that interprets chunks of bytecode.
pub struct VM {
    /// The instruction pointer.
    /// It is the index of the instruction about to be executed, in the current [Chunk]'s code array.
    ip: usize,
    /// The VM's stack.
    stack: [RefCell<Value>; STACK_MAX],
    /// The index pointing right after the last element of the stack.
    stack_top: usize,
}

pub type VMResult = Result<(), VMError>;

pub enum VMError {
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn init() -> VM {
        const v: RefCell<Value> = RefCell::new(Value::Nil);
        VM {
            ip: 0,
            stack: [v; STACK_MAX],
            stack_top: 0,
        }
    }

    pub fn interpret(&mut self, source: String) -> VMResult {
        let r = match Compiler::compile(source) {
            Ok(r) => r,
            Err(_) => return Err(VMError::CompileError),
        };
        self.run(r)
    }

    pub fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    // TODO: refactor InterpretResult to Result?
    fn run(&mut self, chunk: Chunk) -> VMResult {
        // TODO: Check value type with peek instead of popping immediately?
        while self.ip < chunk.bytecode.len() {
            // conditional compilation for logging
            #[cfg(feature = "debug_trace_execution")]
            if cfg!(feature = "debug_trace_execution") {
                for i in 0..self.stack_top {
                    print!("[{:?}]", self.stack[i]);
                }
                println!();
                chunk.disassemble_instruction(self.ip);
            }
            //

            let instruction = self.read_instruction(&chunk);
            match instruction {
                Instruction::OpReturn => {
                    let return_val = self.pop_from_stack();
                    println!("{:?}", return_val);
                    return Ok(());
                }
                Instruction::OpNot => {
                    let b = is_falsey(self.pop_from_stack());
                    self.push_to_stack(Value::Boolean(b))
                }
                Instruction::OpNegate => match self.peek(0) {
                    Value::Number(val) => {
                        self.pop_from_stack();
                        self.push_to_stack(Value::Number(-val))
                    }
                    _ => {
                        self.runtime_error(chunk, "Operand must be a number.", None, None);
                        return Err(VMError::RuntimeError);
                    }
                },
                Instruction::OpEqual => {
                    let v_2 = self.pop_from_stack();
                    let v_1 = self.pop_from_stack();
                    self.push_to_stack(Value::Boolean(Value::equals(v_1, v_2)));
                }
                Instruction::OpAdd => {
                    let operand_1 = self.peek(1);
                    let operand_2 = self.peek(0);
                    if Value::is_string(operand_1) {
                        if let Ok(v) = Value::concatenate_strings(&operand_1, &operand_2) {
                            self.pop_from_stack();
                            self.pop_from_stack();
                            self.push_to_stack(v);
                        } else {
                            return Err(VMError::RuntimeError);
                        };
                    } else {
                        if let Ok(v) = binary_arithmetic_op!(operand_1 + operand_2) {
                            self.pop_from_stack();
                            self.pop_from_stack();
                            self.push_to_stack(v);
                        } else {
                            return Err(VMError::RuntimeError);
                        };
                    }
                }
                Instruction::OpSubtract
                | Instruction::OpMultiply
                | Instruction::OpDivide
                | Instruction::OpGreater
                | Instruction::OpLess => {
                    let operand_1 = self.peek(1);
                    let operand_2 = self.peek(0);
                    if let Ok(v) = match instruction {
                        Instruction::OpSubtract => binary_arithmetic_op!(operand_1 - operand_2),
                        Instruction::OpMultiply => binary_arithmetic_op!(operand_1 * operand_2),
                        Instruction::OpDivide => binary_arithmetic_op!(operand_1 / operand_2),
                        Instruction::OpGreater => binary_boolean_op!(operand_1 > operand_2),
                        Instruction::OpLess => binary_boolean_op!(operand_1 < operand_2),
                        _ => return Err(VMError::RuntimeError),
                    } {
                        self.pop_from_stack();
                        self.pop_from_stack();
                        self.push_to_stack(v);
                    } else {
                        return Err(VMError::RuntimeError);
                    };
                }
                Instruction::OpNil => self.push_to_stack(Value::Nil),
                Instruction::OpTrue => self.push_to_stack(Value::Boolean(true)),
                Instruction::OpFalse => self.push_to_stack(Value::Boolean(false)),
                Instruction::OpConstant(idx) => {
                    let constant: Value = chunk.read_constant(idx);
                    self.push_to_stack(constant);
                }
            }
        }

        // If there has been no return up to this point, it is an error.
        return Err(VMError::RuntimeError);
    }

    fn push_to_stack(&mut self, value: Value) {
        self.stack[self.stack_top].replace(value);
        self.stack_top += 1;
    }

    fn pop_from_stack(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].take()
    }

    fn read_instruction(&mut self, chunk: &Chunk) -> Instruction {
        let instruction = chunk.read_code(self.ip);
        self.ip += 1;
        instruction
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack_top - 1 - distance].borrow()
    }

    // TODO: Make a RuntimeError struct and refactor this method?
    fn runtime_error(
        &mut self,
        chunk: Chunk,
        message: &str,
        arg1: Option<&str>,
        arg2: Option<&str>,
    ) {
        eprint!("{}", message);
        if arg1.is_some() {
            eprint!(" {}", arg1.unwrap());
        }
        if arg2.is_some() {
            eprint!(" {}", arg2.unwrap());
        }
        eprintln!();

        let line = chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);
        self.reset_stack();
    }
}

fn is_falsey(v: Value) -> bool {
    match v {
        Value::Nil => true,
        Value::Boolean(b) => !b,
        _ => false,
    }
}
