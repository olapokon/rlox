use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Instant, SystemTime};

use crate::value::function::Function;
use crate::value::native_function::NativeFunction;
use crate::{binary_arithmetic_op, binary_boolean_op, compiler::*};
use crate::{chunk::Instruction, value::value::Value};

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
        let mut vm = VM {
            frames: Vec::new(),
            stack: [V; STACK_MAX],
            stack_top: 0,
            globals: HashMap::new(),
            printed_values: Vec::new(),
            latest_error_message: String::new(),
        };

        vm.define_native("clock", clock_native);

        vm
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

        self.call(function, 0, 0)?;

        self.run()
    }

    pub fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.frames.clear();
    }

    fn run(&mut self) -> VMResult {
        let mut frame = self.frames[self.frames.len() - 1].clone();

        loop {
            let chunk = &frame.function.chunk;

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
                // TODO: refactor
                Instruction::OpCall(arg_count) => {
                    // TODO: make peek function
                    let val = self.stack[self.stack_top - 1 - arg_count].get_mut();
                    //

                    // TODO: Put into separate function?
                    let mut function: Option<Rc<Function>> = None;
                    match val {
                        Value::Function(f) => {
                            function = Some(Rc::clone(f));
                        }
                        Value::NativeFunction(f) => {
                            let f = &f.function;
                            let result = f();
                            self.stack_top -= arg_count + 1;
                            self.push_to_stack(result);
                            continue;
                        }
                        _ => {
                            self.runtime_error("Can only call functions and classes.");
                            return Err(VMError::RuntimeError);
                        }
                    }
                    if function.is_some() {
                        self.call(function.unwrap(), arg_count, frame.ip)?;
                    }
                    frame = self.frames[self.frames.len() - 1].clone();
                }
                Instruction::OpNot => {
                    let b = is_falsey(&self.pop_from_stack());
                    self.push_to_stack(Value::Boolean(b))
                }
                Instruction::OpNegate => {
                    if let Value::Number(val) = self.pop_from_stack() {
                        self.push_to_stack(Value::Number(-val))
                    } else {
                        self.runtime_error("Operand must be a number.");
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
                            self.runtime_error(&format!("Undefined variable '{}'.", &name));
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
                            self.runtime_error(&format!("Undefined variable '{}'.", &name));
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
                    let return_val = self.pop_from_stack();
                    self.frames.pop();
                    if self.frames.is_empty() {
                        self.pop_from_stack();
                        return Ok(());
                    }

                    self.stack_top = frame.stack_index;
                    self.push_to_stack(return_val);
                    frame = self.frames[self.frames.len() - 1].clone();
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

    // fn call_value(&mut self, callee: Value, arg_count: usize) {
    // }

    fn call(
        &mut self,
        function: Rc<Function>,
        arg_count: usize,
        current_frame_ip: usize,
    ) -> VMResult {
        if arg_count != function.arity as usize {
            self.runtime_error(&format!(
                "Expected {} arguments but got {}.",
                &function.arity, arg_count
            ));
            return Err(VMError::RuntimeError);
        }

        if self.frames.len() == FRAMES_MAX {
            self.runtime_error("Stack overflow.");
            return Err(VMError::RuntimeError);
        }
        // Save the frame ip in the frame in the VM::frames array.
        // The clone being used only has a copy of the ip, as the ip is not heap allocated.
        if !self.frames.is_empty() {
            self.frames.last_mut().unwrap().ip = current_frame_ip;
        }

        let frame = CallFrame {
            function: function,
            ip: 0,
            stack_index: self.stack_top - 1 - arg_count,
        };
        //
        self.frames.push(frame);
        Ok(())
    }

    // TODO: use peek in some cases instead of popping immediately?
    // cloning must be refactored in that case
    //
    // fn peek(&self, distance: usize) -> Value {
    //     self.stack[self.stack_top - 1 - distance].clone().take()
    // }

    // TODO: Make a RuntimeError struct and refactor this method?
    fn runtime_error(&mut self, message: &str) {
        eprint!("{}", &message);
        self.latest_error_message = message.to_string();
        eprintln!();

        // let line = chunk.lines[ip];
        // eprintln!("[line {}] in script", line);

        for i in (0..self.frames.len()).rev() {
            let frame = &self.frames[i];
            let function = &frame.function;

            // TODO: fix index?
            // let instruction_idx = function.chunk.bytecode.len() - 1;
            let instruction_idx = frame.ip;
            eprint!(
                "[line {}] in ",
                function.chunk.lines[instruction_idx as usize]
            );
            if function.name.is_empty() {
                eprintln!("script");
            } else {
                eprintln!("{}()", &function.name);
            }
        }

        self.reset_stack();
    }

    fn define_native(&mut self, name: &str, function: fn() -> Value) {
        let native = NativeFunction {
            arity: 0,
            name: name.to_string(),
            function
        };
        self.globals.insert(name.to_string(), Value::NativeFunction(Rc::new(native)));
    }

    fn print_globals(&self) {
        println!("VM globals:");
        self.globals.iter().for_each(|(global_name, global_value)| {
            println!("\t{}: {}", global_name, global_value)
        });
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

fn clock_native() -> Value {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Native function error.")
        .as_secs_f64();
    Value::Number(time as f64)
}
