mod chunk;
mod compiler;
mod parser;
mod scanner;
mod value;
mod vm;

use std::io::Write;
use vm::vm::*;

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
    // let mut vm = VM::new(&chunk);
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

        let mut vm = VM::new();
        #[allow(unused_must_use)]
        {
            vm.interpret(user_input.clone());
        }
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

    let mut vm = VM::new();
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }
    }

    mod assignment {
        use crate::vm::vm::{VMResult, VM};

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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("c", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("c", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("c", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("arg", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("arg", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("after", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("before", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn grouping() -> VMResult {
            let source = r#"
var a = "a";
(a) = "value"; // Error at '=': Invalid assignment target.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Invalid assignment target.", vm.latest_error_message);
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Invalid assignment target.", vm.latest_error_message);
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("arg", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("arg", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("after", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("before", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn prefix_operator() -> VMResult {
            let source = r#"
var a = "a";
!a = "value"; // Error at '=': Invalid assignment target.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Invalid assignment target.", vm.latest_error_message);
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("var", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("var", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Invalid assignment target.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn undefined() -> VMResult {
            let source = r#"
unknown = "what"; // expect runtime error: Undefined variable 'unknown'.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Undefined variable 'unknown'.", vm.latest_error_message);
            Ok(())
        }
    }

    mod block {
        use crate::vm::vm::VMResult;

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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("outer", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("inner", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn only_line_comment() -> VMResult {
            let source = r#"
// comment
"#
            .to_string();
            let mut vm = VM::new();
            Ok(())
        }

        #[test]
        fn only_line_comment_and_line() -> VMResult {
            let source = r#"
// comment
"#
            .to_string();
            let mut vm = VM::new();
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }
    }

    #[test]
    fn empty_file() -> VMResult {
        let source = r#"
"#
        .to_string();
        let mut vm = VM::new();
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
        let mut vm = VM::new();
        vm.interpret(source)?;
        assert_eq!("4", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("4", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("8", vm.printed_values.pop().unwrap().to_string());
        assert_eq!("14", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Undefined variable 'err'.", vm.latest_error_message);
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("A~¶Þॐஃ", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("a string", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("()", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn unterminated() -> VMResult {
            let source = r#"
// [line 2] Error: Unterminated string.
"this string has no close quote
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Unterminated string.", vm.latest_error_message);
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("outer", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("outer", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("a b d", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("a c", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("a b", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("a", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("outer", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("variable", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("nil", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("second", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("first", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("inner", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("outer", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("global", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("shadow", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("local", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("shadow", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn undefined_global() -> VMResult {
            let source = r#"
print notDefined;  // expect runtime error: Undefined variable 'notDefined'.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Undefined variable 'notDefined'.", vm.latest_error_message);
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Undefined variable 'notDefined'.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn uninitialized() -> VMResult {
            let source = r#"
var a;
print a; // expect: nil
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("nil", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn use_false_as_var() -> VMResult {
            let source = r#"
// [line 2] Error at 'false': Expect variable name.
var false = "value";
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect variable name.", vm.latest_error_message);
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("value", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
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
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect variable name.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn use_this_as_var() -> VMResult {
            let source = r#"
// [line 2] Error at 'this': Expect variable name.
var this = "value";
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect variable name.", vm.latest_error_message);
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("nil", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
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
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("s", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("ok", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }
    }

    mod if_tests {
        use super::*;

        #[test]
        fn class_in_else_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'class': Expect expression.
if (true) "ok"; else class Foo {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn class_in_then_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'class': Expect expression.
if (true) class Foo {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn dangling_else_test() -> VMResult {
            let source = r#"
// A dangling else binds to the right-most if.
if (true) if (false) print "bad"; else print "good"; // expect: good
if (false) if (true) print "bad"; else print "bad";
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("good", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn else_test() -> VMResult {
            let source = r#"
// Evaluate the 'else' expression if the condition is false.
if (true) print "good"; else print "bad"; // expect: good
if (false) print "bad"; else print "good"; // expect: good

// Allow block body.
if (false) nil; else { print "block"; } // expect: block
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("block", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("good", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("good", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn fun_in_else_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'fun': Expect expression.
if (true) "ok"; else fun foo() {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn fun_in_then_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'fun': Expect expression.
if (true) fun foo() {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn if_test() -> VMResult {
            let source = r#"
// Evaluate the 'then' expression if the condition is true.
if (true) print "good"; // expect: good
if (false) print "bad";

// Allow block body.
if (true) { print "block"; } // expect: block

// Assignment in if condition.
var a = false;
if (a = true) print a; // expect: true
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("block", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("good", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn truth_test() -> VMResult {
            let source = r#"
// False and nil are false.
if (false) print "bad"; else print "false"; // expect: false
if (nil) print "bad"; else print "nil"; // expect: nil

// Everything else is true.
if (true) print true; // expect: true
if (0) print 0; // expect: 0
if ("") print "empty"; // expect: empty
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("empty", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("nil", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("false", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn var_in_else_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'var': Expect expression.
if (true) "ok"; else var foo;
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn var_in_then_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'var': Expect expression.
if (true) var foo;
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
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
        let mut vm = VM::new();
        #[allow(unused_must_use)]
        {
            vm.interpret(source);
        }
        assert_eq!("Unexpected character.", vm.latest_error_message);
        Ok(())
    }

    mod while_tests {
        use super::*;

        #[ignore = "class"]
        #[test]
        fn class_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'class': Expect expression.
while (true) class Foo {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[ignore = "closure"]
        #[test]
        fn closure_in_body_test() -> VMResult {
            let source = r#"
var f1;
var f2;
var f3;

var i = 1;
while (i < 4) {
  var j = i;
  fun f() { print j; }

  if (j == 1) f1 = f;
  else if (j == 2) f2 = f;
  else f3 = f;

  i = i + 1;
}

f1(); // expect: 1
f2(); // expect: 2
f3(); // expect: 3
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn fun_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'fun': Expect expression.
while (true) fun foo() {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[ignore = "closure"]
        #[test]
        fn return_closure_test() -> VMResult {
            let source = r#"
fun f() {
  while (true) {
    var i = "i";
    fun g() { print i; }
    return g;
  }
}

var h = f();
h(); // expect: i
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("i", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn return_inside_test() -> VMResult {
            let source = r#"
fun f() {
  while (true) {
    var i = "i";
    return i;
  }
}

print f();
// expect: i
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("i", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn syntax_test() -> VMResult {
            let source = r#"
// Single-expression body.
var c = 0;
while (c < 3) print c = c + 1;
// expect: 1
// expect: 2
// expect: 3

// Block body.
var a = 0;
while (a < 3) {
  print a;
  a = a + 1;
}
// expect: 0
// expect: 1
// expect: 2

// Statement bodies.
while (false) if (true) 1; else 2;
while (false) while (true) 1;
while (false) for (;;) 1;
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn var_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'var': Expect expression.
while (true) var foo;
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }
    }

    mod for_tests {
        use super::*;

        #[ignore = "class"]
        #[test]
        fn class_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'class': Expect expression.
for (;;) class Foo {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[ignore = "closure"]
        #[test]
        fn closure_in_body_test() -> VMResult {
            let source = r#"
var f1;
var f2;
var f3;

for (var i = 1; i < 4; i = i + 1) {
var j = i;
fun f() {
print i;
print j;
}

if (j == 1) f1 = f;
else if (j == 2) f2 = f;
else f3 = f;
}

f1(); // expect: 4
  // expect: 1
f2(); // expect: 4
  // expect: 2
f3(); // expect: 4
  // expect: 3
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("4", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("4", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("4", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn fun_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'fun': Expect expression.
for (;;) fun foo() {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[ignore = "closure"]
        #[test]
        fn return_closure_test() -> VMResult {
            let source = r#"
fun f() {
for (;;) {
var i = "i";
fun g() { print i; }
return g;
}
}

var h = f();
h(); // expect: i
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("i", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn return_inside_test() -> VMResult {
            let source = r#"
fun f() {
for (;;) {
var i = "i";
return i;
}
}

print f();
// expect: i
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("i", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn scope_test() -> VMResult {
            let source = r#"
{
var i = "before";

// New variable is in inner scope.
for (var i = 0; i < 1; i = i + 1) {
print i; // expect: 0

// Loop body is in second inner scope.
var i = -1;
print i; // expect: -1
}
}

{
// New variable shadows outer variable.
for (var i = 0; i > 0; i = i + 1) {}

// Goes out of scope after loop.
var i = "after";
print i; // expect: after

// Can reuse an existing variable.
for (i = 0; i < 1; i = i + 1) {
print i; // expect: 0
}
}
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("after", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("-1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn statement_condition_test() -> VMResult {
            let source = r#"
// [line 3] Error at ')': Expect ';' after expression.
for (var a = 1; {}; a = a + 1) {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect ';' after expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn statement_increment_test() -> VMResult {
            let source = r#"
// [line 2] Error at '{': Expect expression.
for (var a = 1; a < 2; {}) {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn statement_initializer_test() -> VMResult {
            let source = r#"
// [line 3] Error at ')': Expect ';' after expression.
for ({}; a < 2; a = a + 1) {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect ';' after expression.", vm.latest_error_message);
            Ok(())
        }

        #[ignore = "function"]
        #[test]
        fn syntax_test() -> VMResult {
            let source = r#"
// Single-expression body.
for (var c = 0; c < 3;) print c = c + 1;
// expect: 1
// expect: 2
// expect: 3

// Block body.
for (var a = 0; a < 3; a = a + 1) {
print a;
}
// expect: 0
// expect: 1
// expect: 2

// No clauses.
fun foo() {
for (;;) return "done";
}
print foo(); // expect: done

// No variable.
var i = 0;
for (; i < 2; i = i + 1) print i;
// expect: 0
// expect: 1

// No condition.
fun bar() {
for (var i = 0;; i = i + 1) {
print i;
if (i >= 2) return;
}
}
bar();
// expect: 0
// expect: 1
// expect: 2

// No increment.
for (var i = 0; i < 2;) {
print i;
i = i + 1;
}
// expect: 0
// expect: 1

// Statement bodies.
for (; false;) if (true) 1; else 2;
for (; false;) while (true) 1;
for (; false;) for (;;) 1;
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("done", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("2", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn var_in_body_test() -> VMResult {
            let source = r#"
// [line 2] Error at 'var': Expect expression.
for (;;) var foo;
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect expression.", vm.latest_error_message);
            Ok(())
        }
    }
    mod function_tests {
        use super::*;

        #[test]
        fn body_must_be_block_test() -> VMResult {
            let source = r#"
// [line 3] Error at '123': Expect '{' before function body.
// [c line 4] Error at end: Expect '}' after block.
fun f() 123;
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect '{' before function body.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn empty_body_test() -> VMResult {
            let source = r#"
fun f() {}
print f(); // expect: nil
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("nil", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn extra_arguments_test() -> VMResult {
            let source = r#"
fun f(a, b) {
print a;
print b;
}

f(1, 2, 3, 4); // expect runtime error: Expected 2 arguments but got 4.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expected 2 arguments but got 4.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn local_mutual_recursion_test() -> VMResult {
            let source = r#"
{
fun isEven(n) {
if (n == 0) return true;
return isOdd(n - 1); // expect runtime error: Undefined variable 'isOdd'.
}

fun isOdd(n) {
if (n == 0) return false;
return isEven(n - 1);
}

isEven(4);
}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Undefined variable 'isOdd'.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn local_recursion_test() -> VMResult {
            let source = r#"
{
fun fib(n) {
if (n < 2) return n;
return fib(n - 1) + fib(n - 2);
}

print fib(8); // expect: 21
}
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("21", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn missing_arguments_test() -> VMResult {
            let source = r#"
fun f(a, b) {}

f(1); // expect runtime error: Expected 2 arguments but got 1.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expected 2 arguments but got 1.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn missing_comma_in_parameters_test() -> VMResult {
            let source = r#"
// [line 3] Error at 'c': Expect ')' after parameters.
// [c line 4] Error at end: Expect '}' after block.
fun foo(a, b c, d, e, f) {}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!("Expect ')' after parameters.", vm.latest_error_message);
            Ok(())
        }

        #[test]
        fn mutual_recursion_test() -> VMResult {
            let source = r#"
fun isEven(n) {
if (n == 0) return true;
return isOdd(n - 1);
}

fun isOdd(n) {
if (n == 0) return false;
return isEven(n - 1);
}

print isEven(4); // expect: true
print isOdd(3); // expect: true
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("true", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn nested_call_with_arguments_test() -> VMResult {
            let source = r#"
fun returnArg(arg) {
return arg;
}

fun returnFunCallWithArg(func, arg) {
return returnArg(func)(arg);
}

fun printArg(arg) {
print arg;
}

returnFunCallWithArg(printArg, "hello world"); // expect: hello world
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("hello world", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn parameters_test() -> VMResult {
            let source = r#"
fun f0() { return 0; }
print f0(); // expect: 0

fun f1(a) { return a; }
print f1(1); // expect: 1

fun f2(a, b) { return a + b; }
print f2(1, 2); // expect: 3

fun f3(a, b, c) { return a + b + c; }
print f3(1, 2, 3); // expect: 6

fun f4(a, b, c, d) { return a + b + c + d; }
print f4(1, 2, 3, 4); // expect: 10

fun f5(a, b, c, d, e) { return a + b + c + d + e; }
print f5(1, 2, 3, 4, 5); // expect: 15

fun f6(a, b, c, d, e, f) { return a + b + c + d + e + f; }
print f6(1, 2, 3, 4, 5, 6); // expect: 21

fun f7(a, b, c, d, e, f, g) { return a + b + c + d + e + f + g; }
print f7(1, 2, 3, 4, 5, 6, 7); // expect: 28

fun f8(a, b, c, d, e, f, g, h) { return a + b + c + d + e + f + g + h; }
print f8(1, 2, 3, 4, 5, 6, 7, 8); // expect: 36
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("36", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("28", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("21", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("15", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("10", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("6", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("3", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("1", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("0", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn print_test() -> VMResult {
            let source = r#"
fun foo() {}
print foo; // expect: <fn foo>

print clock; // expect: <native fn>
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("<native fn>", vm.printed_values.pop().unwrap().to_string());
            assert_eq!("<fn foo>", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn recursion_test() -> VMResult {
            let source = r#"
fun fib(n) {
if (n < 2) return n;
return fib(n - 1) + fib(n - 2);
}

print fib(8); // expect: 21
"#
            .to_string();
            let mut vm = VM::new();
            vm.interpret(source)?;
            assert_eq!("21", vm.printed_values.pop().unwrap().to_string());
            Ok(())
        }

        #[test]
        fn too_many_arguments_test() -> VMResult {
            let source = r#"
fun foo() {}
{
var a = 1;
foo(
 a, // 1
 a, // 2
 a, // 3
 a, // 4
 a, // 5
 a, // 6
 a, // 7
 a, // 8
 a, // 9
 a, // 10
 a, // 11
 a, // 12
 a, // 13
 a, // 14
 a, // 15
 a, // 16
 a, // 17
 a, // 18
 a, // 19
 a, // 20
 a, // 21
 a, // 22
 a, // 23
 a, // 24
 a, // 25
 a, // 26
 a, // 27
 a, // 28
 a, // 29
 a, // 30
 a, // 31
 a, // 32
 a, // 33
 a, // 34
 a, // 35
 a, // 36
 a, // 37
 a, // 38
 a, // 39
 a, // 40
 a, // 41
 a, // 42
 a, // 43
 a, // 44
 a, // 45
 a, // 46
 a, // 47
 a, // 48
 a, // 49
 a, // 50
 a, // 51
 a, // 52
 a, // 53
 a, // 54
 a, // 55
 a, // 56
 a, // 57
 a, // 58
 a, // 59
 a, // 60
 a, // 61
 a, // 62
 a, // 63
 a, // 64
 a, // 65
 a, // 66
 a, // 67
 a, // 68
 a, // 69
 a, // 70
 a, // 71
 a, // 72
 a, // 73
 a, // 74
 a, // 75
 a, // 76
 a, // 77
 a, // 78
 a, // 79
 a, // 80
 a, // 81
 a, // 82
 a, // 83
 a, // 84
 a, // 85
 a, // 86
 a, // 87
 a, // 88
 a, // 89
 a, // 90
 a, // 91
 a, // 92
 a, // 93
 a, // 94
 a, // 95
 a, // 96
 a, // 97
 a, // 98
 a, // 99
 a, // 100
 a, // 101
 a, // 102
 a, // 103
 a, // 104
 a, // 105
 a, // 106
 a, // 107
 a, // 108
 a, // 109
 a, // 110
 a, // 111
 a, // 112
 a, // 113
 a, // 114
 a, // 115
 a, // 116
 a, // 117
 a, // 118
 a, // 119
 a, // 120
 a, // 121
 a, // 122
 a, // 123
 a, // 124
 a, // 125
 a, // 126
 a, // 127
 a, // 128
 a, // 129
 a, // 130
 a, // 131
 a, // 132
 a, // 133
 a, // 134
 a, // 135
 a, // 136
 a, // 137
 a, // 138
 a, // 139
 a, // 140
 a, // 141
 a, // 142
 a, // 143
 a, // 144
 a, // 145
 a, // 146
 a, // 147
 a, // 148
 a, // 149
 a, // 150
 a, // 151
 a, // 152
 a, // 153
 a, // 154
 a, // 155
 a, // 156
 a, // 157
 a, // 158
 a, // 159
 a, // 160
 a, // 161
 a, // 162
 a, // 163
 a, // 164
 a, // 165
 a, // 166
 a, // 167
 a, // 168
 a, // 169
 a, // 170
 a, // 171
 a, // 172
 a, // 173
 a, // 174
 a, // 175
 a, // 176
 a, // 177
 a, // 178
 a, // 179
 a, // 180
 a, // 181
 a, // 182
 a, // 183
 a, // 184
 a, // 185
 a, // 186
 a, // 187
 a, // 188
 a, // 189
 a, // 190
 a, // 191
 a, // 192
 a, // 193
 a, // 194
 a, // 195
 a, // 196
 a, // 197
 a, // 198
 a, // 199
 a, // 200
 a, // 201
 a, // 202
 a, // 203
 a, // 204
 a, // 205
 a, // 206
 a, // 207
 a, // 208
 a, // 209
 a, // 210
 a, // 211
 a, // 212
 a, // 213
 a, // 214
 a, // 215
 a, // 216
 a, // 217
 a, // 218
 a, // 219
 a, // 220
 a, // 221
 a, // 222
 a, // 223
 a, // 224
 a, // 225
 a, // 226
 a, // 227
 a, // 228
 a, // 229
 a, // 230
 a, // 231
 a, // 232
 a, // 233
 a, // 234
 a, // 235
 a, // 236
 a, // 237
 a, // 238
 a, // 239
 a, // 240
 a, // 241
 a, // 242
 a, // 243
 a, // 244
 a, // 245
 a, // 246
 a, // 247
 a, // 248
 a, // 249
 a, // 250
 a, // 251
 a, // 252
 a, // 253
 a, // 254
 a, // 255
 a); // Error at 'a': Can't have more than 255 arguments.
}
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!(
                "Can't have more than 255 arguments.",
                vm.latest_error_message
            );
            Ok(())
        }

        #[test]
        fn too_many_parameters_test() -> VMResult {
            let source = r#"
// 256 parameters.
fun f(
a1,
a2,
a3,
a4,
a5,
a6,
a7,
a8,
a9,
a10,
a11,
a12,
a13,
a14,
a15,
a16,
a17,
a18,
a19,
a20,
a21,
a22,
a23,
a24,
a25,
a26,
a27,
a28,
a29,
a30,
a31,
a32,
a33,
a34,
a35,
a36,
a37,
a38,
a39,
a40,
a41,
a42,
a43,
a44,
a45,
a46,
a47,
a48,
a49,
a50,
a51,
a52,
a53,
a54,
a55,
a56,
a57,
a58,
a59,
a60,
a61,
a62,
a63,
a64,
a65,
a66,
a67,
a68,
a69,
a70,
a71,
a72,
a73,
a74,
a75,
a76,
a77,
a78,
a79,
a80,
a81,
a82,
a83,
a84,
a85,
a86,
a87,
a88,
a89,
a90,
a91,
a92,
a93,
a94,
a95,
a96,
a97,
a98,
a99,
a100,
a101,
a102,
a103,
a104,
a105,
a106,
a107,
a108,
a109,
a110,
a111,
a112,
a113,
a114,
a115,
a116,
a117,
a118,
a119,
a120,
a121,
a122,
a123,
a124,
a125,
a126,
a127,
a128,
a129,
a130,
a131,
a132,
a133,
a134,
a135,
a136,
a137,
a138,
a139,
a140,
a141,
a142,
a143,
a144,
a145,
a146,
a147,
a148,
a149,
a150,
a151,
a152,
a153,
a154,
a155,
a156,
a157,
a158,
a159,
a160,
a161,
a162,
a163,
a164,
a165,
a166,
a167,
a168,
a169,
a170,
a171,
a172,
a173,
a174,
a175,
a176,
a177,
a178,
a179,
a180,
a181,
a182,
a183,
a184,
a185,
a186,
a187,
a188,
a189,
a190,
a191,
a192,
a193,
a194,
a195,
a196,
a197,
a198,
a199,
a200,
a201,
a202,
a203,
a204,
a205,
a206,
a207,
a208,
a209,
a210,
a211,
a212,
a213,
a214,
a215,
a216,
a217,
a218,
a219,
a220,
a221,
a222,
a223,
a224,
a225,
a226,
a227,
a228,
a229,
a230,
a231,
a232,
a233,
a234,
a235,
a236,
a237,
a238,
a239,
a240,
a241,
a242,
a243,
a244,
a245,
a246,
a247,
a248,
a249,
a250,
a251,
a252,
a253,
a254,
a255, a) {} // Error at 'a': Can't have more than 255 parameters.
"#
            .to_string();
            let mut vm = VM::new();
            #[allow(unused_must_use)]
            {
                vm.interpret(source);
            }
            assert_eq!(
                "Can't have more than 255 parameters.",
                vm.latest_error_message
            );
            Ok(())
        }
    }


















// TEMPORARY
    #[test]
    fn temporary() -> VMResult {
        let source = r#"
fun a() { b(); }
fun b() { c(); }
fun c() {
    c("too", "many");
}
a();
"#
        .to_string();
        let mut vm = VM::new();
        vm.interpret(source)?;
        Ok(())
    }
}
