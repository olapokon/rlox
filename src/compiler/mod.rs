mod parser;
mod scanner;
use core::f64;

use crate::{
    chunk::{Chunk, Instruction},
    value::Value,
};

use self::{
    parser::Parser,
    scanner::{Scanner, Token, TokenType},
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
    // Call,
    Grouping,
    // Dot,
    Unary,
    Binary,
    // Variable,
    // String,
    Number,
    // And,
    Literal,
    // Or,
    // Super,
    // This,
    None,
}

struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

pub struct Compiler {
    current_chunk: Chunk,
    scanner: Scanner,
    parser: Parser,
}

impl Compiler {
    pub fn compile(source: String) -> Result<Chunk, ()> {
        let mut compiler = Compiler::init(source.chars().collect());

        compiler.advance();
        compiler.expression();
        compiler.consume(TokenType::Eof, "Expect end of expression.");

        compiler.end();

        if compiler.parser.had_error {
            Err(())
        } else {
            Ok(compiler.current_chunk)
        }
    }

    fn init(source: Vec<char>) -> Compiler {
        Compiler {
            current_chunk: Chunk::init(),
            scanner: Scanner::init(source),
            parser: Parser::init(),
        }
    }

    fn lexeme_to_string(&self, token: Token) -> String {
        self.scanner.source[token.start..(token.start + token.length as usize)]
            .iter()
            .collect()
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();

            // Report and skip all error tokens, so that the rest of the parser only sees valid ones.
            match self.parser.current.token_type {
                TokenType::Error(_) => self.error_at(
                    self.parser.current,
                    &self.lexeme_to_string(self.parser.current),
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

        eprintln!(": {}", message);
        self.parser.had_error = true;
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.parser.current.token_type == token_type {
            self.advance();
            return;
        }
        self.error_at(self.parser.current, message);
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.current_chunk
            .write(instruction, self.parser.previous.line);
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
        let constant_index = self.current_chunk.add_constant(value);
        if constant_index as u8 > u8::MAX {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant_index
    }

    fn end(&mut self) {
        self.emit_instruction(Instruction::OpReturn);

        //
        //
        // TODO: conditional compilation
        if !self.parser.had_error {
            self.current_chunk.disassemble("code");
        }
        //
        //
        //
    }

    /// Takes [Precedence] converted to i32.
    // TODO: refactor Precedence?
    fn parse_precedence(&mut self, precedence: i32) {
        self.advance();
        let prefix_rule = Compiler::rules(self.parser.previous.token_type);
        if prefix_rule.prefix == ParseFn::None {
            self.error("Expect expression.");
            return;
        }

        self.parse_fn(prefix_rule.prefix);

        while precedence <= Compiler::rules(self.parser.current.token_type).precedence as i32 {
            self.advance();
            let infix_rule = Compiler::rules(self.parser.previous.token_type);
            self.parse_fn(infix_rule.infix);
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment as i32);
    }

    fn number(&mut self) {
        // TODO: lexeme handling?
        let value = self
            .lexeme_to_string(self.parser.previous)
            .parse::<f64>()
            .unwrap();
        self.emit_constant(Value::Number(value));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
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
        let rule: ParseRule = Compiler::rules(operator_type);
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

    fn parse_fn(&mut self, parse_fn: ParseFn) {
        match parse_fn {
            // ParseFn::Call => ,
            ParseFn::Grouping => self.grouping(),
            // ParseFn::Dot => ,
            ParseFn::Unary => self.unary(),
            ParseFn::Binary => self.binary(),
            // ParseFn::Variable => ,
            // ParseFn::String => ,
            ParseFn::Number => self.number(),
            // ParseFn::And => ,
            ParseFn::Literal => self.literal(),
            // ParseFn::Or => ,
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
                infix: ParseFn::None,
                precedence: Precedence::None,
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
                prefix: ParseFn::None,
                infix: ParseFn::None,
                precedence: Precedence::None,
            },
            TokenType::String => ParseRule {
                prefix: ParseFn::None,
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
                infix: ParseFn::None,
                precedence: Precedence::None,
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
                infix: ParseFn::None,
                precedence: Precedence::None,
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
