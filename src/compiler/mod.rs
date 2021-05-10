mod parser;
mod scanner;
use crate::chunk::Chunk;

use self::{parser::Parser, scanner::{Scanner, Token, TokenType}};

pub struct Compiler {
    chunk: Chunk,
    scanner: Scanner,
    parser: Parser,
}

impl Compiler {
    pub fn compile(source: String) -> Result<Chunk, ()> {
        let mut compiler = Compiler::init(source.chars().collect());

        compiler.advance();
        // compiler.expression();
        // compiler.consume();

        Ok(compiler.chunk)
    }

    fn init(source: Vec<char>) -> Compiler {
        Compiler {
            chunk: Chunk::init(),
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
                TokenType::Error(error_message) => {}
                _ => break,
            }
        }
    }

    // fn error(&self) {
    //     error_at(self.parser.previous);
    // }

    fn error_at(&self, token: &Token, message: String) {
        eprint!("[line {}] Error", token.line);

        match &token.token_type {
            TokenType::Eof => {}
            TokenType::Error(error_message) => {}
            _ => eprintln!(" at {:?}", self.scanner.source[token.start..token.length as usize].iter()),
        }
    }
}
