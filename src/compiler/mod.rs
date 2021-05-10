mod parser;
mod scanner;
use crate::chunk::{Chunk, Instruction};

use self::{
    parser::Parser,
    scanner::{Scanner, Token, TokenType},
};

pub struct Compiler {
    current_chunk: Chunk,
    scanner: Scanner,
    parser: Parser,
}

impl Compiler {
    pub fn compile(source: String) -> Result<Chunk, ()> {
        let mut compiler = Compiler::init(source.chars().collect());

        compiler.advance();
        // compiler.expression();
        // compiler.consume();

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

    fn advance(&mut self) {
        self.parser.previous = self.parser.current;

        loop {
            self.parser.current = self.scanner.scan_token();

            // Report and skip all error tokens, so that the rest of the parser only sees valid ones.
            match self.parser.current.token_type {
                TokenType::Error(_) => {}
                _ => break,
            }
        }
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
            _ => eprint!(
                " at {:?}",
                self.scanner.source[token.start..token.length as usize].iter()
            ),
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

    fn emit_instructions(&mut self, instruction_1: Instruction, instruction_2: Instruction) {
        self.emit_instruction(instruction_1);
        self.emit_instruction(instruction_2);
    }

    fn end(&mut self) {
        self.emit_instruction(Instruction::OpReturn);
    }
}
