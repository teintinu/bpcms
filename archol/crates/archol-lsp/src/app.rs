//! Application selection and closure checks for the LSP.

use archol_analysis::{Analysis, ApplicationInfo};

use crate::util::{same_path, uri_to_path};

/// Normalizes an application selector from `initializationOptions` or
/// `archol/setApplication`.
///
/// File URIs become filesystem paths; application URIs (for example
/// `example.com/app/demo`) are kept unchanged.
pub fn selector_text(selector: &str) -> String {
    let selector = selector.trim();
    if selector.starts_with("file://") {
        if let Some(path) = uri_to_path(selector) {
            return path.to_string_lossy().replace('\\', "/");
        }
    }
    selector.to_string()
}

/// Finds an application by URI, by application file path or by a unique suffix
/// of either one.
pub fn find_application<'a>(
    apps: &'a [ApplicationInfo],
    selector: &str,
) -> Option<&'a ApplicationInfo> {
    let selector = selector_text(selector);
    if selector.is_empty() {
        return None;
    }
    apps.iter()
        .find(|app| app.uri == selector || app.path == selector)
        .or_else(|| {
            apps.iter()
                .find(|app| same_path(&app.path, &selector) || app.uri.ends_with(&selector))
        })
}

/// Last segment of an application URI (`example.com/app/demo` -> `demo`).
pub fn name_of(uri: &str) -> &str {
    uri.rsplit('/').next().unwrap_or(uri)
}

/// True only when a closure exists and owns the file.
pub fn in_application(analysis: &Analysis, path: &str) -> bool {
    analysis
        .closure
        .as_ref()
        .map(|closure| {
            closure.contains_file(path) || closure.files.iter().any(|file| same_path(file, path))
        })
        .unwrap_or(false)
}
