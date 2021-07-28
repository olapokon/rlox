use core::f64;
use std::{rc::Rc, usize};

use crate::{
    chunk::Instruction,
    parser::Parser,
    scanner::{Scanner, Token, TokenType},
    value::{
        function::{Function, FunctionType},
        value::Value,
    },
};

#[derive(Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

#[derive(PartialEq)]
enum ParseFn {
    Call,
    Grouping,
    // Dot,
    Unary,
    Binary,
    Variable,
    String,
    Number,
    And,
    Literal,
    Or,
    // Super,
    // This,
    None,
}

struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

/// A local variable.
#[derive(Clone, Copy)]
struct Local {
    /// The variable's name.
    name: Token,
    /// The scope depth of the block where the local variable was declared.
    ///
    /// A depth of -1 indicates that the variable has not been initialized.
    depth: i32,
}

pub struct Compiler {
    /// The [Function] currently being compiled.
    function: Function,
    /// The type of the [Function] currently being compiled.
    ///
    /// [Function::Script] indicates the top-level function, which wraps all other functions.
    function_type: FunctionType,
    /// All local variables that are in scope.
    /// They are in the order in which they are declared in the program,
    /// so the local variable's index in this vector is the same as its position in the stack,
    /// relative to the stack frame.
    locals: Vec<Local>,
    /// The number of blocks surrounding the code that is currently being compiled.
    scope_depth: i32,
}

impl Compiler {
    fn new(function_type: FunctionType) -> Compiler {
        Compiler {
            function: Function::new(),
            function_type,
            locals: Vec::new(),
            scope_depth: 0,
        }
    }
}

/// Manages a collection of [Compiler]s.
pub struct CompilerManager {
    /// The index of the [Compiler] currently in use, in the compilers array.
    current: i32,
    compilers: Vec<Compiler>,
    scanner: Scanner,
    parser: Parser,
}

impl CompilerManager {
    pub fn compile(source: String) -> Result<Function, String> {
        let source = source.chars().collect();

        let mut compiler_manager = CompilerManager {
            current: -1,
            compilers: Vec::new(),
            scanner: Scanner::init(source),
            parser: Parser::init(),
        };

        // Add the [Compiler] responsible for compiling the top-level script.
        compiler_manager.init_compiler(FunctionType::Script);

        compiler_manager.advance();
        while !compiler_manager.match_token(TokenType::Eof) {
            compiler_manager.declaration();
        }
        let compiled_function = compiler_manager.end();

        if compiler_manager.parser.had_error {
            Err(compiler_manager.parser.error_message.clone())
        } else {
            Ok(compiled_function)
        }
    }

    fn current_compiler(&mut self) -> &mut Compiler {
        let compiler_idx = self.current as usize;
        self.compilers.get_mut(compiler_idx).unwrap()
    }

    /// Copies a token's lexeme from the source string.
    fn lexeme_to_string(&self, token: Token) -> String {
        self.scanner.source[token.start..(token.start + token.length as usize)]
            .iter()
            .collect()
    }

