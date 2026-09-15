// The lexer's whole job is to turn raw source text into a flat list of
// tokens, without worrying yet about whether that sequence actually
// makes sense grammatically. That correctness check is the parser's
// problem, not ours. Keeping this boundary clean means the lexer stays
// simple and easy to test in isolation.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    Number(f64),
    Equals,
    Plus,
    LeftBrace,
    RightBrace,
    Eof,
}

// We attach a span to every token from the very start rather than
// bolting it on later, because the blueprint calls for diagnostics with
// real file/line/column information, and that's much harder to retrofit
// once the lexer already exists without it.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, position: 0 }
    }

    // Walks the source once and produces every token up front. This is
    // simple to reason about for a first pass; we can switch to a lazy,
    // pull-based iterator later if performance ever demands it.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = self.source.chars().collect();

        while self.position < chars.len() {
            let ch = chars[self.position];
            let start = self.position;

            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.position += 1;
                    continue;
                }
                '=' => {
                    tokens.push(Token { kind: TokenKind::Equals, start, end: start + 1 });
                    self.position += 1;
                }
                '+' => {
                    tokens.push(Token { kind: TokenKind::Plus, start, end: start + 1 });
                    self.position += 1;
                }
                '{' => {
                    tokens.push(Token { kind: TokenKind::LeftBrace, start, end: start + 1 });
                    self.position += 1;
                }
                '}' => {
                    tokens.push(Token { kind: TokenKind::RightBrace, start, end: start + 1 });
                    self.position += 1;
                }
                c if c.is_ascii_digit() => {
                    let mut end = self.position;
                    while end < chars.len() && chars[end].is_ascii_digit() {
                        end += 1;
                    }
                    let text: String = chars[self.position..end].iter().collect();
                    tokens.push(Token {
                        kind: TokenKind::Number(text.parse().unwrap_or(0.0)),
                        start,
                        end,
                    });
                    self.position = end;
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut end = self.position;
                    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                        end += 1;
                    }
                    let text: String = chars[self.position..end].iter().collect();
                    tokens.push(Token { kind: TokenKind::Identifier(text), start, end });
                    self.position = end;
                }
                _ => {
                    // Unknown character. For now we just skip it silently
                    // rather than crashing, but this is exactly the kind
                    // of spot that will need a real diagnostic system
                    // once semantic analysis lands.
                    self.position += 1;
                }
            }
        }

        tokens.push(Token { kind: TokenKind::Eof, start: self.position, end: self.position });
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_simple_assignment() {
        let mut lexer = Lexer::new("speed = 250");
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("speed".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Equals);
        assert_eq!(tokens[2].kind, TokenKind::Number(250.0));
    }
}
