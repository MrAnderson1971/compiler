use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::SyntaxError;
use crate::lexer::Token;
use std::collections::VecDeque;
use std::fmt::Debug;

pub struct Parser {
    loop_label_counter: i32,
    tokens: VecDeque<Token>,
    line_number: Position,
}

impl Parser {
    fn get_token_and_advance(&mut self) -> Result<Token, CompilerError> {
        if let Some(next) = self.tokens.pop_front() {
            Ok(next)
        } else {
            Err(SyntaxError("Unexpected EOF".to_string()))
        }
    }

    fn peek_token(&self) -> Result<Token, CompilerError> {
        if let Some(next) = self.tokens.front() {
            Ok(next.clone())
        } else {
            Err(SyntaxError("Unexpected EOF".to_string()))
        }
    }

    fn get_token_and_advance_expected<T>(&mut self) -> Result<T, CompilerError>
    where
        T: TryFrom<Token, Error = CompilerError>,
    {
        T::try_from(self.get_token_and_advance()?)
    }

    fn get_token_and_advance_expected_value<T>(&mut self, expected: T) -> Result<T, CompilerError>
    where
        T: TryFrom<Token, Error = CompilerError> + Eq + Debug
    {
        let result: T = self.get_token_and_advance_expected::<T>()?;
        if result != expected {
            Err(SyntaxError(format!(
                "Expected {:?} but got {:?} at {:?}",
                expected,
                result,
                self.line_number.clone()
            )))
        } else {
            Ok(result)
        }
    }
}
