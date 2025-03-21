use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Format, Prettier, array, dynamic_text, group, if_break, indent, ir::Doc, join, line, softline,
    text,
};

pub fn print_import_declaration<'a>(p: &mut Prettier<'a>, decl: &ImportDeclaration<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!("import"));

    if let Some(phase) = decl.phase {
        parts.push(text!(" "));
        parts.push(dynamic_text!(p, phase.as_str()));
    }

    if decl.import_kind.is_type() {
        parts.push(text!(" type"));
    }

    if let Some(specifiers) = &decl.specifiers {
        let validate_namespace = |ids: &ImportDeclarationSpecifier| {
            matches!(ids, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
        };

        let is_default = specifiers.first().is_some_and(|ids| {
            matches!(ids, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
        });
        let is_namespace = specifiers.first().is_some_and(validate_namespace)
            || specifiers.get(1).is_some_and(validate_namespace);

        parts.push(print_module_specifiers(p, specifiers, is_default, is_namespace));
        parts.push(text!(" from"));
    }

    parts.push(text!(" "));
    parts.push(decl.source.format(p));

    if let Some(with_clause) = &decl.with_clause {
        parts.push(print_import_attributes(p, with_clause));
    }

    if p.options.semi {
        parts.push(text!(";"));
    }

    array!(p, parts)
}

#[expect(clippy::enum_variant_names)]
pub enum ExportDeclarationLike<'a, 'b> {
    ExportAllDeclaration(&'b ExportAllDeclaration<'a>),
    ExportNamedDeclaration(&'b ExportNamedDeclaration<'a>),
    ExportDefaultDeclaration(&'b ExportDefaultDeclaration<'a>),
}

impl<'a> ExportDeclarationLike<'a, '_> {
    fn print_semicolon_after_export_declaration(&self, p: &Prettier<'a>) -> bool {
        if !p.options.semi {
            return false;
        }

        match self {
            ExportDeclarationLike::ExportAllDeclaration(_) => true,
            ExportDeclarationLike::ExportNamedDeclaration(decl) => {
                let Some(declaration) = &decl.declaration else {
                    return true;
                };

                // Prettier's `shouldOmitSemicolon()` function
                !matches!(
                    declaration,
                    Declaration::ClassDeclaration(_)
                        | Declaration::VariableDeclaration(_)
                        | Declaration::FunctionDeclaration(_)
                        | Declaration::TSInterfaceDeclaration(_)
                        | Declaration::TSEnumDeclaration(_)
                        | Declaration::TSModuleDeclaration(_)
                        | Declaration::TSImportEqualsDeclaration(_)
                )
            }
            ExportDeclarationLike::ExportDefaultDeclaration(decl) => {
                matches!(decl.declaration, match_expression!(ExportDefaultDeclarationKind))
            }
            _ => false,
        }
    }
}

