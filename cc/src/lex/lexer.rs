use crate::{common::{Error, RawToken, Result, Span, Token}, span};


/// Lexer can only ensure proper tokenization for valid ASCII characters, for now.
#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    has_error: bool,
    cur_line: usize,
    cur_column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let input: Vec<char> = input.chars().collect();
        Self {
            input,
            position: 0,
            has_error: false,
            cur_line: 1,
            cur_column: 1,
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>> {
        let mut tokens = vec![];
        let mut errors = vec![];
        loop {
            match self.next_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {
                    assert!(self.is_at_end());
                    break;
                }
                Err(e) => {
                    errors.push(e);
                    self.has_error = true;
                }
            }
        }
        if self.has_error {
            Err(Error::Errors(errors.into()))
        } else {
            Ok(tokens)
        }
    }

    fn is_at_end(&self) -> bool { 
        self.position >= self.input.len()
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            let current_char = self.input[self.position];
            self.position += 1;
            
            if current_char == '\n' {
                self.cur_line += 1;
                self.cur_column = 1;
            } else {
                self.cur_column += 1;
            }

            Some(current_char)
        }
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 >= self.input.len() {
            None
        } else {
            Some(self.input[self.position + 1])
        }
    }

    /// Advances the lexer until the predicate returns true or the end of input is reached.
    /// Returns: .0 Span of the characters consumed, .1: start position of the span.
    fn advance_while<P>(&mut self, predicate: P) -> (Span, usize)
    where
        P: Fn(char) -> bool,
    {
        let start_position = self.position;
        let start_line = self.cur_line;
        let start_column = self.cur_column;
        while let Some(c) = self.peek() {
            if predicate(c) {
                self.advance();
            } else {
                break;
            }
        }
        (span!(
            start_line,
            start_column,
            self.position - start_position
        ), start_position)
    }

    /*
    Following methods are for lexing specific tokens.
    They will be called by `next_token` method, which is the main entry point for
    lexing tokens from the input.
    The caller must make sure the first character is valid for the token type.
    */

    fn integer(&mut self) -> Result<Token> {
        let (span, start_position) = self.advance_while(|c| c.is_digit(10));        
        
        let integer_str: String = self.input[start_position..self.position].iter().collect();
        let integer_val = integer_str.parse::<i64>()
            .map_err(|e| super::Error::new(&format!("Invalid integer: {e:?}")))?;
        
        let raw = RawToken::Integer(integer_val);

        Ok(Token::new(raw, span))
    }

    fn identifier(&mut self) -> Result<Token> {
        let (span, start_position) = self.advance_while(|c| 
            c.is_alphanumeric() || c == '_'
        );

        let identifier_str: String = self.input[start_position..self.position].iter().collect();

        match identifier_str.as_str() {
            "return" => Ok(Token::new(RawToken::Return, span)),
            "int" => Ok(Token::new(RawToken::Int, span)),
            "void" => Ok(Token::new(RawToken::Void, span)),
            _ => Ok(Token::new(RawToken::Identifier(identifier_str), span)),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        use RawToken::*;

        // skip whitespace
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => {
                    self.cur_column += 1;
                    self.position += 1;
                },
                '\n' => {
                    self.cur_line += 1;
                    self.cur_column = 1;
                    self.position += 1;
                },
                _ => break,
            }
        }

        if self.is_at_end() {
            return Ok(None);
        }

        let cur_char = self.peek().unwrap();
        let start_line = self.cur_line;
        let start_column = self.cur_column;
        
        let raw = match cur_char {
            '(' =>  {
                self.advance();
                LParen
            },
            ')' => {
                self.advance();
                RParen
            },
            '{' => {
                self.advance();
                LBrace
            },
            '}' => {
                self.advance();
                RBrace
            },
            ';' => {
                self.advance();
                Semicolon
            },
            '0'..='9' => return self.integer().map(|token| Some(token)),
            'a'..='z' | 'A'..='Z' | '_' => return self.identifier().map(|token| Some(token)),
            _ => {
                self.has_error = true;
                self.advance();
                return Err(super::Error::new(&format!("Unexpected character: '{}'", cur_char)).into());
            }
        };
        let span = span!(start_line, start_column);

        Ok(Some(Token::new(raw, span)))
    }
}