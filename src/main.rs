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
        assert_eq!(vm.printed_values.pop().unwrap(), Value::Number(2f64));
        Ok(())
    }

    #[test]
    fn boolean_expression() -> VMResult {
        let source = "print !(5 - 4 > 3 * 2 == !nil);".to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.printed_values.pop().unwrap(), Value::Boolean(true));
        Ok(())
    }

    #[test]
    fn non_ascii_string() -> VMResult {
        let source = r#"print "A~¶Þॐஃ";"#.to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.printed_values.pop().unwrap().to_string(), "A~¶Þॐஃ".to_string());
        Ok(())
    }

    #[test]
    fn concatenate_strings() -> VMResult {
        let source = r#"print "(" + "" + ")";"#.to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(vm.printed_values.pop().unwrap().to_string(), "()".to_string());
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
            vm.printed_values.pop().unwrap().to_string(),
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
        assert_eq!(vm.printed_values.pop().unwrap().to_string(), "1\n2\n3".to_string());
        Ok(())
    }

    mod variable {
        use super::*;

        #[ignore = "unimplemented - function"]
        #[test]
        fn collide_with_parameter() -> VMResult {
            let source = r#"
fun foo(a) {
    var a;
}"#
            .to_string();
            let mut vm = VM::init();
            let res = vm.interpret(source);
            assert_eq!(Err(VMError::CompileError), res);
            assert_eq!(
                "Already variable with this name in this scope.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn duplicate_local() -> VMResult {
            let source = r#"
{
    var a = "value";
    var a = "other";
}"#
            .to_string();
            let mut vm = VM::init();
            let res = vm.interpret(source);
            assert_eq!(Err(VMError::CompileError), res);
            assert_eq!(
                "Already variable with this name in this scope.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[ignore = "unimplemented - function"]
        #[test]
        fn duplicate_parameter() -> VMResult {
            let source = r#"
fun foo(arg, arg) {
    "body";
}"#
            .to_string();
            let mut vm = VM::init();
            let res = vm.interpret(source);
            assert_eq!(Err(VMError::CompileError), res);
            assert_eq!(
                "Already variable with this name in this scope.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[ignore = "unimplemented - function"]
        #[test]
        fn early_bound() -> VMResult {
            let source = r#"
var a = "outer";
{
    fun foo() {
        print a;
    }

    foo(); // expect: outer
    var a = "inner";
    foo(); // expect: outer
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(vm.printed_values.pop().unwrap().to_string(), "outer".to_string());
            Ok(())
        }

        #[test]
        fn undefined_global() -> VMResult {
            let source = r#"print notDefined;"#.to_string();
            let mut vm = VM::init();
            let res = vm.interpret(source);
            assert_eq!(Err(VMError::RuntimeError), res);
            assert_eq!("Undefined variable 'notDefined'.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn variable_scopes() -> VMResult {
            let source = r#"
{
    var a = "outer";
    {
        var a = "inner";
    }
    print a;
}"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(vm.printed_values.pop().unwrap().to_string(), "outer".to_string());
            Ok(())
        }

        #[test]
        fn variable_scopes_shadow_outer_local() -> VMResult {
            let source = r#"
{
    var a = "outer";
    {
        var a = "inner";
        print a;
    }
}"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(vm.printed_values.pop().unwrap().to_string(), "inner".to_string());
            Ok(())
        }

        #[test]
        fn variable_scopes_shadow_global() -> VMResult {
            let source = r#"
var a = "global";
{
    var a = "shadow";
}
print a;"#
                .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(vm.printed_values.pop().unwrap().to_string(), "global".to_string());
            Ok(())
        }

        #[test]
        fn variable_scopes_shadow_global_1() -> VMResult {
            let source = r#"
var a = "global";
{
    var a = "shadow";
    print a;
}"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                vm.printed_values.pop().unwrap().to_string(),
                "shadow".to_string()
            );
            Ok(())
        }
    }
}
