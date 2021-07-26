use std::cell::Cell;
use std::collections::HashMap;

use crate::{binary_arithmetic_op, binary_boolean_op, compiler::*};
use crate::{
    chunk::{Chunk, Instruction},
    value::value::Value,
};

const STACK_MAX: usize = 256;

/// A virtual machine that interprets chunks of bytecode.
pub struct VM {
    /// The instruction pointer.
    /// It is the index of the instruction about to be executed, in the current [Chunk]'s code array.
    ip: usize,
    /// The VM's stack.
    stack: [Cell<Value>; STACK_MAX],
    /// The index pointing right after the last element of the stack.
    stack_top: usize,
    /// All global variables.
    globals: HashMap<String, Value>,

    /// Only for testing. Holds the values printed by the print statement,
    /// so that they can be compared to the expected output in the tests.
    pub printed_values: Vec<Value>,
    /// Only for testing. Holds the latest error value
    pub latest_error_message: String,
}

pub type VMResult = Result<(), VMError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VMError {
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> VM {
        const v: Cell<Value> = Cell::new(Value::Nil);
        VM {
            ip: 0,
            stack: [v; STACK_MAX],
            stack_top: 0,
            globals: HashMap::new(),
            printed_values: Vec::new(),
            latest_error_message: String::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> VMResult {
        let r = match Compiler::compile(source) {
            Ok(r) => r,
            Err(error_message) => {
                self.latest_error_message = error_message;
                return Err(VMError::CompileError);
            }
        };
        self.run(r)
    }

    pub fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    fn run(&mut self, chunk: Chunk) -> VMResult {
        while self.ip < chunk.bytecode.len() {
            // conditional compilation for logging
            #[cfg(feature = "debug_trace_execution")]
            if cfg!(feature = "debug_trace_execution") {
                for i in 0..self.stack_top {
                    print!("[{:?}]", self.stack[i].get_mut());
                }
                println!();
                chunk.disassemble_instruction(self.ip);
            }
            //

            let instruction = self.read_instruction(&chunk);
            match instruction {
                Instruction::OpNot => {
                    let b = is_falsey(&self.pop_from_stack());
                    self.push_to_stack(Value::Boolean(b))
                }
                Instruction::OpNegate => {
                    if let Value::Number(val) = self.pop_from_stack() {
                        self.push_to_stack(Value::Number(-val))
                    } else {
                        self.runtime_error(&chunk, "Operand must be a number.", None, None);
                        return Err(VMError::RuntimeError);
                    }
                }
                Instruction::OpGetLocal(stack_index) => {
                    let v = self.stack[stack_index].take();
                    self.stack[stack_index] = Cell::new(v.clone());
                    self.push_to_stack(v);
                }
                Instruction::OpJump(offset) => {
                    self.ip += offset;
                }
                Instruction::OpJumpIfFalse(offset) => {
                    let v: Value = self.pop_from_stack();
                    if is_falsey(&v) {
                        self.ip += offset;
                    }
                    self.push_to_stack(v);
                }
                Instruction::OpLoop(offset) => {
                    self.ip -= offset;
                }
                Instruction::OpSetLocal(stack_index) => {
                    let v = self.stack[self.stack_top - 1].take();
                    self.stack[self.stack_top - 1] = Cell::new(v.clone());
                    self.stack[stack_index] = Cell::new(v);
                }
                Instruction::OpGetGlobal(index) => {
                    if let Value::String(name) = chunk.read_constant(index) {
                        let v = self.globals.get(&name.to_string());
                        if v.is_none() {
                            self.runtime_error(
                                &chunk,
                                &format!("Undefined variable '{}'.", &name),
                                None,
                                None,
                            );
                            return Err(VMError::RuntimeError);
                        }
                        let v = v.unwrap().clone();
                        self.push_to_stack(v);
                    } else {
                        return Err(VMError::RuntimeError);
                    };
                }
                Instruction::OpSetGlobal(index) => {
                    if let Value::String(name) = chunk.read_constant(index) {
                        // cannot set uninitialized variable
                        // in case of error, delete it from the table (only relevant for the REPL)
                        if !self.globals.contains_key(&name.to_string()) {
                            self.globals.remove(&name.to_string());
                            self.runtime_error(
                                &chunk,
                                &format!("Undefined variable '{}'.", &name),
                                None,
                                None,
                            );
                            return Err(VMError::RuntimeError);
                        }

                        // value is not popped from the stack after setting
                        // assignment is an expression so the value should be present at the top
                        let val = self.stack[self.stack_top - 1].take();
                        self.stack[self.stack_top - 1] = Cell::new(val.clone());
                        self.globals
                            .insert(name.to_string(), val)
                            .ok_or(VMError::RuntimeError)?;
                    } else {
                        return Err(VMError::RuntimeError);
                    };
                }
                Instruction::OpDefineGlobal(index) => {
                    if let Value::String(name) = chunk.read_constant(index) {
                        let val = self.pop_from_stack();
                        self.globals.insert(String::clone(name), val);
                        //
                        // TODO: remove this print
                        // println!("\nDEFINING NEW GLOBAL");
                        // self.print_globals();
                        //
                    } else {
                        return Err(VMError::RuntimeError);
                    };
                }
                Instruction::OpEqual => {
                    let v_2 = self.pop_from_stack();
                    let v_1 = self.pop_from_stack();
                    self.push_to_stack(Value::Boolean(Value::equals(v_1, v_2)));
                }
                Instruction::OpAdd => {
                    let operand_2 = self.pop_from_stack();
                    let operand_1 = self.pop_from_stack();
                    if Value::is_string(&operand_1) {
                        if let Ok(v) = Value::concatenate_strings(&operand_1, &operand_2) {
                            self.push_to_stack(v);
                        } else {
                            return Err(VMError::RuntimeError);
                        };
                    } else {
                        if let Ok(v) = binary_arithmetic_op!(operand_1 + operand_2) {
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
                    let operand_2 = self.pop_from_stack();
                    let operand_1 = self.pop_from_stack();
                    if let Ok(v) = match instruction {
                        Instruction::OpSubtract => binary_arithmetic_op!(operand_1 - operand_2),
                        Instruction::OpMultiply => binary_arithmetic_op!(operand_1 * operand_2),
                        Instruction::OpDivide => binary_arithmetic_op!(operand_1 / operand_2),
                        Instruction::OpGreater => binary_boolean_op!(operand_1 > operand_2),
                        Instruction::OpLess => binary_boolean_op!(operand_1 < operand_2),
                        _ => return Err(VMError::RuntimeError),
                    } {
                        self.push_to_stack(v);
                    } else {
                        return Err(VMError::RuntimeError);
                    };
                }
                Instruction::OpNil => self.push_to_stack(Value::Nil),
                Instruction::OpTrue => self.push_to_stack(Value::Boolean(true)),
                Instruction::OpFalse => self.push_to_stack(Value::Boolean(false)),
                Instruction::OpConstant(idx) => {
                    let constant = chunk.read_constant(idx);
                    self.push_to_stack(constant.clone());
                }
                Instruction::OpPop => {
                    self.pop_from_stack();
                }
                // Instruction::OpPrint => print!("{}", self.pop_from_stack()),
                Instruction::OpPrint => {
                    let v = self.pop_from_stack();
                    // TODO: conditional execution only for tests
                    self.printed_values.push(v.clone());
                    //
                    print!("{}", v);
                }
                Instruction::OpReturn => {
                    // let return_val = self.pop_from_stack();
                    // println!("{:?}", return_val);
                    return Ok(());
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

    // TODO: use peek in some cases instead of popping immediately?
    // cloning must be refactored in that case
    //
    // fn peek(&self, distance: usize) -> Value {
    //     self.stack[self.stack_top - 1 - distance].clone().take()
    // }

    // TODO: Make a RuntimeError struct and refactor this method?
    fn runtime_error(
        &mut self,
        chunk: &Chunk,
        message: &str,
        arg1: Option<&str>,
        arg2: Option<&str>,
    ) {
        eprint!("{}", &message);
        self.latest_error_message = message.to_string();
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

    fn print_globals(&self) {
        println!("VM globals:");
        self.globals.iter().for_each(|con| println!("\t{:?}", con));
        println!();
    }
}

// TODO: move to value.rs
fn is_falsey(v: &Value) -> bool {
    match v {
        Value::Nil => true,
        Value::Boolean(b) => !b,
        _ => false,
    }
}
