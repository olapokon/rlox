pub struct Scanner {
    /// The source input, as a [Vec] of [char]s.
    pub source: Vec<char>,
    /// The index in the source of the first character of the token currently being scanned.
    pub start: usize,
    /// The index in the source of the character currently being scanned.
    pub current: usize,
    /// The number of the line currently being scanned.
    pub line: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String,
    Number,
    // Keywords.
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Error.
    Error(ScannerError),

    // End of file.
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScannerError {
    UnexpectedCharacter,
    UnterminatedString,
    UninitializedToken,
}

#[derive(Clone, Copy)]
pub struct Token {
    pub token_type: TokenType,
    /// The token's start index in the source string.
    pub start: usize,
    pub length: i32,
    /// The line in the source code where the [Token] is found.
    pub line: i32,
}

impl Scanner {
    pub fn init(mut source: Vec<char>) -> Scanner {
        source.push('\0');
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        return match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            '"' => self.string(),
            c if is_digit(c) => self.number(),
            c if is_alpha(c) => self.identifier(),

            _ => self.make_token(TokenType::Error(ScannerError::UnexpectedCharacter)),
        };
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            start: self.start,
            length: (self.current - self.start) as i32,
            line: self.line,
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn peek(&self) -> char {
        self.source[self.current]
    }

    fn peek_next(&self) -> char {
        return if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current + 1]
        };
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.is_at_end() {
                return;
            }
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        // self.source.len() == self.current
        self.source[self.current] == '\0'
    }

    fn string(&mut self) -> Token {
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1
            };
            self.advance();
        }

        if self.is_at_end() {
            // TODO: fix error message
            return self.make_token(TokenType::Error(ScannerError::UnterminatedString));
        }

        self.advance(); // closing quote

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        return match self.source[self.start] {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            'f' if self.current - self.start > 1usize => match self.source[self.start + 1] {
                'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                'o' => self.check_keyword(2, 1, "r", TokenType::For),
                'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                _ => TokenType::Identifier,
            },
            't' if self.current - self.start > 1usize => match self.source[self.start + 1] {
                'h' => self.check_keyword(2, 2, "is", TokenType::This),
                'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                _ => TokenType::Identifier,
            },
            _ => TokenType::Identifier,
        };
    }

    fn check_keyword(
        &self,
        start: i32,
        length: i32,
        rest: &str,
        token_type: TokenType,
    ) -> TokenType {
        if (self.current - self.start) as i32 != start + length {
            return TokenType::Identifier;
        }
        for (&c1, c2) in self.source[(self.start + start as usize)..self.current]
            .iter()
            .zip(rest.chars())
        {
            if c1 != c2 {
                return TokenType::Identifier;
            }
        }
        token_type
    }
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_number() {
        let source = "84".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(TokenType::Number, t.token_type);
    }

    #[test]
    fn scan_true_keyword() {
        let source = "true;".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(TokenType::True, t.token_type);
    }

    #[test]
    fn scan_equal_equal() {
        let source = "==".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(TokenType::EqualEqual, t.token_type);
    }

    #[test]
    fn scan_string() {
        let source = "\"asda\"".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(TokenType::String, t.token_type);
    }

    #[test]
    fn scan_unterminated_string() {
        let source = "\"asda".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(
            TokenType::Error(ScannerError::UnterminatedString),
            t.token_type
        );
    }

    #[test]
    fn scan_identifier() {
        let source = "asda".chars().collect();
        let mut sc = Scanner::init(source);
        let t = sc.scan_token();
        assert_eq!(TokenType::Identifier, t.token_type);
    }
}
