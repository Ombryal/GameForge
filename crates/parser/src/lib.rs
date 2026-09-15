use gameforge_ast::{Assignment, BinaryOp, Expr, Stmt};
use gameforge_lexer::{Token, TokenKind};

// One error variant for now. There's only one real failure mode in a
// grammar this small — expected some token, found something else — so
// splitting this into a proper diagnostics enum before the grammar has
// enough shapes to actually confuse each other would just be guessing.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.at_eof() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    // Only `identifier = expr` exists right now. The `player.transform.
    // position.x += ...` line from the language design sample needs
    // dotted paths and compound assignment, neither of which the lexer
    // produces yet, so that's out of scope until it does.
    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let name = match self.current().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("expected identifier at start of statement")),
        };

        self.expect(TokenKind::Equals, "expected '=' after identifier")?;
        let value = self.parse_expression()?;

        Ok(Stmt::Assignment(Assignment { name, value }))
    }

    // Left-associative '+' only, no precedence table. There's nothing to
    // rank against yet since '+' is the only operator the lexer emits.
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;
        while self.current().kind == TokenKind::Plus {
            self.advance();
            let right = self.parse_primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().kind.clone() {
            TokenKind::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Identifier(name))
            }
            _ => Err(self.error("expected a number or identifier")),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<(), ParseError> {
        if self.current().kind == kind {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.current().start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gameforge_lexer::Lexer;

    fn parse_source(src: &str) -> Result<Vec<Stmt>, ParseError> {
        let tokens = Lexer::new(src).tokenize();
        Parser::new(tokens).parse()
    }

    #[test]
    fn parses_simple_assignment() {
        let stmts = parse_source("speed = 250").unwrap();
        assert_eq!(
            stmts[0],
            Stmt::Assignment(Assignment {
                name: "speed".to_string(),
                value: Expr::Number(250.0),
            })
        );
    }

    #[test]
    fn parses_addition() {
        let stmts = parse_source("total = 1 + 2").unwrap();
        assert_eq!(
            stmts[0],
            Stmt::Assignment(Assignment {
                name: "total".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Number(1.0)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(2.0)),
                },
            })
        );
    }

    #[test]
    fn missing_equals_is_an_error() {
        assert!(parse_source("speed 250").is_err());
    }
}
