use gameforge_ast::{Assignment, BinaryOp, EntityDecl, Expr, FieldDecl, Item, Stmt};
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

    pub fn parse(&mut self) -> Result<Vec<Item>, ParseError> {
        let mut items = Vec::new();
        while !self.at_eof() {
            items.push(self.parse_item()?);
        }
        Ok(items)
    }

    // "entity" isn't a real keyword at the lexer level — this just checks
    // whether an identifier's text happens to match. A top-level variable
    // literally named `entity` would be misread as a declaration start.
    // Harmless today since nothing generates that, but it needs fixing
    // with a real keyword table once "system" exists as a second keyword
    // and this stops being a one-off special case.
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        if let TokenKind::Identifier(name) = &self.current().kind {
            if name == "entity" {
                return Ok(Item::Entity(self.parse_entity_decl()?));
            }
        }
        Ok(Item::Statement(self.parse_statement()?))
    }

    fn parse_entity_decl(&mut self) -> Result<EntityDecl, ParseError> {
        self.advance(); // consume 'entity'
        let name = match self.current().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("expected entity name after 'entity'")),
        };

        self.expect(TokenKind::LeftBrace, "expected '{' after entity name")?;

        let mut fields = Vec::new();
        while self.current().kind != TokenKind::RightBrace {
            if self.at_eof() {
                return Err(self.error("unterminated entity body, expected '}'"));
            }
            fields.push(self.parse_field_decl()?);
        }
        self.advance(); // consume '}'

        Ok(EntityDecl { name, fields })
    }

    fn parse_field_decl(&mut self) -> Result<FieldDecl, ParseError> {
        let name = match self.current().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                name
            }
            _ => return Err(self.error("expected field name")),
        };

        if self.current().kind == TokenKind::Colon {
            self.advance();
            let type_name = match self.current().kind.clone() {
                TokenKind::Identifier(type_name) => {
                    self.advance();
                    type_name
                }
                _ => return Err(self.error("expected type name after ':'")),
            };
            let default = if self.current().kind == TokenKind::Equals {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            Ok(FieldDecl { name, type_name: Some(type_name), default })
        } else if self.current().kind == TokenKind::Equals {
            self.advance();
            let default = self.parse_expression()?;
            Ok(FieldDecl { name, type_name: None, default: Some(default) })
        } else {
            Ok(FieldDecl { name, type_name: None, default: None })
        }
    }

    // Only `identifier = expr` exists at the top level. Dotted paths and
    // compound assignment (`player.transform.position.x += ...`) need
    // grammar this parser doesn't have yet.
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

    fn parse_source(src: &str) -> Result<Vec<Item>, ParseError> {
        let tokens = Lexer::new(src).tokenize();
        Parser::new(tokens).parse()
    }

    #[test]
    fn parses_simple_assignment() {
        let items = parse_source("speed = 250").unwrap();
        assert_eq!(
            items[0],
            Item::Statement(Stmt::Assignment(Assignment {
                name: "speed".to_string(),
                value: Expr::Number(250.0),
            }))
        );
    }

    #[test]
    fn parses_addition() {
        let items = parse_source("total = 1 + 2").unwrap();
        assert_eq!(
            items[0],
            Item::Statement(Stmt::Assignment(Assignment {
                name: "total".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Number(1.0)),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(2.0)),
                },
            }))
        );
    }

    #[test]
    fn missing_equals_is_an_error() {
        assert!(parse_source("speed 250").is_err());
    }

    #[test]
    fn parses_entity_with_typed_field_and_default() {
        let items = parse_source("entity Player { speed: Float = 250 }").unwrap();
        assert_eq!(
            items[0],
            Item::Entity(EntityDecl {
                name: "Player".to_string(),
                fields: vec![FieldDecl {
                    name: "speed".to_string(),
                    type_name: Some("Float".to_string()),
                    default: Some(Expr::Number(250.0)),
                }],
            })
        );
    }

    #[test]
    fn parses_entity_with_marker_field() {
        let items = parse_source("entity Player { transform }").unwrap();
        assert_eq!(
            items[0],
            Item::Entity(EntityDecl {
                name: "Player".to_string(),
                fields: vec![FieldDecl {
                    name: "transform".to_string(),
                    type_name: None,
                    default: None,
                }],
            })
        );
    }

    #[test]
    fn parses_entity_with_untyped_field_with_default() {
        let items = parse_source("entity Player { hp = 100 }").unwrap();
        assert_eq!(
            items[0],
            Item::Entity(EntityDecl {
                name: "Player".to_string(),
                fields: vec![FieldDecl {
                    name: "hp".to_string(),
                    type_name: None,
                    default: Some(Expr::Number(100.0)),
                }],
            })
        );
    }

    #[test]
    fn parses_entity_with_multiple_fields() {
        let items =
            parse_source("entity Player { transform speed: Float = 250 hp = 100 }").unwrap();
        let Item::Entity(entity) = &items[0] else {
            panic!("expected an entity item");
        };
        assert_eq!(entity.fields.len(), 3);
    }

    #[test]
    fn missing_closing_brace_is_an_error() {
        assert!(parse_source("entity Player { speed: Float = 250").is_err());
    }

    #[test]
    fn entity_and_assignment_can_appear_in_the_same_file() {
        let items = parse_source("entity Player { transform } total = 1 + 2").unwrap();
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], Item::Entity(_)));
        assert!(matches!(items[1], Item::Statement(_)));
    }
    }
