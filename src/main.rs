mod chunk;
mod compiler;
mod vm;
mod value;
mod memory;

use std::io::Write;
use vm::*;

fn main() {
	let args_count = std::env::args().count();
	match args_count {
		1 => repl(),
		2 => run_file(std::env::args().nth(1).unwrap()),
		_ => {
			eprintln!("Usage: clox [path]");
			std::process::exit(64);
		}
	}

	// let mut chunk = Chunk::init();
	// chunk.disassemble("test chunk");
	// let mut vm = VM::init(&chunk);
	// vm.interpret();
}

fn repl() {
	let mut user_input = String::new();
	loop {
		print!("> ");
		std::io::stdout()
			.flush()
			.expect("Failed to write to stdout");
		std::io::stdin()
			.read_line(&mut user_input)
			.expect("Failed to read input");

		let mut vm = VM::init();
		vm.interpret(user_input.clone());
		user_input.clear();
	}
}

fn run_file(path: String) {
	let source = match std::fs::read_to_string(&path) {
		Ok(source) => source,
		Err(_) => {
			eprintln!("Could not read file \"{:?}\".", &path);
			std::process::exit(74);
		}
	};

	let mut vm = VM::init();
	let result = vm.interpret(source);

	match result {
		InterpretResult::CompileError => std::process::exit(65),
		InterpretResult::RuntimeError => std::process::exit(70),
		_ => {}
	}
}
