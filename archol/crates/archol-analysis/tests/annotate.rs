#![allow(clippy::collapsible_match, clippy::single_match)]

use archol_analysis::{analyze_application, FileInput};
use archol_core::{Expr, Member, Stmt};

fn files() -> Vec<FileInput> {
    vec![
        (
            "ws/applications/example.com/app/demo/demo-application.archol",
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/billing\" as Billing\n  }\n}",
        ),
        (
            "ws/services/example.com/services/billing/billing-service.archol",
            "service \"example.com/services/billing\" {\n  title: { pt-BR: \"Billing\" }\n  locales: [ \"pt-BR\" ]\n}",
        ),
        (
            "ws/services/example.com/services/billing/billing-shapes.archol",
            "shape InvoiceStatus {\n  base: classification\n  value Previous { title: { pt-BR: \"P\" } }\n  value Overdue { title: { pt-BR: \"O\" } }\n  value Paid { title: { pt-BR: \"Paid\" } }\n}",
        ),
        (
            "ws/services/example.com/services/billing/documents/billing-invoice-document.archol",
            "document Invoice {\n  title: { pt-BR: \"Nota\" }\n  fields {\n    status: InvoiceStatus { title: { pt-BR: \"Situação\" } }\n  }\n}",
        ),
        (
            "ws/services/example.com/services/billing/processes/billing-dunning-process.archol",
            "process Dunning(invoice: Invoice) {\n  title: { pt-BR: \"Cobrança\" }\n  if (invoice.status == Overdue) {\n    return Done\n  }\n}",
        ),
    ]
    .into_iter()
    .map(|(path, source)| FileInput {
        path: path.to_string(),
        source: source.to_string(),
    })
    .collect()
}

fn analysis() -> archol_analysis::Analysis {
    analyze_application(files(), Some("ws/services"), "example.com/app/demo", false)
        .expect("analysis")
}

#[test]
fn enum_value_usage_becomes_typed_reference_with_lock_id() {
    let analysis = analysis();
    let semantic = analysis.semantic.as_ref().expect("semantic");
    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("billing-dunning-process.archol"))
        .expect("process file");
    let artifact = &process.ast.items[0];

    let mut reference = None;
    walk(&artifact.body, &mut |member| {
        if let Member::Statement {
            statement: Stmt::IfStatement { condition, .. },
            ..
        } = member
        {
            if let Expr::Binary { right, .. } = condition {
                if let Expr::Reference {
                    ref_kind,
                    name,
                    target,
                    ..
                } = right.as_ref()
                {
                    reference = Some((ref_kind.clone(), name.clone(), target.clone()));
                }
            }
        }
    });
    let (ref_kind, name, target) = reference.expect("resolved enum reference");
    assert_eq!(ref_kind, "enumValue");
    assert_eq!(name, "Overdue");
    assert!(semantic.declarations.contains_key(&target));

    let value_path = "example.com/services/billing#shape/InvoiceStatus/value/Overdue";
    let entry = semantic.lock.entry_for(value_path).expect("lock entry");
    assert_eq!(entry.id, target);
    assert!(semantic
        .usages
        .iter()
        .any(|usage| usage.id == target && usage.kind == "enumValue"));
}

#[test]
fn field_access_gets_field_ref_and_declaration_id() {
    let analysis = analysis();
    let document = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("billing-invoice-document.archol"))
        .expect("document");
    let fields = document.ast.items[0]
        .body
        .iter()
        .find_map(|member| match member {
            Member::Block { name, body, .. } if name == "fields" => Some(body),
            _ => None,
        })
        .expect("fields");
    let Member::PropertyAssignment { id: field_id, .. } = fields
        .iter()
        .find(|member| member.name() == Some("status"))
        .expect("status field")
    else {
        panic!("property");
    };
    let field_id = field_id.clone().expect("field id");

    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("billing-dunning-process.archol"))
        .expect("process");
    let mut member_access_field = None;
    walk(&process.ast.items[0].body, &mut |member| {
        if let Member::Statement {
            statement: Stmt::IfStatement { condition, .. },
            ..
        } = member
        {
            if let Expr::Binary { left, .. } = condition {
                if let Expr::MemberAccess { field_ref, .. } = left.as_ref() {
                    member_access_field = field_ref.clone();
                }
            }
        }
    });
    assert_eq!(member_access_field, Some(field_id));
}

#[test]
fn artifact_reference_is_resolved() {
    let mut input = files();
    input[4].source = "process Dunning(invoice: Invoice) {\n  document: Invoice\n}".to_string();
    let analysis =
        analyze_application(input, Some("ws/services"), "example.com/app/demo", false).unwrap();
    let process = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("billing-dunning-process.archol"))
        .unwrap();
    let property = process.ast.items[0]
        .body
        .iter()
        .find(|member| member.name() == Some("document"))
        .expect("document property");
    let Member::PropertyAssignment { value, .. } = property else {
        panic!("property");
    };
    match value {
        Expr::Reference {
            ref_kind, target, ..
        } => {
            assert_eq!(ref_kind, "document");
            assert!(analysis
                .semantic
                .as_ref()
                .unwrap()
                .declarations
                .contains_key(target));
        }
        other => panic!("expected reference, got {other:?}"),
    }
}

#[test]
fn lock_entries_are_created_and_pruned() {
    let analysis = analysis();
    let semantic = analysis.semantic.as_ref().unwrap();
    assert!(semantic
        .lock
        .entry_for("example.com/services/billing#service")
        .is_some());
    assert!(semantic
        .lock
        .entry_for("example.com/services/billing#document/Invoice/field/status")
        .is_some());
}

fn walk(members: &[Member], visit: &mut impl FnMut(&Member)) {
    for member in members {
        visit(member);
        match member {
            Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
                walk(body, visit)
            }
            Member::Statement { statement, .. } => match statement {
                Stmt::IfStatement {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    walk(then_branch, visit);
                    if let Some(else_branch) = else_branch {
                        walk(else_branch, visit);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[test]
fn application_service_alias_gets_service_id() {
    let analysis = analysis();
    let app = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path.ends_with("demo-application.archol"))
        .expect("application file");
    let mut entry = None;
    walk(&app.ast.items[0].body, &mut |member| {
        if let Member::ServiceEntry { target, .. } = member {
            entry = target.clone();
        }
    });
    let target = entry.expect("service id");
    let semantic = analysis.semantic.as_ref().unwrap();
    let declaration = semantic.declarations.get(&target).expect("declaration");
    assert!(declaration.entry.path.ends_with("#service"));
    assert!(semantic
        .usages
        .iter()
        .any(|usage| usage.id == target && usage.kind == "serviceRef"));
}
