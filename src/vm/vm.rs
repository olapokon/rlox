use std::borrow::{BorrowMut};
use std::cell::{Cell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::{binary_arithmetic_op, binary_boolean_op, compiler::*};
use crate::{
    chunk::{Chunk, Instruction},
    value::value::Value,
};

use super::call_frame::CallFrame;

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = 256 * FRAMES_MAX;

/// A virtual machine that interprets chunks of bytecode.
pub struct VM {
    /// The VM's [CallFrame] stack.
    // frames: Vec<Rc<RefCell<CallFrame>>>,
    frames: Vec<CallFrame>,
    /// The current number of [CallFrame].
    // frame_count: usize,
    /// The VM's value stack.
    stack: [Cell<Value>; STACK_MAX],
    /// The index pointing right after the last element of the stack.
    stack_top: usize,
    /// All global variables.
    globals: HashMap<String, Value>,

    /// Only for testing.
    ///
    ///Holds the values printed by the print statement,
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
        const V: Cell<Value> = Cell::new(Value::Nil);
        VM {
            frames: Vec::new(),
            // frame_count: 0,
            stack: [V; STACK_MAX],
            stack_top: 0,
            globals: HashMap::new(),
            printed_values: Vec::new(),
            latest_error_message: String::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> VMResult {
        let r = match CompilerManager::compile(source) {
            Ok(r) => r,
            Err(error_message) => {
                self.latest_error_message = error_message;
                return Err(VMError::CompileError);
            }
        };

        let function = Rc::new(r);

        // Push the compiled function to the stack.
        self.push_to_stack(Value::Function(Rc::clone(&function)));

        let frame = CallFrame {
            function,
            ip: 0,
            stack_index: 0,
        };
        self.frames.push(frame);

        self.run()
    }

    pub fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.frames.clear();
    }

    fn run(&mut self) -> VMResult {
        let mut frame = self.frames.pop().unwrap();
        let chunk = &frame.function.chunk;

        loop {
            // conditional compilation for logging
            #[cfg(feature = "debug_trace_execution")]
            if cfg!(feature = "debug_trace_execution") {
                for i in 0..self.stack_top {
                    print!("[{}]", self.stack[i].get_mut());
                }
                println!();
                chunk.disassemble_instruction(frame.ip);
            }
            //

            let instruction = chunk.read_code(frame.ip);
            frame.ip += 1;
            match instruction {
                Instruction::OpNot => {
                    let b = is_falsey(&self.pop_from_stack());
                    self.push_to_stack(Value::Boolean(b))
                }
                Instruction::OpNegate => {
                    if let Value::Number(val) = self.pop_from_stack() {
                        self.push_to_stack(Value::Number(-val))
                    } else {
                        self.runtime_error(
                            &frame.function.borrow_mut().chunk,
                            frame.ip - 1,
                            "Operand must be a number.",
                        );
                        return Err(VMError::RuntimeError);
                    }
                }
                Instruction::OpJump(offset) => {
                    frame.ip += offset;
                }
                Instruction::OpJumpIfFalse(offset) => {
                    let v: Value = self.pop_from_stack();
                    if is_falsey(&v) {
                        frame.ip += offset;
                    }
                    self.push_to_stack(v);
                }
                Instruction::OpLoop(offset) => {
                    frame.ip -= offset;
                }
                Instruction::OpGetLocal(frame_index) => {
                    let idx = frame.stack_index + frame_index;
                    let v = self.stack[idx].take();
                    self.stack[idx] = Cell::new(v.clone());
                    self.push_to_stack(v);
                }
                Instruction::OpSetLocal(frame_index) => {
                    let idx = frame.stack_index + frame_index;
                    let v = self.stack[self.stack_top - 1].take();
                    self.stack[self.stack_top - 1] = Cell::new(v.clone());
                    self.stack[idx] = Cell::new(v);
                }
                Instruction::OpGetGlobal(index) => {
                    if let Value::String(name) = chunk.read_constant(index) {
                        let v = self.globals.get(&name.to_string());
                        if v.is_none() {
                            self.runtime_error(
                                chunk,
                                frame.ip - 1,
                                &format!("Undefined variable '{}'.", &name),
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
                                chunk,
                                frame.ip - 1,
                                &format!("Undefined variable '{}'.", &name),
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
                        println!("\nDEFINING NEW GLOBAL");
                        self.print_globals();
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
                    let constant = chunk.read_constant(idx).clone();
                    self.push_to_stack(constant.clone());
                }
                Instruction::OpPop => {
                    self.pop_from_stack();
                }
                Instruction::OpPrint => {
                    let v = self.pop_from_stack();
                    // TODO: conditional execution only for tests
                    self.printed_values.push(v.clone());
                    //
                    println!("{}", v);
                }
                Instruction::OpReturn => {
                    return Ok(());
                }
            }
        }
    }

    fn push_to_stack(&mut self, value: Value) {
        self.stack[self.stack_top].replace(value);
        self.stack_top += 1;
    }

    fn pop_from_stack(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].take()
    }

    // TODO: use peek in some cases instead of popping immediately?
    // cloning must be refactored in that case
    //
    // fn peek(&self, distance: usize) -> Value {
    //     self.stack[self.stack_top - 1 - distance].clone().take()
    // }

    // TODO: Make a RuntimeError struct and refactor this method?
    fn runtime_error(&mut self, chunk: &Chunk, ip: usize, message: &str) {
        eprint!("{}", &message);
        self.latest_error_message = message.to_string();
        eprintln!();

        let line = chunk.lines[ip];
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