    /// Copies part of the source string.
    fn section_to_string(&self, start: usize, length: usize) -> String {
        self.scanner.source[start..(start + length)]
            .iter()
            .collect()
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();

            // Report and skip all error tokens, so that the rest of the parser only sees valid ones.
            match self.parser.current.token_type {
                TokenType::Error(e) => self.error_at(
                    self.parser.current,
                    match e {
                        crate::scanner::ScannerError::UnexpectedCharacter => {
                            "Unexpected character."
                        }
                        crate::scanner::ScannerError::UnterminatedString => "Unterminated string.",
                        // TODO: remove this error
                        crate::scanner::ScannerError::UninitializedToken => "Uninitialized token.",
                    },
                ),
                _ => break,
            }
        }
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.parser.previous, message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }

        self.parser.panic_mode = true;
        eprint!("[line {}] Error", token.line);

        match &token.token_type {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error(_) => {}
            _ => eprint!(" at {:?}", self.lexeme_to_string(token)),
        }

        eprintln!(": {}", &message);
        self.parser.had_error = true;
        self.parser.error_message = message.to_string();
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.current.token_type == token_type {
            self.advance();
            return;
        }
        self.error_at(self.parser.current, message);
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        let line_num = self.parser.previous.line;
        self.current_compiler()
            .function
            .chunk
            .write(instruction, line_num);
    }

    fn emit_instructions(&mut self, i_1: Instruction, i_2: Instruction) {
        self.emit_instruction(i_1);
        self.emit_instruction(i_2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant_index = self.make_constant(value);
        self.emit_instruction(Instruction::OpConstant(constant_index));
    }

    // Adds a constant to the Chunk's constants array and returns the index.
    fn make_constant(&mut self, value: Value) -> usize {
        let constant_index = self.current_compiler().function.chunk.add_constant(value);
        if constant_index as u8 > u8::MAX {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant_index
    }

    fn end(&mut self) -> Function {
        self.emit_instruction(Instruction::OpReturn);

        // conditional compilation for logging
        #[cfg(feature = "debug_print_code")]
        if !self.parser.had_error {
            self.print_current_chunk_constants();
            self.current_compiler().function.chunk.disassemble("code");
        }

        // TODO: refactor cloning?
        let compiled_function = self.current_compiler().function.clone();
        self.current -= 1;
        compiled_function
    }

    // TODO: current compiler?
    fn begin_scope(&mut self) {
        self.current_compiler().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.current_compiler().scope_depth -= 1;

        // pop all local variables for the scope that is ending
        for i in (0..self.current_compiler().locals.len()).rev() {
            if self.current_compiler().locals.get(i).unwrap().depth
                > self.current_compiler().scope_depth
            {
                self.emit_instruction(Instruction::OpPop);
                self.current_compiler().locals.pop();
            }
        }
    }

    fn print_current_chunk_constants(&mut self) {
        println!("chunk constants:");
        self.current_compiler()
            .function
            .chunk
            .constants
            .iter()
            .enumerate()
            .for_each(|(i, con)| println!("\t{}: {:?}", i, con));
        println!();
    }

    /// Takes [Precedence] converted to i32.
    // TODO: refactor Precedence?
    fn parse_precedence(&mut self, precedence: i32) {
        self.advance();
        let prefix_rule = CompilerManager::rules(self.parser.previous.token_type);
        if prefix_rule.prefix == ParseFn::None {
            self.error("Expect expression.");
            return;
        }

        let can_assign: bool = precedence <= Precedence::Assignment as i32;
        self.parse_fn(prefix_rule.prefix, can_assign);

        while precedence <= CompilerManager::rules(self.parser.current.token_type).precedence as i32
        {
            self.advance();
            let infix_rule = CompilerManager::rules(self.parser.previous.token_type);
            self.parse_fn(infix_rule.infix, can_assign);
        }

        if can_assign && self.match_token(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    fn match_token(&mut self, tt: TokenType) -> bool {
        if !self.check(tt) {
            return false;
        }
        self.advance();
        return true;
    }

    fn check(&self, tt: TokenType) -> bool {
        self.parser.current.token_type == tt
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment as i32);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::Fun) {
            self.fun_declaration();
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        // TODO: global variables?
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::Equal) {
            self.expression();
        } else {
            // if the variable is not being initialized, set it to nil
            self.emit_instruction(Instruction::OpNil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );

        // TODO: global variables?
        self.define_variable(global);
    }

    fn fun_declaration(&mut self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn parse_variable(&mut self, error_message: &str) -> usize {
        self.consume(TokenType::Identifier, error_message);

        self.declare_variable();
        // TODO: current scope depth
        if self.current_compiler().scope_depth > 0 {
            return 0;
        }
        return self.identifier_constant(self.parser.previous);
    }

    fn identifier_constant(&mut self, name: Token) -> usize {
        return self.make_constant(Value::String(Rc::new(self.lexeme_to_string(name))));
    }

    // Add variable to the scope.
    fn declare_variable(&mut self) {
        // TODO: current scope depth
        if self.current_compiler().scope_depth == 0 {
            return;
        }

        let name: Token = self.parser.previous;

        let mut error = false;
        // error if trying to declare a local that is already declared in the same scope
        let scope_depth = self.current_compiler().scope_depth;
        for i in (0..self.current_compiler().locals.len()).rev() {
            let l = self.current_compiler().locals[i];
            if l.depth != -1 && l.depth < scope_depth {
                break;
            }

            if self.identifiers_equal(name, l.name) {
                error = true;
                break;
            }
        }

        if error {
            self.error("Already variable with this name in this scope.");
        }

        self.add_local(name);
    }

    /// The variable becomes available for use.
    fn define_variable(&mut self, global: usize) {
        // TODO: current scope depth
        if self.current_compiler().scope_depth > 0 {
            self.mark_initialized();
            return;
        }

        self.emit_instruction(Instruction::OpDefineGlobal(global));
    }

    /// Change the depth of the [Local] from -1 to the correct depth,
    /// indicating that the declaration statement has ended and the variable can now be used.
    fn mark_initialized(&mut self) {
        if self.current_compiler().scope_depth == 0 {
            // If mark_initialized is called from scope_depth 0,
            // it is a top-level function declaration.
            // The function will be a global variable in this case, so there is no local to mark.
            return;
        }
        let i = self.current_compiler().locals.len() - 1;
        self.current_compiler().locals[i].depth = self.current_compiler().scope_depth;
    }

    fn add_local(&mut self, name: Token) {
        if self.current_compiler().locals.len() as u8 == u8::MAX {
            self.error("Too many local variables in function.");
            return;
        }
        // When declaring a local, set the depth to -1, indicating it has not been initialized.
        self.current_compiler()
            .locals
            .push(Local { name, depth: -1 });
    }

    fn identifiers_equal(&self, t_1: Token, t_2: Token) -> bool {
        if t_1.length != t_2.length {
            return false;
        }
        for i in 0..t_1.length {
            let i = i as usize;
            if self.scanner.source[t_1.start + i] != self.scanner.source[t_2.start + i] {
                return false;
            }
        }
        return true;
    }

    /// Advance until one of a number of tokens is found, so that one error does not
    /// lead to a flood of redundant error messages.
    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.current.token_type != TokenType::Eof {
            if self.parser.previous.token_type == TokenType::Semicolon {
                return;
            }
            match self.parser.current.token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    fn init_compiler(&mut self, function_type: FunctionType) {
        let mut compiler = Compiler::new(function_type);
        // Reserve stack slot 0 for the Compiler's internal use, with placeholder values.
        compiler.locals.push(Local {
            name: Token {
                token_type: TokenType::Identifier,
                start: 0,
                length: 0,
                line: 0,
            },
            depth: 0,
        });
        self.compilers.push(compiler);
        self.current += 1;

        if function_type != FunctionType::Script {
            self.current_compiler().function.name = self.lexeme_to_string(self.parser.previous);
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::Print) {
            self.print_statement();
        } else if self.match_token(TokenType::For) {
            self.for_statement();
        } else if self.match_token(TokenType::If) {
            self.if_statement();
        } else if self.match_token(TokenType::While) {
            self.while_statement();
        } else if self.match_token(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        // Using a placeholder offset for the OpJumpIfFalse instruction.
        let then_jump = self.emit_jump(Instruction::OpJumpIfFalse(0xffff));
        // Pop the result of the if expression, if it was true, after it has been used by OpJumpIfFalse.
        self.emit_instruction(Instruction::OpPop);
        self.statement();

        // Using a placeholder offset for the OpJump instruction.
        let else_jump = self.emit_jump(Instruction::OpJump(0xffff));

        self.patch_jump(then_jump);
        // If the if expression was false, the result of the if expression was not popped earlier.
        // In that case, it is popped here.
        self.emit_instruction(Instruction::OpPop);

        if self.match_token(TokenType::Else) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn for_statement(&mut self) {
        // Starting new scope, in case the initializer declares a variable.
        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        // Left/Initializer clause.
        if self.match_token(TokenType::Semicolon) {
            // There is no initializer.
        } else if self.match_token(TokenType::Var) {
            self.var_declaration();
        } else {
            // The initializer may be an expression.
            self.expression_statement();
        }

        let mut loop_start = self.current_compiler().function.chunk.bytecode.len();
        let mut exit_jump = -1;
        // Middle/Test clause.
        if !self.match_token(TokenType::Semicolon) {
            self.expression();
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.");

            // If the middle clause is false exit the for loop.
            exit_jump = self.emit_jump(Instruction::OpJumpIfFalse(0xffff)) as i32;
            self.emit_instruction(Instruction::OpPop);
        }

        // Right/Increment clause.
        if !self.match_token(TokenType::RightParen) {
            let body_jump = self.emit_jump(Instruction::OpJump(0xfff));
            let increment_start = self.current_compiler().function.chunk.bytecode.len();
            self.expression();
            self.emit_instruction(Instruction::OpPop);
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        // Body
        self.statement();
        self.emit_loop(loop_start);

        // A jump instruction only exists if there is a middle clause.
        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_instruction(Instruction::OpPop);
        }

        self.end_scope();
    }

    fn while_statement(&mut self) {
        let loop_start = self.current_compiler().function.chunk.bytecode.len();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(Instruction::OpJumpIfFalse(0xffff));
        self.emit_instruction(Instruction::OpPop);
        self.statement();
        // jump back to the beginning
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_instruction(Instruction::OpPop);
    }

    /// Returns the offset of the emitted instruction in the chunk.
    fn emit_jump(&mut self, instruction: Instruction) -> usize {
        self.emit_instruction(instruction);
        self.current_compiler().function.chunk.bytecode.len() - 1
    }

    /// Put the correct number of instructions to jump over, if the if condition is false,
    /// now that the if block has been compiled.
    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_compiler().function.chunk.bytecode.len() - offset - 1;
        let instruction = match self.current_compiler().function.chunk.bytecode[offset] {
            Instruction::OpJump(_) => Some(Instruction::OpJump(jump)),
            Instruction::OpJumpIfFalse(_) => Some(Instruction::OpJumpIfFalse(jump)),
            _ => None,
        };
        self.current_compiler().function.chunk.bytecode[offset] = instruction.unwrap();
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_compiler().function.chunk.bytecode.len() - loop_start + 1;
        self.emit_instruction(Instruction::OpLoop(offset));
    }

    fn function(&mut self, function_type: FunctionType) {
        self.init_compiler(function_type);

        self.begin_scope();

        self.consume(TokenType::LeftParen, "Expect '(' after function name.");
        if !self.check(TokenType::RightParen) {
            loop {
                self.current_compiler().function.arity += 1;
                if self.current_compiler().function.arity > 255 {
                    self.error_at(self.parser.current, "Can't have more than 255 parameters.");
                }

                let constant = self.parse_variable("Expect parameter name.");
                self.define_variable(constant);

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.");
        self.consume(TokenType::LeftBrace, "Expect ')' after parameters.");
        self.block();

        let function = self.end();
        self.emit_constant(Value::Function(Rc::new(function)))
    }

    /// Compiles a function call.
    fn call(&mut self) {
        let arg_count = self.argument_list();
        self.emit_instruction(Instruction::OpCall(arg_count));
    }

    fn argument_list(&mut self) -> usize {
        let mut arg_count: usize = 0;

        if !self.check(TokenType::RightParen) {
            loop {
                self.expression();
                if arg_count == 255 {
                    self.error("Can't have more than 255 arguments.");
                }
                arg_count += 1;

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::LeftBrace, "Expect ')' after arguments.");

        arg_count
    }

    fn and(&mut self) {
        let end_jump = self.emit_jump(Instruction::OpJumpIfFalse(0xffff));
        self.emit_instruction(Instruction::OpPop);
        self.parse_precedence(Precedence::And as i32);
        self.patch_jump(end_jump);
    }

    fn or(&mut self) {
        let else_jump = self.emit_jump(Instruction::OpJumpIfFalse(0xffff));
        let end_jump = self.emit_jump(Instruction::OpJump(0xffff));

        self.patch_jump(else_jump);
        self.emit_instruction(Instruction::OpPop);

        self.parse_precedence(Precedence::Or as i32);
        self.patch_jump(end_jump);
    }

    fn block(&mut self) {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration();
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_instruction(Instruction::OpPop);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_instruction(Instruction::OpPrint);
    }

    fn number(&mut self) {
        // TODO: lexeme handling?
        let value = self
            .lexeme_to_string(self.parser.previous)
            .parse::<f64>()
            .unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous, can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op: Instruction;
        let set_op: Instruction;
        let mut arg = self.resolve_local(name);
        if arg != -1 {
            // If a local variable with the given name exists, this is a local variable.
            get_op = Instruction::OpGetLocal(arg as usize);
            set_op = Instruction::OpSetLocal(arg as usize);
        } else {
            // If it does not exist, it should be a global variable.
            arg = self.identifier_constant(name) as i32;
            get_op = Instruction::OpGetGlobal(arg as usize);
            set_op = Instruction::OpSetGlobal(arg as usize);
        };

        if can_assign && self.match_token(TokenType::Equal) {
            self.expression();
            self.emit_instruction(set_op);
        } else {
            self.emit_instruction(get_op);
        }
    }

    /// Returns the index of the local variable in the locals vector.
    fn resolve_local(&mut self, name: Token) -> i32 {
        // let mut err = false;
        for i in (0..self.current_compiler().locals.len()).rev() {
            let l = self.current_compiler().locals[i];
            if self.identifiers_equal(l.name, name) {
                if l.depth == -1 {
                    self.error("Can't read local variable in its own initializer.");
                }
                return i as i32;
            }
        }
        return -1;
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn string(&mut self) {
        // Copy the string from the source string, without the quote marks.
        let s = self.section_to_string(
            self.parser.previous.start + 1,
            (self.parser.previous.length - 2) as usize,
        );
        let v: Value = Value::String(Rc::new(s));
        self.emit_constant(v);
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.token_type;

        self.parse_precedence(Precedence::Unary as i32);

        match operator_type {
            TokenType::Bang => self.emit_instruction(Instruction::OpNot),
            TokenType::Minus => self.emit_instruction(Instruction::OpNegate),
            _ => {}
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.token_type;
        let rule: ParseRule = CompilerManager::rules(operator_type);
        let precedence = rule.precedence as i32 + 1;
        self.parse_precedence(precedence);

        match operator_type {
            TokenType::BangEqual => {
                self.emit_instructions(Instruction::OpEqual, Instruction::OpNot)
            }
            TokenType::EqualEqual => self.emit_instruction(Instruction::OpEqual),
            TokenType::Greater => self.emit_instruction(Instruction::OpGreater),
            TokenType::GreaterEqual => {
                self.emit_instructions(Instruction::OpLess, Instruction::OpNot)
            }
            TokenType::Less => self.emit_instruction(Instruction::OpLess),
            TokenType::LessEqual => {
                self.emit_instructions(Instruction::OpGreater, Instruction::OpNot)
            }
            TokenType::Plus => self.emit_instruction(Instruction::OpAdd),
            TokenType::Minus => self.emit_instruction(Instruction::OpSubtract),
            TokenType::Star => self.emit_instruction(Instruction::OpMultiply),
            TokenType::Slash => self.emit_instruction(Instruction::OpDivide),
            _ => return,
        }
    }

    fn literal(&mut self) {
        let operator_type = self.parser.previous.token_type;

        match operator_type {
            TokenType::False => self.emit_instruction(Instruction::OpFalse),
            TokenType::Nil => self.emit_instruction(Instruction::OpNil),
            TokenType::True => self.emit_instruction(Instruction::OpTrue),
            _ => return,
        }
    }

    fn parse_fn(&mut self, parse_fn: ParseFn, can_assign: bool) {
        match parse_fn {
            ParseFn::Call => self.call(),
            ParseFn::Grouping => self.grouping(),
            // ParseFn::Dot => ,
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::String => self.string(),
            ParseFn::Number => self.number(),
            ParseFn::And => self.and(),
            ParseFn::Literal => self.literal(),
            ParseFn::Or => self.or(),
            // ParseFn::Super => ,
            // ParseFn::This => ,
            // ParseFn::None => ,
            ParseFn::None => (),
        }
    }

    fn rules(token_type: TokenType) -> ParseRule {
        return match token_type {
            TokenType::LeftParen => ParseRule {
                prefix: ParseFn::Grouping,
                infix: ParseFn::Call,
                precedence: Precedence::Call,
            },
            TokenType::RightParen => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::LeftBrace => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::RightBrace => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Comma => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Dot => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Minus => ParseRule {
                prefix: ParseFn::Unary,
                infix: ParseFn::Binary,
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Slash => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: ParseFn::Unary,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Equality,
            },
            TokenType::Equal => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::EqualEqual => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Comparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Comparison,
            },
            TokenType::Less => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Comparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Binary,
                precedence: Precedence::Comparison,
            },
            TokenType::Identifier => ParseRule {
                prefix: ParseFn::Variable,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: ParseFn::String,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Number => ParseRule {
                prefix: ParseFn::Number,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::And => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::And,
                precedence: Precedence::And,
            },
            TokenType::Class => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Else => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::False => ParseRule {
                prefix: ParseFn::Literal,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::For => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Fun => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::If => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Nil => ParseRule {
                prefix: ParseFn::Literal,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Or => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::Or,
                precedence: Precedence::Or,
            },
            TokenType::Print => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Return => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Super => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::This => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::True => ParseRule {
                prefix: ParseFn::Literal,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Var => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::While => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Error(_) => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::Eof => ParseRule {
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
        };
    }
}
