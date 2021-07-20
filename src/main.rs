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
        #[allow(unused_must_use)]
        { vm.interpret(user_input.clone()); }
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
#[cfg(test)]
mod tests {
    use super::*;

    mod expressions {
        use super::*;

        #[test]
        fn evaluate() -> VMResult {
            let source = r#"
// Note: Slightly modified from the original.
print (5 - (3 - 1)) + -1;
// expect: 2
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "2".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }
    }

    mod assignment {
        use super::*;

        #[test]
        fn associativity() -> VMResult {
            let source = r#"
var a = "a";
var b = "b";
var c = "c";

// Assignment is right-associative.
a = b = c;
print a; // expect: c
print b; // expect: c
print c; // expect: c
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "c".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "c".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "c".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn global() -> VMResult {
            let source = r#"
var a = "before";
print a; // expect: before

a = "after";
print a; // expect: after

print a = "arg"; // expect: arg
print a; // expect: arg
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "arg".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "arg".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "after".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "before".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn grouping() -> VMResult {
            let source = r#"
var a = "a";
(a) = "value"; // Error at '=': Invalid assignment target.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Invalid assignment target.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn infix_operator() -> VMResult {
            let source = r#"
var a = "a";
var b = "b";
a + b = "value"; // Error at '=': Invalid assignment target.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Invalid assignment target.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn local() -> VMResult {
            let source = r#"
{
  var a = "before";
  print a; // expect: before

  a = "after";
  print a; // expect: after

  print a = "arg"; // expect: arg
  print a; // expect: arg
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "arg".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "arg".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "after".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "before".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn prefix_operator() -> VMResult {
            let source = r#"
var a = "a";
!a = "value"; // Error at '=': Invalid assignment target.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Invalid assignment target.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn syntax() -> VMResult {
            let source = r#"
// Assignment on RHS of variable.
var a = "before";
var c = a = "var";
print a; // expect: var
print c; // expect: var
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "var".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "var".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[ignore = "class"]
        #[test]
        fn to_this() -> VMResult {
            let source = r#"
class Foo {
  Foo() {
    this = "value"; // Error at '=': Invalid assignment target.
  }
}

Foo();
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Invalid assignment target.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn undefined() -> VMResult {
            let source = r#"
unknown = "what"; // expect runtime error: Undefined variable 'unknown'.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Undefined variable 'unknown'.",
                vm.latest_error_message
            );
            Ok(())
        }
    }

    mod block {
        use super::*;

        #[ignore = "if"]
        #[test]
        fn empty() -> VMResult {
            let source = r#"
{} // By itself.

// In a statement.
if (true) {}
if (false) {} else {}

print "ok"; // expect: ok
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn scope() -> VMResult {
            let source = r#"
var a = "outer";

{
  var a = "inner";
  print a; // expect: inner
}

print a; // expect: outer
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "outer".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "inner".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }
    }

    mod bool {
        use super::*;

        #[test]
        fn equality() -> VMResult {
            let source = r#"
print true == true;    // expect: true
print true == false;   // expect: false
print false == true;   // expect: false
print false == false;  // expect: true

// Not equal to other types.
print true == 1;        // expect: false
print false == 0;       // expect: false
print true == "true";   // expect: false
print false == "false"; // expect: false
print false == "";      // expect: false

print true != true;    // expect: false
print true != false;   // expect: true
print false != true;   // expect: true
print false != false;  // expect: false

// Not equal to other types.
print true != 1;        // expect: true
print false != 0;       // expect: true
print true != "true";   // expect: true
print false != "false"; // expect: true
print false != "";      // expect: true
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn not() -> VMResult {
            let source = r#"
print !true;    // expect: false
print !false;   // expect: true
print !!true;   // expect: true
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }
    }

    mod comments {
        use super::*;

        #[test]
        fn line_at_eof() -> VMResult {
            let source = r#"
print "ok"; // expect: ok
// comment
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn only_line_comment() -> VMResult {
            let source = r#"
// comment
"#
            .to_string();
            let mut vm = VM::init();
            Ok(())
        }

        #[test]
        fn only_line_comment_and_line() -> VMResult {
            let source = r#"
// comment
"#
            .to_string();
            let mut vm = VM::init();
            Ok(())
        }

        #[test]
        fn unicode() -> VMResult {
            let source = r#"
// Unicode characters are allowed in comments.
//
// Latin 1 Supplement: £§¶ÜÞ
// Latin Extended-A: ĐĦŋœ
// Latin Extended-B: ƂƢƩǁ
// Other stuff: ឃᢆ᯽₪ℜ↩⊗┺░
// Emoji: ☃☺♣

print "ok"; // expect: ok
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }
    }

    #[test]
    fn empty_file() -> VMResult {
        let source = r#"
"#
        .to_string();
        let mut vm = VM::init();
        Ok(())
    }

    #[test]
    fn precedence() -> VMResult {
        let source = r#"
// * has higher precedence than +.
print 2 + 3 * 4; // expect: 14

// * has higher precedence than -.
print 20 - 3 * 4; // expect: 8

// / has higher precedence than +.
print 2 + 6 / 3; // expect: 4

// / has higher precedence than -.
print 2 - 6 / 3; // expect: 0

// < has higher precedence than ==.
print false == 2 < 1; // expect: true

// > has higher precedence than ==.
print false == 1 > 2; // expect: true

// <= has higher precedence than ==.
print false == 2 <= 1; // expect: true

// >= has higher precedence than ==.
print false == 1 >= 2; // expect: true

// 1 - 1 is not space-sensitive.
print 1 - 1; // expect: 0
print 1 -1;  // expect: 0
print 1- 1;  // expect: 0
print 1-1;   // expect: 0

// Using () for grouping.
print (2 * (6 - (2 + 2))); // expect: 4
"#
        .to_string();
        let mut vm = VM::init();
        vm.interpret(source)?;
        assert_eq!(
            "4".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "0".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "0".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "0".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "0".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "true".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "true".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "true".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "true".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "0".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "4".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "8".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        assert_eq!(
            "14".to_string(),
            vm.printed_values.pop().unwrap().to_string()
        );
        Ok(())
    }

    mod print {
        use super::*;

        #[test]
        fn missing_argument() -> VMResult {
            let source = r#"
// [line 2] Error at ';': Expect expression.
print;
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Expect expression.",
                vm.latest_error_message
            );
            Ok(())
        }
    }

    mod string {
        use super::*;

        #[test]
        fn error_after_multiline() -> VMResult {
            let source = r#"
// Tests that we correctly track the line info across multiline strings.
var a = "1
2
3
";

err; // // expect runtime error: Undefined variable 'err'.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Undefined variable 'err'.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn literals() -> VMResult {
            let source = r#"
print "(" + "" + ")";   // expect: ()
print "a string"; // expect: a string

// Non-ASCII.
print "A~¶Þॐஃ"; // expect: A~¶Þॐஃ
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "A~¶Þॐஃ".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "a string".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "()".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[ignore = "refactor or remove"]
        #[test]
        fn multiline() -> VMResult {
            let source = r#"
var a = "1
2
3";
print a;
// expect: 1
// expect: 2
// expect: 3
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "3".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "2".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "1".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn unterminated() -> VMResult {
            let source = r#"
// [line 2] Error: Unterminated string.
"this string has no close quote
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Unterminated string.",
                vm.latest_error_message
            );
            Ok(())
        }
    }

    mod variable {
        use super::*;

        #[ignore = "function"]
        #[test]
        fn collide_with_parameter() -> VMResult {
            let source = r#"
fun foo(a) {
  var a; // Error at 'a': Already variable with this name in this scope.
}
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
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
  var a = "other"; // Error at 'a': Already variable with this name in this scope.
}
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Already variable with this name in this scope.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn duplicate_parameter() -> VMResult {
            let source = r#"
fun foo(arg,
        arg) { // Error at 'arg': Already variable with this name in this scope.
  "body";
}
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Already variable with this name in this scope.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[ignore = "function"]
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
            assert_eq!(
                "outer".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "outer".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn in_middle_of_block() -> VMResult {
            let source = r#"
{
  var a = "a";
  print a; // expect: a
  var b = a + " b";
  print b; // expect: a b
  var c = a + " c";
  print c; // expect: a c
  var d = b + " d";
  print d; // expect: a b d
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "a b d".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "a c".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "a b".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "a".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn in_nested_block() -> VMResult {
            let source = r#"
{
  var a = "outer";
  {
    print a; // expect: outer
  }
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "outer".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[ignore = "method"]
        #[test]
        fn local_from_method() -> VMResult {
            let source = r#"
var foo = "variable";

class Foo {
  method() {
    print foo;
  }
}

Foo().method(); // expect: variable
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "variable".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn redeclare_global() -> VMResult {
            let source = r#"
var a = "1";
var a;
print a; // expect: nil
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "nil".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn redefine_global() -> VMResult {
            let source = r#"
var a = "1";
var a = "2";
print a; // expect: 2
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "2".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn scope_reuse_in_different_blocks() -> VMResult {
            let source = r#"
{
  var a = "first";
  print a; // expect: first
}

{
  var a = "second";
  print a; // expect: second
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "second".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "first".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn shadow_and_local() -> VMResult {
            let source = r#"
{
  var a = "outer";
  {
    print a; // expect: outer
    var a = "inner";
    print a; // expect: inner
  }
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "inner".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "outer".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn shadow_global() -> VMResult {
            let source = r#"
var a = "global";
{
  var a = "shadow";
  print a; // expect: shadow
}
print a; // expect: global
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "global".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "shadow".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn shadow_local() -> VMResult {
            let source = r#"
{
  var a = "local";
  {
    var a = "shadow";
    print a; // expect: shadow
  }
  print a; // expect: local
}
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "local".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "shadow".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn undefined_global() -> VMResult {
            let source = r#"
print notDefined;  // expect runtime error: Undefined variable 'notDefined'.
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Undefined variable 'notDefined'.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn undefined_local() -> VMResult {
            let source = r#"
{
  print notDefined;  // expect runtime error: Undefined variable 'notDefined'.
}
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Undefined variable 'notDefined'.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn uninitialized() -> VMResult {
            let source = r#"
var a;
print a; // expect: nil
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "nil".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[ignore = "if"]
        #[test]
        fn unreached_undefined() -> VMResult {
            let source = r#"
if (false) {
  print notDefined;
}

print "ok"; // expect: ok
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn use_false_as_var() -> VMResult {
            let source = r#"
// [line 2] Error at 'false': Expect variable name.
var false = "value";
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Expect variable name.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn use_global_in_initializer() -> VMResult {
            let source = r#"
var a = "value";
var a = a;
print a; // expect: value
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "value".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn use_local_in_initializer() -> VMResult {
            let source = r#"
var a = "outer";
{
  var a = a; // Error at 'a': Can't read local variable in its own initializer.
}
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Can't read local variable in its own initializer.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn use_nil_as_var() -> VMResult {
            let source = r#"
// [line 2] Error at 'nil': Expect variable name.
var nil = "value";
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Expect variable name.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn use_this_as_var() -> VMResult {
            let source = r#"
// [line 2] Error at 'this': Expect variable name.
var this = "value";
"#
            .to_string();
            let mut vm = VM::init();
            #[allow(unused_must_use)]
            { vm.interpret(source); }
            assert_eq!(
                "Expect variable name.",
                vm.latest_error_message
            );
            Ok(())
        }
    }

    mod logical_operator {
        use super::*;

        #[test]
        fn and() -> VMResult {
            let source = r#"
// Note: These tests implicitly depend on ints being truthy.

// Return the first non-true argument.
print false and 1; // expect: false
print true and 1; // expect: 1
print 1 and 2 and false; // expect: false

// Return the last argument if all are true.
print 1 and true; // expect: true
print 1 and 2 and 3; // expect: 3

// Short-circuit at the first false argument.
var a = "before";
var b = "before";
(a = true) and
    (b = false) and
    (a = "bad");
print a; // expect: true
print b; // expect: false
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "3".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "1".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn and_truth() -> VMResult {
            let source = r#"
// False and nil are false.
print false and "bad"; // expect: false
print nil and "bad"; // expect: nil

// Everything else is true.
print true and "ok"; // expect: ok
print 0 and "ok"; // expect: ok
print "" and "ok"; // expect: ok
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "nil".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn or() -> VMResult {
            let source = r#"
// Note: These tests implicitly depend on ints being truthy.

// Return the first true argument.
print 1 or true; // expect: 1
print false or 1; // expect: 1
print false or false or true; // expect: true

// Return the last argument if all are false.
print false or false; // expect: false
print false or false or false; // expect: false

// Short-circuit at the first true argument.
var a = "before";
var b = "before";
(a = false) or
    (b = true) or
    (a = "bad");
print a; // expect: false
print b; // expect: true
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "false".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "1".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "1".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }

        #[test]
        fn or_truth() -> VMResult {
            let source = r#"
// False and nil are false.
print false or "ok"; // expect: ok
print nil or "ok"; // expect: ok

// Everything else is true.
print true or "ok"; // expect: true
print 0 or "ok"; // expect: 0
print "s" or "ok"; // expect: s
"#
            .to_string();
            let mut vm = VM::init();
            vm.interpret(source)?;
            assert_eq!(
                "s".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "0".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "true".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            assert_eq!(
                "ok".to_string(),
                vm.printed_values.pop().unwrap().to_string()
            );
            Ok(())
        }
    }

    #[test]
    fn unexpected_character() -> VMResult {
        let source = r#"
// [line 3] Error: Unexpected character.
// [java line 3] Error at 'b': Expect ')' after arguments.
foo(a | b);
"#
        .to_string();
        let mut vm = VM::init();
        #[allow(unused_must_use)]
        { vm.interpret(source); }
        assert_eq!(
            "Unexpected character.",
            vm.latest_error_message
        );
        Ok(())
    }
}