pub fn print_export_declaration<'a>(
    p: &mut Prettier<'a>,
    decl: &ExportDeclarationLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    // TODO: Print decorators before export for ExportDefaultDeclaration and ExportNamedDeclaration
    // ```
    // @deco export class X {}
    // ```
    // Print decorators here, then skip them in the Class.decorators

    parts.push(text!("export"));

    if matches!(decl, ExportDeclarationLike::ExportDefaultDeclaration(_)) {
        parts.push(text!(" default "));
    }

    // TODO: Dangling comments

    match decl {
        ExportDeclarationLike::ExportAllDeclaration(decl) => {
            if decl.export_kind.is_type() {
                parts.push(text!(" type"));
            }
            parts.push(text!(" *"));
            if let Some(exported) = &decl.exported {
                parts.push(text!(" as "));
                parts.push(exported.format(p));
            }
        }
        ExportDeclarationLike::ExportNamedDeclaration(decl) => {
            if decl.export_kind.is_type()
                && !decl.declaration.as_ref().is_some_and(oxc_ast::ast::Declaration::is_type)
            {
                parts.push(text!(" type"));
            }
            if let Some(decl) = &decl.declaration {
                parts.push(text!(" "));
                parts.push(decl.format(p));
            } else {
                parts.push(print_module_specifiers(p, &decl.specifiers, false, false));
            }
        }
        ExportDeclarationLike::ExportDefaultDeclaration(decl) => {
            parts.push(match &decl.declaration {
                match_expression!(ExportDefaultDeclarationKind) => {
                    decl.declaration.to_expression().format(p)
                }
                ExportDefaultDeclarationKind::FunctionDeclaration(decl) => decl.format(p),
                ExportDefaultDeclarationKind::ClassDeclaration(decl) => decl.format(p),
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl) => decl.format(p),
            });
        }
    }

    if let Some(source_doc) = match decl {
        ExportDeclarationLike::ExportAllDeclaration(decl) => Some(decl.source.format(p)),
        ExportDeclarationLike::ExportNamedDeclaration(decl) => {
            decl.source.as_ref().map(|s| s.format(p))
        }
        ExportDeclarationLike::ExportDefaultDeclaration(_) => None,
    } {
        parts.push(text!(" from "));
        parts.push(source_doc);
    }

    if let Some(with_clause) = match decl {
        ExportDeclarationLike::ExportAllDeclaration(decl) => decl.with_clause.as_ref(),
        ExportDeclarationLike::ExportNamedDeclaration(decl) => decl.with_clause.as_ref(),
        ExportDeclarationLike::ExportDefaultDeclaration(_) => None,
    } {
        parts.push(print_import_attributes(p, with_clause));
    }

    if decl.print_semicolon_after_export_declaration(p) {
        parts.push(text!(";"));
    }

    array!(p, parts)
}

// ---

fn print_import_attributes<'a>(p: &mut Prettier<'a>, with_clause: &WithClause<'a>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!(" "));
    parts.push(with_clause.attributes_keyword.format(p));
    parts.push(text!(" {"));

    if !with_clause.with_entries.is_empty() {
        if p.options.bracket_spacing {
            parts.push(text!(" "));
        }

        let attributes_doc = with_clause
            .with_entries
            .iter()
            .map(|import_attr| import_attr.format(p))
            .collect::<std::vec::Vec<_>>();
        parts.push(join!(p, text!(", "), attributes_doc));

        if p.options.bracket_spacing {
            parts.push(text!(" "));
        }
    }

    parts.push(text!("}"));

    array!(p, parts)
}

fn print_module_specifiers<'a, T: Format<'a>>(
    p: &mut Prettier<'a>,
    specifiers: &Vec<'a, T>,
    include_default: bool,
    include_namespace: bool,
) -> Doc<'a> {
    if specifiers.is_empty() {
        return text!(" {}");
    }

    let mut parts = Vec::new_in(p.allocator);
    parts.push(text!(" "));

    let mut specifiers_iter: std::collections::VecDeque<_> = specifiers.iter().collect();
    if include_default {
        parts.push(specifiers_iter.pop_front().unwrap().format(p));
        if !specifiers_iter.is_empty() {
            parts.push(text!(", "));
        }
    }

    if include_namespace {
        parts.push(specifiers_iter.pop_front().unwrap().format(p));
        if !specifiers_iter.is_empty() {
            parts.push(text!(", "));
        }
    }

    if !specifiers_iter.is_empty() {
        let can_break = specifiers.len() > 1;
        let specifier_docs =
            specifiers_iter.iter().map(|s| s.format(p)).collect::<std::vec::Vec<_>>();

        if can_break {
            parts.push(group!(
                p,
                [
                    text!("{"),
                    indent!(
                        p,
                        [
                            if p.options.bracket_spacing { line!() } else { softline!() },
                            join!(p, array!(p, [text!(","), line!()]), specifier_docs)
                        ]
                    ),
                    if_break!(
                        p,
                        text!(if p.options.trailing_comma.should_print_es5() { "," } else { "" })
                    ),
                    if p.options.bracket_spacing { line!() } else { softline!() },
                    text!("}"),
                ]
            ));
        } else {
            parts.push(text!("{"));
            if p.options.bracket_spacing {
                parts.push(text!(" "));
            }
            parts.extend(specifier_docs);
            if p.options.bracket_spacing {
                parts.push(text!(" "));
            }
            parts.push(text!("}"));
        }
    }

    array!(p, parts)
}
