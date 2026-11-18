//! Unit tests for application-scoped analysis and identifier navigation.

use std::path::Path;

use archol_analysis::application::discover;
use archol_analysis::{analyze_application, parse_workspace, Analysis, FileInput};

use crate::app;
use crate::util::{path_to_uri, uri_to_path};

use super::semantic;

const APP_URI: &str = "example.com/app/demo";
const SERVICES_ROOT: &str = "ws/services";
const APP_PATH: &str = "ws/applications/example.com/app/demo/demo-application.archol";
const PROCESS_PATH: &str =
    "ws/services/example.com/services/billing/processes/billing-dunning-process.archol";
const OUTSIDE_PATH: &str =
    "ws/services/example.com/services/catalog/documents/catalog-product-document.archol";

fn fixtures() -> Vec<FileInput> {
    let files = [
        (
            APP_PATH,
            "application \"example.com/app/demo\" {\n  services {\n    \"example.com/services/billing\" as Billing\n  }\n}",
        ),
        (
            "ws/services/example.com/services/billing/billing-service.archol",
            "service \"example.com/services/billing\" { title: { pt-BR: \"Billing\" } }",
        ),
        (
            "ws/services/example.com/services/billing/billing-shapes.archol",
            "shape InvoiceStatus {\n  base: classification\n  value Previous { title: { pt-BR: \"P\" } }\n  value Overdue { title: { pt-BR: \"O\" } }\n  value Paid { title: { pt-BR: \"Paid\" } }\n}",
        ),
        (
            "ws/services/example.com/services/billing/documents/billing-invoice-document.archol",
            "document Invoice {\n  title: { pt-BR: \"Nota\" }\n  fields {\n    number: string { title: { pt-BR: \"Número\" } }\n    status: InvoiceStatus { title: { pt-BR: \"Situação\" } }\n  }\n}",
        ),
        (
            PROCESS_PATH,
            "process Dunning(invoice: Invoice) {\n  fanOut: Invoice where status == Overdue\n  if (invoice.status == Overdue) {\n    return Done\n  }\n  if (invoice.status == Overdue) {\n    return Done\n  }\n}",
        ),
        (
            "ws/services/example.com/services/catalog/catalog-service.archol",
            "service \"example.com/services/catalog\" { title: { pt-BR: \"Catalog\" } }",
        ),
        (
            OUTSIDE_PATH,
            "document Product {\n  title: { pt-BR: \"Produto\" }\n}",
        ),
    ];
    files
        .into_iter()
        .map(|(path, source)| FileInput {
            path: path.to_string(),
            source: source.to_string(),
        })
        .collect()
}

fn analysis() -> Analysis {
    analyze_application(fixtures(), Some(SERVICES_ROOT), APP_URI, false)
        .expect("application analysis")
}

/// Ids come from the semantic index; the lock is fresh in memory because
/// `write_lock` is false and no application lock file exists under the fixture path.
fn overdue_id(analysis: &Analysis) -> String {
    analysis
        .semantic
        .as_ref()
        .expect("semantic index")
        .declarations
        .iter()
        .find(|(_, declaration)| declaration.kind == "value" && declaration.name == "Overdue")
        .map(|(id, _)| id.clone())
        .expect("Overdue declaration id")
}

#[test]
fn selects_application_by_uri_and_by_file_uri() {
    let workspace = parse_workspace(fixtures(), Some(SERVICES_ROOT));
    let applications = discover(&workspace);
    assert_eq!(applications.len(), 1);

    let selected = app::find_application(&applications, APP_URI).expect("application by URI");
    assert_eq!(selected.uri, APP_URI);
    assert_eq!(app::name_of(&selected.uri), "demo");

    let file_uri = path_to_uri(Path::new(&selected.path));
    let from_file_uri =
        app::find_application(&applications, &file_uri).expect("application by file URI");
    assert_eq!(from_file_uri.uri, APP_URI);
    assert_eq!(
        uri_to_path(&file_uri).expect("file URI path"),
        Path::new(&format!("/{APP_PATH}"))
    );
}

#[test]
fn checks_closure_membership() {
    let analysis = analysis();
    assert!(app::in_application(&analysis, PROCESS_PATH));
    assert!(!app::in_application(&analysis, OUTSIDE_PATH));
    let closure = analysis.closure.as_ref().expect("closure");
    assert_eq!(closure.application.uri, APP_URI);
    assert!(closure.services.contains("example.com/services/billing"));
    assert!(!closure.services.contains("example.com/services/catalog"));
}

#[test]
fn finds_declaration_by_id() {
    let analysis = analysis();
    let id = overdue_id(&analysis);
    let target = semantic::declaration(&analysis, &id).expect("declaration");
    assert!(target.path.ends_with("billing-shapes.archol"));
    let file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == target.path)
        .expect("shape file");
    assert!(
        file.source[target.span.start..target.span.end].starts_with("value Overdue"),
        "{}",
        &file.source[target.span.start..target.span.end]
    );
    assert!(semantic::declaration(&analysis, "missing-id").is_none());
}

#[test]
fn lists_usages_sorted_by_file_and_offset() {
    let analysis = analysis();
    let id = overdue_id(&analysis);
    let locations = semantic::usages(&analysis, &id);
    assert!(locations.len() >= 2, "{locations:?}");
    assert!(locations.iter().any(|location| {
        location.path.ends_with("billing-dunning-process.archol") && location.kind == "enumValue"
    }));
    for pair in locations.windows(2) {
        assert!(
            (&pair[0].path, pair[0].span.start) <= (&pair[1].path, pair[1].span.start),
            "usages must be sorted: {locations:?}"
        );
    }
    assert!(semantic::usages(&analysis, "missing-id").is_empty());
}

#[test]
fn resolves_fanout_enum_value() {
    let analysis = analysis();
    let file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == PROCESS_PATH)
        .expect("process");
    let offset = file.source.find("status == Overdue").expect("fanOut") + "status == ".len();
    let target = crate::resolve::definition(&analysis, PROCESS_PATH, offset).expect("definition");
    assert!(
        target.path.ends_with("billing-shapes.archol"),
        "{}",
        target.path
    );
    let target_file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == target.path)
        .expect("target file");
    assert!(target_file.source[target.span.start..target.span.end].starts_with("value Overdue"));
}
