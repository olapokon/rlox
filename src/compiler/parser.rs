use super::scanner::{ScannerError, Token, TokenType};


pub struct Parser {
    pub current: Token,
    pub previous: Token,
}

impl Parser {
    pub fn init() -> Parser {
        let placeholder_token = Token {
            token_type: TokenType::Error(ScannerError::UninitializedToken),
            start: 0,
            length: 0,
            line: 0,
        };
        Parser {
            current: placeholder_token,
            previous: placeholder_token,
        }
    }
}
