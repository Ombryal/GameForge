use gameforge_ast::{Expr, Stmt};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndefinedName { name: String },
}

pub fn analyze(statements: &[Stmt]) -> Result<(), Vec<SemanticError>> {
    let mut declared: HashSet<String> = HashSet::new();
    let mut errors = Vec::new();

    for stmt in statements {
        match stmt {
            Stmt::Assignment(assign) => {
                check_expr(&assign.value, &declared, &mut errors);
                declared.insert(assign.name.clone());
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_expr(expr: &Expr, declared: &HashSet<String>, errors: &mut Vec<SemanticError>) {
    match expr {
        Expr::Number(_) => {}
        Expr::Identifier(name) => {
            if !declared.contains(name) {
                errors.push(SemanticError::UndefinedName { name: name.clone() });
            }
        }
        Expr::Binary { left, right, .. } => {
            check_expr(left, declared, errors);
            check_expr(right, declared, errors);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gameforge_ast::{Assignment, BinaryOp};

    #[test]
    fn undefined_identifier_is_rejected() {
        let stmts = vec![Stmt::Assignment(Assignment {
            name: "total".into(),
            value: Expr::Identifier("missing".into()),
        })];
        let result = analyze(&stmts);
        assert_eq!(
            result,
            Err(vec![SemanticError::UndefinedName { name: "missing".into() }])
        );
    }

    #[test]
    fn identifier_defined_earlier_is_accepted() {
        let stmts = vec![
            Stmt::Assignment(Assignment { name: "x".into(), value: Expr::Number(1.0) }),
            Stmt::Assignment(Assignment {
                name: "y".into(),
                value: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".into())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(1.0)),
                },
            }),
        ];
        assert_eq!(analyze(&stmts), Ok(()));
    }
}
