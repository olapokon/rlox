mod chunk;
mod compiler;
mod parser;
mod scanner;
mod value;
mod vm;

use std::io::Write;
use vm::*;

fn main() {
    let args_count = std::env::args().count();
    match args_count {
        1 => repl(),
        2 => run_file(std::env::args().nth(1).unwrap()),
        _ => {
            eprintln!("Usage: rlox [path]");
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
        Err(VMError::CompileError) => std::process::exit(65),
        Err(VMError::RuntimeError) => std::process::exit(70),
        _ => {}
    }
}

// TODO: move tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn arithmetic_expression() -> VMResult {
        let source = "print (5 - (3 - 1)) + -1;".to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.latest_printed_value, Value::Number(2f64));
        Ok(())
    }

    #[test]
    fn boolean_expression() -> VMResult {
        let source = "print !(5 - 4 > 3 * 2 == !nil);".to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.latest_printed_value, Value::Boolean(true));
        Ok(())
    }

    #[test]
    fn non_ascii_string() -> VMResult {
        let source = r#"print "A~¶Þॐஃ";"#.to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.latest_printed_value.to_string(), "A~¶Þॐஃ".to_string());
        Ok(())
    }

    #[test]
    fn concatenate_strings() -> VMResult {
        let source = r#"print "(" + "" + ")";"#.to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.latest_printed_value.to_string(), "()".to_string());
        Ok(())
    }

    #[test]
    fn concatenate_strings_with_variables() -> VMResult {
        let source = r#"
var breakfast = "beignets";
var beverage = "cafe au lait";
breakfast = "beignets with " + beverage;
print breakfast;"#
            .to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(
            vm.latest_printed_value.to_string(),
            "beignets with cafe au lait".to_string()
        );
        Ok(())
    }

    #[test]
    fn multiline_string() -> VMResult {
        let source = r#"
var a = "1
2
3";
print a;"#
            .to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.latest_printed_value.to_string(), "1\n2\n3".to_string());
        Ok(())
    }
}
