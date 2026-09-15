// Expr covers exactly what the lexer can currently produce — numbers,
// identifiers, and '+' — and nothing else. Adding node types nobody
// parses yet just means dead code sitting around until it's needed.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
}

// `speed: Float = 250` from the language design sample is the only
// statement shape we've actually written source for so far, so
// Assignment is the only variant. Entities, systems, and events each
// get their own Stmt variant once the parser is actually building one.
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assignment(Assignment),
}
