use crate::compiler::*;
use crate::{
	chunk::{Chunk, Instruction},
	value::Value,
};
use crate::memory::Heap;

const STACK_MAX: usize = 256;

/// A virtual machine that interprets chunks of bytecode.
pub struct VM {
	/// The instruction pointer.
	/// It is the index of the instruction about to be executed, in the current [Chunk]'s code array.
	ip: usize,
	/// The VM's stack.
	stack: [Value; STACK_MAX],
	/// The index pointing right after the last element of the stack.
	stack_top: usize,
	/// The VM's heap.
	heap: Heap,
}

pub enum InterpretResult {
	Ok,
	CompileError,
	RuntimeError,
}

impl VM {
	pub fn interpret(source: String) -> InterpretResult {
		let r = match Compiler::compile(source) {
			Ok(r) => r,
			Err(_) => return InterpretResult::CompileError,
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

	pub fn reset_stack(&mut self) {
		self.stack_top = 0;
	}

	fn run(&mut self, chunk: Chunk) -> InterpretResult {
		// TODO: Check value type with peek instead of popping immediately?
		while self.ip < chunk.bytecode.len() {

			#[cfg(feature = "debug_trace_execution")]
			if cfg!(feature = "debug_trace_execution") {
				for i in 0..self.stack_top {
					print!("[{:?}]", self.stack[i]);
				}
				println!();
				chunk.disassemble_instruction(self.ip);
			}

			let instruction = self.read_instruction(&chunk);
			match instruction {
				Instruction::OpReturn => {
					let return_val = self.pop_from_stack();
					println!("{:?}", return_val);
					return InterpretResult::Ok;
				}
				Instruction::OpNot => {
					let b = is_falsey(self.pop_from_stack());
					self.push_to_stack(Value::Boolean(b))
				}
				Instruction::OpNegate => match self.peek(0) {
					Value::Number(val) => self.push_to_stack(Value::Number(-val)),
					_ => {
						self.runtime_error(chunk, "Operand must be a number.", None, None);
						return InterpretResult::RuntimeError;
					}
				},
				Instruction::OpEqual => {
					let v_2 = self.pop_from_stack();
					let v_1 = self.pop_from_stack();
					self.push_to_stack(Value::Boolean(values_equal(v_1, v_2)));
				}
				Instruction::OpAdd
				| Instruction::OpSubtract
				| Instruction::OpMultiply
				| Instruction::OpDivide
				| Instruction::OpGreater
				| Instruction::OpLess => {
					let operand_2 = if let Value::Number(operand_2) = self.peek(0) {
						self.pop_from_stack();
						operand_2
					} else {
						self.runtime_error(chunk, "Operands must be numbers.", None, None);
						return InterpretResult::RuntimeError;
					};
					let operand_1 = if let Value::Number(operand_1) = self.peek(0) {
						self.pop_from_stack();
						operand_1
					} else {
						self.runtime_error(chunk, "Operands must be numbers.", None, None);
						return InterpretResult::RuntimeError;
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
						Instruction::OpGreater => {
							self.push_to_stack(Value::Boolean(operand_1 > operand_2))
						}
						Instruction::OpLess => {
							self.push_to_stack(Value::Boolean(operand_1 < operand_2))
						}
						_ => return InterpretResult::RuntimeError,
					}
				}
				Instruction::OpNil => self.push_to_stack(Value::Nil),
				Instruction::OpTrue => self.push_to_stack(Value::True),
				Instruction::OpFalse => self.push_to_stack(Value::False),
				Instruction::OpConstant(idx) => {
					let constant: Value = chunk.read_constant(idx);
					self.push_to_stack(constant);
				}
			}
		}

		// If there has been no return up to this point, it is an error.
		InterpretResult::RuntimeError
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

	fn peek(&self, distance: usize) -> Value {
		self.stack[self.stack_top - 1 - distance]
	}

	fn runtime_error(&mut self, chunk: Chunk, message: &str, arg1: Option<&str>, arg2: Option<&str>) {
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

	fn is_falsey(v: Value) -> bool {
		match v {
			Value::Nil => true,
			Value::Boolean(b) => b,
			_ => false,
		}
	}
}

fn is_falsey(v: Value) -> bool {
	match v {
		Value::Nil => true,
		Value::Boolean(b) => !b,
		_ => false,
	}
}

fn values_equal(v1: Value, v2: Value) -> bool {
	match v1 {
		Value::Boolean(b1) => match v2 {
			Value::Boolean(b2) => b1 == b2,
			_ => false,
		},
		Value::Number(n1) => match v2 {
			Value::Number(n2) => n1 == n2,
			_ => false,
		},
		Value::Nil => true,
		_ => false,
	}
}
