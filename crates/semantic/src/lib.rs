use gameforge_ast::{EntityDecl, Expr, Item, Stmt};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndefinedName { name: String },
    DuplicateEntity { name: String },
    DuplicateField { entity: String, field: String },
    UnknownType { entity: String, field: String, type_name: String },
}

/// Type names the language currently understands. Deliberately small —
/// grows only once the runtime actually has a representation for a new
/// type, not ahead of that.
const KNOWN_TYPES: [&str; 4] = ["Float", "Int", "Bool", "String"];

pub fn analyze(items: &[Item]) -> Result<(), Vec<SemanticError>> {
    let mut declared: HashSet<String> = HashSet::new();
    let mut entity_names: HashSet<String> = HashSet::new();
    let mut errors = Vec::new();

    for item in items {
        match item {
            Item::Statement(Stmt::Assignment(assign)) => {
                check_expr(&assign.value, &declared, &mut errors);
                declared.insert(assign.name.clone());
            }
            Item::Entity(entity) => {
                if !entity_names.insert(entity.name.clone()) {
                    errors.push(SemanticError::DuplicateEntity { name: entity.name.clone() });
                }
                check_entity(entity, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn check_entity(entity: &EntityDecl, errors: &mut Vec<SemanticError>) {
    let mut seen_fields = HashSet::new();
    for field in &entity.fields {
        if !seen_fields.insert(field.name.clone()) {
            errors.push(SemanticError::DuplicateField {
                entity: entity.name.clone(),
                field: field.name.clone(),
            });
        }

        if let Some(type_name) = &field.type_name {
            if !KNOWN_TYPES.contains(&type_name.as_str()) {
                errors.push(SemanticError::UnknownType {
                    entity: entity.name.clone(),
                    field: field.name.clone(),
                    type_name: type_name.clone(),
                });
            }
        }

        // A field default referencing another field on the same entity,
        // or `self`, isn't checked against anything here — entities
        // don't have their own name scope defined yet, only the file's
        // top-level variables do. That's a real gap, not a decision to
        // allow it; it needs entity-scoped resolution before it can be
        // validated instead of silently ignored.
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
    use gameforge_ast::{Assignment, BinaryOp, FieldDecl};

    #[test]
    fn undefined_identifier_is_rejected() {
        let items = vec![Item::Statement(Stmt::Assignment(Assignment {
            name: "total".into(),
            value: Expr::Identifier("missing".into()),
        }))];
        assert_eq!(
            analyze(&items),
            Err(vec![SemanticError::UndefinedName { name: "missing".into() }])
        );
    }

    #[test]
    fn identifier_defined_earlier_is_accepted() {
        let items = vec![
            Item::Statement(Stmt::Assignment(Assignment { name: "x".into(), value: Expr::Number(1.0) })),
            Item::Statement(Stmt::Assignment(Assignment {
                name: "y".into(),
                value: Expr::Binary {
                    left: Box::new(Expr::Identifier("x".into())),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::Number(1.0)),
                },
            })),
        ];
        assert_eq!(analyze(&items), Ok(()));
    }

    #[test]
    fn entity_with_known_types_is_accepted() {
        let items = vec![Item::Entity(EntityDecl {
            name: "Player".into(),
            fields: vec![
                FieldDecl { name: "transform".into(), type_name: None, default: None },
                FieldDecl {
                    name: "speed".into(),
                    type_name: Some("Float".into()),
                    default: Some(Expr::Number(250.0)),
                },
            ],
        })];
        assert_eq!(analyze(&items), Ok(()));
    }

    #[test]
    fn duplicate_field_name_is_rejected() {
        let items = vec![Item::Entity(EntityDecl {
            name: "Player".into(),
            fields: vec![
                FieldDecl { name: "speed".into(), type_name: Some("Float".into()), default: None },
                FieldDecl { name: "speed".into(), type_name: Some("Int".into()), default: None },
            ],
        })];
        assert_eq!(
            analyze(&items),
            Err(vec![SemanticError::DuplicateField {
                entity: "Player".into(),
                field: "speed".into(),
            }])
        );
    }

    #[test]
    fn duplicate_entity_name_is_rejected() {
        let items = vec![
            Item::Entity(EntityDecl { name: "Player".into(), fields: vec![] }),
            Item::Entity(EntityDecl { name: "Player".into(), fields: vec![] }),
        ];
        assert_eq!(
            analyze(&items),
            Err(vec![SemanticError::DuplicateEntity { name: "Player".into() }])
        );
    }

    #[test]
    fn unknown_type_name_is_rejected() {
        let items = vec![Item::Entity(EntityDecl {
            name: "Player".into(),
            fields: vec![FieldDecl {
                name: "sprite".into(),
                type_name: Some("Texture".into()),
                default: None,
            }],
        })];
        assert_eq!(
            analyze(&items),
            Err(vec![SemanticError::UnknownType {
                entity: "Player".into(),
                field: "sprite".into(),
                type_name: "Texture".into(),
            }])
        );
    }
}
