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

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assignment(Assignment),
}

/// A single field inside an entity declaration.
///
/// `type_name` and `default` are independently optional to cover every
/// shape the parser currently accepts without four separate structs:
/// `speed: Float = 250` (typed, defaulted), `speed: Float` (typed, no
/// default), `hp = 100` (untyped, defaulted), and a bare `transform`
/// marker field (neither).
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub type_name: Option<String>,
    pub default: Option<Expr>,
}

/// `entity Player { speed: Float = 250 }`
#[derive(Debug, Clone, PartialEq)]
pub struct EntityDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

/// A top-level thing a GameForge file can contain. Only two shapes exist
/// because those are the only two the parser currently produces —
/// `system` blocks will be a third variant here, not a special case of
/// either of these.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Statement(Stmt),
    Entity(EntityDecl),
}
