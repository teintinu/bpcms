//! Resolved application AST (`ArcholApplication`, schemaVersion 1).
//!
//! One closed tree with every definition in the `artifact` map and every
//! reference pointing to a lock id. `reintroduce` is merged before emission.
//! The output is JSON (`serde_json::Value`) because the contract is consumed as
//! JSON by the builder and the CLI.

use std::collections::{BTreeMap, BTreeSet};

use archol_core::{Artifact, ArtifactKind, Expr, HierarchyChain, Member, Pending, Span, Stmt};
use serde_json::{json, Map, Value};

use crate::annotate::DeclInfo;
use crate::application::Closure;
use crate::index::Symbols;
use crate::workspace::{ParsedFile, Workspace};
use crate::Analysis;

/// Builds the application AST for an analysis produced by
/// [`crate::analyze_application`]. `workspace_root` is used to make every path
/// relative to the workspace.
pub fn build(analysis: &Analysis, workspace_root: &str) -> Result<Value, String> {
    let closure = analysis
        .closure
        .as_ref()
        .ok_or_else(|| "application AST requires an application context".to_string())?;
    let semantic = analysis
        .semantic
        .as_ref()
        .ok_or_else(|| "application AST requires semantic annotation".to_string())?;
    let app_file = analysis
        .workspace
        .files
        .iter()
        .find(|file| file.path == closure.application.path)
        .ok_or_else(|| format!("application file `{}` not found", closure.application.path))?;
    let app = app_file
        .ast
        .items
        .iter()
        .find(|artifact| artifact.kind == ArtifactKind::Application)
        .ok_or_else(|| "application declaration not found".to_string())?;
    let mut builder = Builder::new(
        &analysis.workspace,
        closure,
        semantic,
        &analysis.symbols,
        workspace_root,
        app_file,
        app,
    );
    Ok(builder.build())
}

// ---------------------------------------------------------------------------
// Indexes
// ---------------------------------------------------------------------------

struct ServiceMeta<'a> {
    uri: String,
    id: String,
    file: &'a ParsedFile,
    artifact: &'a Artifact,
}

struct ArtifactMeta<'a> {
    service: String,
    name: String,
    id: String,
    file: &'a ParsedFile,
    artifact: &'a Artifact,
}

#[derive(Clone)]
struct OriginMember {
    member: Member,
    file: String,
}

#[derive(Clone, Default)]
struct Merged {
    members: Option<Vec<OriginMember>>,
    contributions: Vec<OriginMember>,
    inserts: Vec<(String, String, String)>,
    transition_origins: BTreeMap<(String, String), String>,
    action_origins: BTreeMap<String, String>,
}

struct Builder<'a> {
    workspace: &'a Workspace,
    closure: &'a Closure,
    symbols: &'a Symbols,
    root: String,
    workspace_root: String,
    app_file: &'a ParsedFile,
    app: &'a Artifact,
    services: BTreeMap<String, ServiceMeta<'a>>,
    artifacts: BTreeMap<(String, String, String), ArtifactMeta<'a>>,
    app_aliases: BTreeMap<String, String>,
    decls: BTreeMap<String, &'a DeclInfo>,
    merged: BTreeMap<String, Merged>,
    effective: BTreeMap<String, Vec<String>>,
    permission_roles: BTreeMap<String, Vec<String>>,
    role_organogram: BTreeMap<String, String>,
    parameter_values: BTreeMap<String, Value>,
}

impl<'a> Builder<'a> {
    fn new(
        workspace: &'a Workspace,
        closure: &'a Closure,
        semantic: &'a crate::annotate::SemanticIndex,
        symbols: &'a Symbols,
        root: &str,
        app_file: &'a ParsedFile,
        app: &'a Artifact,
    ) -> Self {
        let mut services = BTreeMap::new();
        let mut artifacts = BTreeMap::new();
        let mut decls = BTreeMap::new();
        for file in &workspace.files {
            if !closure.contains_file(&file.path) {
                continue;
            }
            let Some(service) = file.service.clone() else {
                continue;
            };
            for artifact in &file.ast.items {
                let Some(id) = artifact.id.clone() else {
                    continue;
                };
                if artifact.kind == ArtifactKind::Service {
                    services.insert(
                        artifact.name.clone(),
                        ServiceMeta {
                            uri: artifact.name.clone(),
                            id,
                            file,
                            artifact,
                        },
                    );
                } else if artifact.kind != ArtifactKind::Application {
                    artifacts.insert(
                        (
                            service.clone(),
                            artifact.kind.name().to_ascii_lowercase(),
                            artifact.name.clone(),
                        ),
                        ArtifactMeta {
                            service: service.clone(),
                            name: artifact.name.clone(),
                            id,
                            file,
                            artifact,
                        },
                    );
                }
            }
        }
        for (id, decl) in &semantic.declarations {
            decls.insert(id.clone(), decl);
        }
        let workspace_root = std::fs::canonicalize(root)
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|_| root.to_string());
        let mut builder = Self {
            workspace,
            closure,
            symbols,
            root: root.to_string(),
            workspace_root,
            app_file,
            app,
            services,
            artifacts,
            app_aliases: BTreeMap::new(),
            decls,
            merged: BTreeMap::new(),
            effective: BTreeMap::new(),
            permission_roles: BTreeMap::new(),
            role_organogram: BTreeMap::new(),
            parameter_values: BTreeMap::new(),
        };
        builder.index_app_aliases();
        builder.build_reintroductions();
        builder.finalize_merges();
        builder.build_role_index();
        builder
    }

    fn index_app_aliases(&mut self) {
        for member in &self.app.body {
            let Member::Block { name, body, .. } = member else {
                continue;
            };
            if name != "services" {
                continue;
            }
            for entry in body {
                if let Member::ServiceEntry { uri, alias, .. } = entry {
                    if !alias.is_empty() {
                        self.app_aliases.insert(alias.clone(), uri.clone());
                    }
                }
            }
        }
    }

    fn build(&mut self) -> Value {
        let (application, main) = self.application_and_main();
        for entry in main
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let (Some(id), Some(value)) = (
                entry
                    .get("parameter")
                    .and_then(|value| value.get("id"))
                    .and_then(Value::as_str),
                entry.get("value"),
            ) {
                self.parameter_values.insert(id.to_string(), value.clone());
            }
        }
        let mut artifact: BTreeMap<String, Value> = BTreeMap::new();
        for service in self.services.values() {
            artifact.insert(service.id.clone(), self.service_def(service));
        }
        for meta in self.artifacts.values() {
            if let Some(def) = self.artifact_def(meta) {
                artifact.insert(meta.id.clone(), def);
            }
        }
        for id in self.decls.keys() {
            if let Some(def) = self.permission_def(id) {
                artifact.insert(id.clone(), def);
            }
        }
        let mut by_kind: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut by_service: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for def in artifact.values() {
            let Some(kind) = def.get("kind").and_then(Value::as_str) else {
                continue;
            };
            let Some(id) = def.get("id").and_then(Value::as_str) else {
                continue;
            };
            by_kind
                .entry(kind.to_string())
                .or_default()
                .push(id.to_string());
            if let Some(service) = def.get("serviceId").and_then(Value::as_str) {
                by_service
                    .entry(service.to_string())
                    .or_default()
                    .push(id.to_string());
            }
        }
        for ids in by_kind.values_mut() {
            ids.sort();
        }
        for ids in by_service.values_mut() {
            ids.sort();
        }
        json!({
            "kind": "ArcholApplication",
            "schemaVersion": 1,
            "uri": self.closure.application.uri,
            "path": self.rel(&self.closure.application.path),
            "workspaceRoot": self.workspace_root,
            "artifact": artifact,
            "application": application,
            "main": main,
            "files": self.files(),
            "pending": self.pending(),
            "index": {
                "byKind": by_kind,
                "byService": by_service,
            },
        })
    }

    // --- paths / locations ------------------------------------------------

    fn service_id(&self, uri: &str) -> String {
        self.services
            .get(uri)
            .map(|service| service.id.clone())
            .unwrap_or_else(|| uri.to_string())
    }

    fn rel(&self, path: &str) -> String {
        let root = self.root.trim_end_matches('/');
        if root.is_empty() || root == "." {
            return path.to_string();
        }
        path.strip_prefix(&format!("{root}/"))
            .unwrap_or(path)
            .to_string()
    }

    fn loc(&self, file: &ParsedFile, span: Span) -> Value {
        let (line, column) = file.lines.line_col(span.start);
        json!({
            "file": self.rel(&file.path),
            "start": span.start,
            "end": span.end,
            "line": line + 1,
            "column": column + 1,
        })
    }

    fn loc_in(&self, path: &str, span: Span) -> Value {
        let file = self.workspace.files.iter().find(|file| file.path == path);
        match file {
            Some(file) => self.loc(file, span),
            None => json!({ "file": self.rel(path), "start": span.start, "end": span.end }),
        }
    }

    fn expr_json(&self, expr: &Expr, file: &ParsedFile) -> Value {
        self.expr_json_in(&file.path, expr)
    }

    fn expr_json_in(&self, path: &str, expr: &Expr) -> Value {
        let mut value = serde_json::to_value(expr).unwrap_or(Value::Null);
        if let Some(object) = value.as_object_mut() {
            if object.contains_key("loc") {
                object.insert("loc".to_string(), self.loc_in(path, expr.span()));
            }
        }
        value
    }

    fn reference(&self, kind: &str, id: &str, file: &ParsedFile, span: Span) -> Value {
        if id.is_empty() {
            return Value::Null;
        }
        json!({ "kind": kind, "id": id, "loc": self.loc(file, span) })
    }

    fn reference_in(&self, kind: &str, id: &str, path: &str, span: Span) -> Value {
        json!({ "kind": kind, "id": id, "loc": self.loc_in(path, span) })
    }

    fn body_json(&self, body: &[Member]) -> Value {
        serde_json::to_value(body).unwrap_or(Value::Array(Vec::new()))
    }

    // --- artifact definitions --------------------------------------------

    fn artifact_def(&self, meta: &ArtifactMeta<'a>) -> Option<Value> {
        match meta.artifact.kind {
            ArtifactKind::Role => Some(self.role_def(meta)),
            ArtifactKind::Organogram => Some(self.organogram_def(meta)),
            ArtifactKind::Document => Some(self.document_def(meta)),
            ArtifactKind::Shape => Some(self.shape_def(meta)),
            ArtifactKind::View => Some(self.view_def(meta)),
            ArtifactKind::Form => Some(self.form_def(meta)),
            ArtifactKind::Process => Some(self.process_def(meta)),
            ArtifactKind::Message => Some(self.message_def(meta)),
            ArtifactKind::Policy => Some(self.generic_def(meta, "PolicyDef")),
            ArtifactKind::Decision => Some(self.generic_def(meta, "DecisionDef")),
            ArtifactKind::Integration => Some(self.integration_def(meta)),
            ArtifactKind::Parameter => Some(self.parameter_def(meta)),
            ArtifactKind::Secret => Some(self.secret_def(meta)),
            ArtifactKind::Icon => Some(self.icon_def(meta)),
            ArtifactKind::Manual => Some(self.manual_def(meta)),
            ArtifactKind::Error => Some(self.error_def(meta)),
            _ => None,
        }
    }

    fn base_def(&self, meta: &ArtifactMeta<'a>, kind: &str) -> Map<String, Value> {
        let mut def = Map::new();
        def.insert("kind".to_string(), json!(kind));
        def.insert("id".to_string(), json!(meta.id));
        def.insert(
            "serviceId".to_string(),
            json!(self.service_id(&meta.service)),
        );
        def.insert("name".to_string(), json!(meta.name));
        def.insert("loc".to_string(), self.loc(meta.file, meta.artifact.span));
        def
    }

    fn members_of(&self, meta: &ArtifactMeta<'a>) -> Vec<OriginMember> {
        if let Some(merged) = self.merged.get(&meta.id) {
            if let Some(members) = &merged.members {
                return members.clone();
            }
        }
        meta.artifact
            .body
            .iter()
            .cloned()
            .map(|member| OriginMember {
                member,
                file: meta.file.path.clone(),
            })
            .collect()
    }

    fn body_of(&self, meta: &ArtifactMeta<'a>) -> Vec<Member> {
        self.members_of(meta)
            .into_iter()
            .map(|origin| origin.member)
            .collect()
    }

    fn service_def(&self, service: &ServiceMeta<'a>) -> Value {
        let mut def = Map::new();
        def.insert("kind".to_string(), json!("ServiceDef"));
        def.insert("id".to_string(), json!(service.id));
        def.insert("name".to_string(), json!(service.artifact.name));
        def.insert("url".to_string(), json!(service.uri));
        def.insert("path".to_string(), json!(self.rel(&service.file.path)));
        def.insert(
            "loc".to_string(),
            self.loc(service.file, service.artifact.span),
        );
        if let Some(title) = property(&service.artifact.body, "title") {
            def.insert("title".to_string(), self.expr_json(title, service.file));
        }
        def.insert(
            "locales".to_string(),
            json!(crate::i18n::locales_of(service.artifact)),
        );
        let mut entry_point = false;
        let mut aliases = Vec::new();
        for member in &self.app.body {
            let Member::Block { name, body, .. } = member else {
                continue;
            };
            if name != "services" {
                continue;
            }
            for entry in body {
                if let Member::ServiceEntry {
                    uri, alias, span, ..
                } = entry
                {
                    if uri == &service.uri {
                        entry_point = true;
                        aliases.push(json!({
                            "kind": "ServiceAlias",
                            "name": if alias.is_empty() { service.artifact.name.clone() } else { alias.clone() },
                            "use": "entryPoint",
                            "by": Value::Null,
                            "path": self.rel(&self.app_file.path),
                            "loc": self.loc(self.app_file, *span),
                        }));
                    }
                }
            }
        }
        for file in &self.workspace.files {
            if !self.closure.contains_file(&file.path)
                || file.service.as_deref() == Some(&service.uri)
            {
                continue;
            }
            for artifact in &file.ast.items {
                if artifact.kind != ArtifactKind::Service {
                    continue;
                }
                for member in &artifact.body {
                    let Member::Block { name, body, .. } = member else {
                        continue;
                    };
                    if name != "imports" {
                        continue;
                    }
                    for entry in body {
                        if let Member::ImportDeclaration {
                            uri, alias, span, ..
                        } = entry
                        {
                            if uri == &service.uri {
                                aliases.push(json!({
                                    "kind": "ServiceAlias",
                                    "name": if alias.is_empty() { service.artifact.name.clone() } else { alias.clone() },
                                    "use": "import",
                                    "by": artifact.id,
                                    "path": self.rel(&file.path),
                                    "loc": self.loc(file, *span),
                                }));
                            }
                        }
                    }
                }
            }
        }
        aliases.sort_by(|left, right| {
            left.get("path")
                .and_then(Value::as_str)
                .cmp(&right.get("path").and_then(Value::as_str))
                .then_with(|| {
                    left.get("name")
                        .and_then(Value::as_str)
                        .cmp(&right.get("name").and_then(Value::as_str))
                })
        });
        def.insert("entryPoint".to_string(), json!(entry_point));
        def.insert("aliases".to_string(), json!(aliases));
        def.insert("imports".to_string(), json!(self.service_imports(service)));
        def.insert("body".to_string(), self.body_json(&service.artifact.body));
        Value::Object(def)
    }

    fn service_imports(&self, service: &ServiceMeta<'a>) -> Vec<Value> {
        let mut imports = Vec::new();
        for member in &service.artifact.body {
            let Member::Block { name, body, .. } = member else {
                continue;
            };
            if name != "imports" {
                continue;
            }
            for entry in body {
                if let Member::ImportDeclaration {
                    uri, span, target, ..
                } = entry
                {
                    let id = target
                        .clone()
                        .or_else(|| self.services.get(uri).map(|service| service.id.clone()));
                    if let Some(id) = id {
                        imports.push(self.reference_in(
                            "ServiceRef",
                            &id,
                            &service.file.path,
                            *span,
                        ));
                    }
                }
            }
        }
        imports
    }

    fn role_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "RoleDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert(
            "organogram".to_string(),
            self.role_organogram
                .get(&meta.id)
                .and_then(|id| self.artifacts.values().find(|artifact| &artifact.id == id))
                .map(|organogram| {
                    self.reference(
                        "OrganogramRef",
                        &organogram.id,
                        meta.file,
                        organogram.artifact.span,
                    )
                })
                .unwrap_or(Value::Null),
        );
        let mut inherits = Vec::new();
        if let Some(value) = property(&body, "inherits") {
            for (expr, span) in expr_items(value) {
                if let Some(id) = self.resolve_expr_id(expr, &meta.service, "Role") {
                    inherits.push(self.reference("RoleRef", &id, meta.file, span));
                }
            }
        }
        def.insert("inherits".to_string(), json!(inherits));
        let permissions = self.effective.get(&meta.id).cloned().unwrap_or_default();
        let mut permission_refs = Vec::new();
        for id in permissions {
            let span = meta.artifact.span;
            permission_refs.push(self.reference("PermissionRef", &id, meta.file, span));
        }
        def.insert("permissions".to_string(), json!(permission_refs));
        def.insert(
            "scopes".to_string(),
            json!(property_string_list(&body, "scopes")),
        );
        def.insert("capabilities".to_string(), json!(capabilities_of(&body)));
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn permission_def(&self, id: &str) -> Option<Value> {
        let decl = self.decls.get(id)?;
        if decl.kind != "permission" {
            return None;
        }
        let service = decl.service.clone()?;
        let (namespace, action) = split_permission(&decl.name);
        let loc = self.loc_in(&decl.file, decl.span);
        let mut def = Map::new();
        def.insert("kind".to_string(), json!("PermissionDef"));
        def.insert("id".to_string(), json!(id));
        def.insert("serviceId".to_string(), json!(self.service_id(&service)));
        def.insert("name".to_string(), json!(decl.name));
        def.insert("namespace".to_string(), json!(namespace));
        def.insert("action".to_string(), json!(action));
        let roles = self.permission_roles.get(id).cloned().unwrap_or_default();
        let mut role_refs = Vec::new();
        for role_id in roles {
            if let Some(role) = self.role_meta(&role_id) {
                role_refs.push(self.reference("RoleRef", &role_id, role.file, role.artifact.span));
            }
        }
        def.insert("roles".to_string(), json!(role_refs));
        let mut used_by = Vec::new();
        for organogram in self.artifacts.values().filter(|meta| {
            meta.artifact.kind == ArtifactKind::Organogram && meta.service == service
        }) {
            if members_contain_permission(&organogram.artifact.body, &decl.name) {
                used_by.push(self.reference(
                    "OrganogramRef",
                    &organogram.id,
                    organogram.file,
                    organogram.artifact.span,
                ));
            }
        }
        def.insert("usedBy".to_string(), json!(used_by));
        def.insert("loc".to_string(), loc);
        Some(Value::Object(def))
    }

    fn organogram_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "OrganogramDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert(
            "scopes".to_string(),
            json!(property_string_list(&body, "scopes")),
        );
        let chains = body
            .iter()
            .find_map(|member| match member {
                Member::Hierarchy { chains, .. } => Some(self.hierarchy_json(chains, meta.file)),
                _ => None,
            })
            .unwrap_or_else(|| json!({ "kind": "Hierarchy", "chains": [] }));
        def.insert("hierarchy".to_string(), chains);
        let mut roles: Vec<Value> = Vec::new();
        for role in &meta.artifact.roles {
            let mut manages = Vec::new();
            for pointer in &role.manages {
                let span = self
                    .role_meta(&pointer.role_ref)
                    .map(|meta| meta.artifact.span)
                    .unwrap_or(meta.artifact.span);
                manages.push(self.reference("RoleRef", &pointer.role_ref, meta.file, span));
            }
            let mut inherits = Vec::new();
            for pointer in &role.inherits {
                let span = self
                    .role_meta(&pointer.role_ref)
                    .map(|meta| meta.artifact.span)
                    .unwrap_or(meta.artifact.span);
                inherits.push(self.reference("RoleRef", &pointer.role_ref, meta.file, span));
            }
            let mut permissions = Vec::new();
            for pointer in &role.permissions {
                permissions.push(self.reference(
                    "PermissionRef",
                    &pointer.permission_ref,
                    meta.file,
                    meta.artifact.span,
                ));
            }
            roles.push(json!({
                "kind": "OrganogramRoleDef",
                "role": self.reference("RoleRef", &role.role_ref, meta.file, meta.artifact.span),
                "manages": manages,
                "inherits": inherits,
                "permissions": permissions,
            }));
        }
        def.insert("roles".to_string(), json!(roles));
        def.insert(
            "fourEyes".to_string(),
            self.body_json(&block_body(&body, "fourEyes").unwrap_or_default()),
        );
        def.insert(
            "segregation".to_string(),
            self.body_json(&block_body(&body, "segregation").unwrap_or_default()),
        );
        def.insert(
            "delegations".to_string(),
            self.body_json(&block_body(&body, "delegations").unwrap_or_default()),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn hierarchy_json(&self, chains: &[HierarchyChain], file: &ParsedFile) -> Value {
        let chains: Vec<Value> = chains
            .iter()
            .map(|chain| {
                chain
                    .roles
                    .iter()
                    .filter_map(|role| {
                        role.role_ref
                            .as_ref()
                            .map(|id| self.reference("RoleRef", id, file, role.span))
                    })
                    .collect::<Vec<Value>>()
            })
            .map(Value::Array)
            .collect();
        json!({ "kind": "Hierarchy", "chains": chains })
    }

    fn document_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "DocumentDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        let fields = block_body(&body, "fields")
            .map(|fields| self.fields_json(fields, meta))
            .unwrap_or_default();
        def.insert("fields".to_string(), json!(fields));
        def.insert("lifecycle".to_string(), self.lifecycle_json(meta, &body));
        def.insert(
            "actions".to_string(),
            json!(self.actions_json(meta, &members)),
        );
        def.insert(
            "indexes".to_string(),
            self.body_json(&block_body(&body, "indexes").unwrap_or_default()),
        );
        def.insert(
            "relations".to_string(),
            self.body_json(&block_body(&body, "relations").unwrap_or_default()),
        );
        def.insert(
            "events".to_string(),
            self.body_json(&block_body(&body, "events").unwrap_or_default()),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn fields_json(&self, fields: &[Member], meta: &ArtifactMeta<'a>) -> Vec<Value> {
        let mut result = Vec::new();
        for member in fields {
            let Member::PropertyAssignment {
                name, value, span, ..
            } = member
            else {
                continue;
            };
            let mut field = Map::new();
            field.insert("kind".to_string(), json!("FieldDef"));
            field.insert("name".to_string(), json!(name));
            field.insert(
                "type".to_string(),
                self.type_of(value, &meta.service, meta.file, *span),
            );
            let object = match value {
                Expr::Object { members, .. } => Some(members.as_slice()),
                _ => None,
            };
            if let Some(object) = object {
                for key in ["required", "private", "default", "maxLength", "compute"] {
                    if let Some(value) = property(object, key) {
                        match key {
                            "required" | "private" => {
                                field.insert(
                                    key.to_string(),
                                    json!(bool_value(value).unwrap_or(true)),
                                );
                            }
                            "maxLength" => {
                                if let Some(number) = number_value(value) {
                                    field.insert(key.to_string(), json!(number));
                                }
                            }
                            _ => {
                                field.insert(key.to_string(), self.expr_json(value, meta.file));
                            }
                        }
                    } else if has_shorthand(object, key) && matches!(key, "required" | "private") {
                        field.insert(key.to_string(), json!(true));
                    }
                }
            }
            field.insert("loc".to_string(), self.loc(meta.file, *span));
            result.push(Value::Object(field));
        }
        result
    }

    fn lifecycle_json(&self, meta: &ArtifactMeta<'a>, body: &[Member]) -> Value {
        let Some(fields) = block_body(body, "fields") else {
            return Value::Null;
        };
        let Some(status) = fields.iter().find_map(|member| match member {
            Member::PropertyAssignment { name, value, .. } if name == "status" => Some(value),
            _ => None,
        }) else {
            return Value::Null;
        };
        let Some(shape_id) = self.shape_id_of(status, &meta.service) else {
            return Value::Null;
        };
        let Some(shape) = self.shape_meta(&shape_id) else {
            return Value::Null;
        };
        let initial = property(&shape.artifact.body, "initial")
            .and_then(expr_text)
            .map(|name| self.shape_value_ref(&shape_id, &name, shape.file, shape.artifact.span));
        let states: Vec<Value> = self
            .members_of(shape)
            .iter()
            .filter_map(|origin| match &origin.member {
                Member::EnumValueDeclaration { name, span, .. } => {
                    Some(self.shape_value_ref(&shape_id, name, shape.file, *span))
                }
                _ => None,
            })
            .collect();
        json!({
            "kind": "LifecycleDef",
            "shape": self.reference("ShapeRef", &shape_id, meta.file, meta.artifact.span),
            "initial": initial.unwrap_or(Value::Null),
            "creationEvent": Value::Null,
            "states": states,
        })
    }

    fn actions_json(&self, meta: &ArtifactMeta<'a>, members: &[OriginMember]) -> Vec<Value> {
        let mut result = Vec::new();
        for origin in members {
            match &origin.member {
                Member::Block {
                    name, body, span, ..
                } if name == "actions" => {
                    for member in body {
                        if let Member::Block {
                            name, body, span, ..
                        } = member
                        {
                            result.push(self.action_json(
                                meta,
                                name,
                                body,
                                *span,
                                &origin.file,
                                None,
                            ));
                        }
                    }
                }
                Member::Block {
                    name, body, span, ..
                } if name.starts_with("action ") => {
                    let normalized = name.trim_start_matches("action ");
                    result.push(self.action_json(
                        meta,
                        normalized,
                        body,
                        *span,
                        &origin.file,
                        self.merged.get(&meta.id).and_then(|merged| {
                            merged
                                .action_origins
                                .get(action_name(normalized).as_str())
                                .cloned()
                        }),
                    ));
                }
                _ => {}
            }
        }
        result
    }

    fn action_json(
        &self,
        meta: &ArtifactMeta<'a>,
        name: &str,
        body: &[Member],
        span: Span,
        file: &str,
        _origin: Option<String>,
    ) -> Value {
        let action_name = action_name(name);
        let mut action = Map::new();
        action.insert("kind".to_string(), json!("ActionDef"));
        action.insert("name".to_string(), json!(action_name));
        let mut roles = Vec::new();
        if let Some(value) = property(body, "roles") {
            for (expr, span) in expr_items(value) {
                if let Some(id) = self.resolve_expr_id(expr, &meta.service, "Role") {
                    roles.push(self.reference_in("RoleRef", &id, file, span));
                }
            }
        }
        action.insert("roles".to_string(), json!(roles));
        action.insert(
            "to".to_string(),
            self.action_target(meta, body).unwrap_or(Value::Null),
        );
        action.insert(
            "form".to_string(),
            property(body, "form")
                .and_then(|expr| self.resolve_expr_id(expr, &meta.service, "Form"))
                .map(|id| self.reference_in("FormRef", &id, file, span))
                .unwrap_or(Value::Null),
        );
        action.insert(
            "limits".to_string(),
            self.body_json(&block_body(body, "limits").unwrap_or_default()),
        );
        action.insert("loc".to_string(), self.loc_in(file, span));
        action.insert("body".to_string(), self.body_json(body));
        Value::Object(action)
    }

    fn action_target(&self, meta: &ArtifactMeta<'a>, body: &[Member]) -> Option<Value> {
        let value = property(body, "to")?;
        if let Expr::Reference {
            ref_kind,
            target,
            span,
            ..
        } = value
        {
            if ref_kind == "enumValue" {
                if let Some((shape_id, value_name)) = self.shape_value_of_id(target) {
                    return Some(self.shape_value_ref(&shape_id, &value_name, meta.file, *span));
                }
            }
        }
        let text = expr_text(value)?;
        let (field, target_value) = text.split_once('.')?;
        let shape_id = self.field_shape_id(meta, field)?;
        Some(self.shape_value_ref(&shape_id, target_value, meta.file, value.span()))
    }

    fn field_shape_id(&self, meta: &ArtifactMeta<'a>, field: &str) -> Option<String> {
        let fields = block_body(&meta.artifact.body, "fields")?;
        for member in fields {
            if let Member::PropertyAssignment { name, value, .. } = member {
                if name == field {
                    return self.shape_id_of(value, &meta.service);
                }
            }
        }
        None
    }

    fn shape_id_of(&self, value: &Expr, service: &str) -> Option<String> {
        if let Some(id) = self.type_id_of(value, service) {
            if let Some(meta) = self.artifacts.values().find(|meta| meta.id == id) {
                if meta.artifact.kind == ArtifactKind::Shape {
                    return Some(id);
                }
            }
        }
        let ty = match value {
            Expr::Object {
                type_ref: Some(ty), ..
            } => ty.clone(),
            Expr::Identifier { name, .. } => name.clone(),
            Expr::Reference { name, .. } => name.clone(),
            _ => return None,
        };
        self.artifacts
            .values()
            .find(|meta| {
                meta.name == base_type(&ty)
                    && meta.artifact.kind == ArtifactKind::Shape
                    && (meta.service == service || true)
            })
            .map(|meta| meta.id.clone())
    }

    fn shape_meta(&self, id: &str) -> Option<&ArtifactMeta<'a>> {
        self.artifacts
            .values()
            .find(|meta| meta.id == id && meta.artifact.kind == ArtifactKind::Shape)
    }

    fn shape_value_ref(&self, shape_id: &str, value: &str, file: &ParsedFile, span: Span) -> Value {
        json!({
            "kind": "ShapeValueRef",
            "shapeId": shape_id,
            "valuePath": value,
            "loc": self.loc(file, span),
        })
    }

    fn shape_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "ShapeDef");
        def.insert(
            "shapeKind".to_string(),
            json!(property_string(&body, "base").unwrap_or_default()),
        );
        def.insert(
            "base".to_string(),
            property(&body, "base")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        def.insert(
            "unit".to_string(),
            property(&body, "unit")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        let mut values = Vec::new();
        for origin in &members {
            let Member::EnumValueDeclaration {
                name, body, span, ..
            } = &origin.member
            else {
                continue;
            };
            let mut value = Map::new();
            value.insert("kind".to_string(), json!("ShapeValueDef"));
            value.insert("name".to_string(), json!(name));
            if let Some(ordinal) = property(body, "ordinal").and_then(number_value) {
                value.insert("ordinal".to_string(), json!(ordinal));
            }
            if let Some(title) = property(body, "title") {
                value.insert("title".to_string(), self.expr_json_in(&origin.file, title));
            }
            value.insert(
                "icon".to_string(),
                property(body, "icon")
                    .and_then(|expr| {
                        let name = expr_text(expr)?;
                        let id =
                            self.resolve_artifact_id(&origin.file, &name, "Icon", &meta.service)?;
                        let file = self.file_of_id(&id).unwrap_or(meta.file);
                        Some(self.reference("IconRef", &id, file, meta.artifact.span))
                    })
                    .unwrap_or(Value::Null),
            );
            let mut transitions = Vec::new();
            for member in body {
                if let Member::Block {
                    keyword,
                    name: event,
                    body,
                    span,
                    ..
                } = member
                {
                    if keyword.as_deref() != Some("transition") {
                        continue;
                    }
                    let transition_file = self
                        .merged
                        .get(&meta.id)
                        .and_then(|merged| {
                            merged
                                .transition_origins
                                .get(&(name.clone(), event.clone()))
                                .cloned()
                        })
                        .unwrap_or_else(|| origin.file.clone());
                    transitions.push(json!({
                        "kind": "TransitionDef",
                        "name": event,
                        "to": self.shape_value_ref(&meta.id, event, meta.file, *span),
                        "event": property(body, "event")
                            .and_then(expr_text)
                            .map(Value::String)
                            .unwrap_or(Value::Null),
                        "title": property(body, "title")
                            .map(|value| self.expr_json_in(&transition_file, value))
                            .unwrap_or(Value::Null),
                        "payload": block_body(body, "payload")
                            .map(|payload| self.fields_json(payload, meta))
                            .unwrap_or_default(),
                        "loc": self.loc_in(&transition_file, *span),
                    }));
                }
            }
            value.insert("transitions".to_string(), json!(transitions));
            value.insert("loc".to_string(), self.loc_in(&origin.file, *span));
            values.push(Value::Object(value));
        }
        def.insert("values".to_string(), json!(values));
        def.insert(
            "formula".to_string(),
            property(&body, "formula")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        def.insert(
            "conversions".to_string(),
            self.body_json(&block_body(&body, "conversions").unwrap_or_default()),
        );
        def.insert(
            "methods".to_string(),
            self.body_json(&block_body(&body, "methods").unwrap_or_default()),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn view_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "ViewDef");
        def.insert(
            "viewKind".to_string(),
            json!(property_string(&body, "type").unwrap_or_default()),
        );
        let mut binds = Vec::new();
        for key in ["source", "sources", "document", "process"] {
            if let Some(value) = property(&body, key) {
                for (expr, span) in expr_items(value) {
                    if let Some((id, kind)) = self.resolve_expr_artifact(expr, &meta.service) {
                        binds.push(self.reference(ref_kind_name(&kind), &id, meta.file, span));
                    }
                }
            }
        }
        def.insert("binds".to_string(), json!(binds));
        def.insert(
            "commands".to_string(),
            json!(self.view_commands(meta, &body)),
        );
        def.insert(
            "filters".to_string(),
            self.body_json(&block_body(&body, "filters").unwrap_or_default()),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn view_commands(&self, meta: &ArtifactMeta<'a>, body: &[Member]) -> Vec<Value> {
        let Some(actions) = block_body(body, "actions") else {
            return Vec::new();
        };
        let mut commands = Vec::new();
        for member in actions {
            let Member::Statement { statement, .. } = member else {
                continue;
            };
            let Stmt::ExpressionStatement { expression, .. } = statement else {
                continue;
            };
            let Expr::Binary { op, right, .. } = expression else {
                continue;
            };
            if op != "->" {
                continue;
            }
            if let Some(action) = self.action_ref_from_call(right, &meta.service, meta.file) {
                commands.push(action);
            }
        }
        commands
    }

    fn form_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "FormDef");
        def.insert(
            "document".to_string(),
            property(&body, "document")
                .and_then(|expr| {
                    self.resolve_expr_id(expr, &meta.service, "Document")
                        .map(|id| self.reference("DocumentRef", &id, meta.file, expr.span()))
                })
                .unwrap_or(Value::Null),
        );
        let mut roles = Vec::new();
        if let Some(value) = property(&body, "roles") {
            for (expr, span) in expr_items(value) {
                if let Some(id) = self.resolve_expr_id(expr, &meta.service, "Role") {
                    roles.push(self.reference("RoleRef", &id, meta.file, span));
                }
            }
        }
        def.insert("roles".to_string(), json!(roles));
        let mut fields = Vec::new();
        for key in ["fields"] {
            if let Some(field_members) = block_body(&body, key) {
                fields.extend(self.fields_json(field_members, meta));
            }
        }
        def.insert("fields".to_string(), json!(fields));
        let mut actions = Vec::new();
        if let Some(action_members) = block_body(&body, "actions") {
            for member in action_members {
                if let Member::Block {
                    name, body, span, ..
                } = member
                {
                    actions.push(self.form_action_json(meta, name, body, *span));
                }
            }
        }
        def.insert("actions".to_string(), json!(actions));
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn form_action_json(
        &self,
        meta: &ArtifactMeta<'a>,
        name: &str,
        body: &[Member],
        span: Span,
    ) -> Value {
        let mut action = Map::new();
        action.insert("kind".to_string(), json!("FormActionDef"));
        action.insert("name".to_string(), json!(name));
        action.insert(
            "calls".to_string(),
            property(body, "calls")
                .and_then(|expr| self.action_ref_from_call(expr, &meta.service, meta.file))
                .unwrap_or(Value::Null),
        );
        action.insert(
            "stay".to_string(),
            json!(has_shorthand(body, "stay") || property(body, "stay").is_some()),
        );
        action.insert("loc".to_string(), self.loc(meta.file, span));
        Value::Object(action)
    }

    fn action_ref_from_call(&self, expr: &Expr, service: &str, file: &ParsedFile) -> Option<Value> {
        let (callee, span) = match expr {
            Expr::Call { callee, span, .. } => (callee.clone(), *span),
            Expr::MemberAccess { .. } | Expr::Identifier { .. } => (expr_text(expr)?, expr.span()),
            _ => return None,
        };
        let (owner, action) = callee.split_once('.')?;
        let document = self
            .artifacts
            .values()
            .find(|meta| {
                meta.name == owner
                    && meta.service == service
                    && meta.artifact.kind == ArtifactKind::Document
            })
            .or_else(|| {
                self.artifacts
                    .values()
                    .find(|meta| meta.name == owner && meta.artifact.kind == ArtifactKind::Document)
            })?;
        Some(json!({
            "kind": "ActionRef",
            "documentId": document.id,
            "actionPath": action,
            "loc": self.loc(file, span),
        }))
    }

    fn process_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "ProcessDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        let trigger = body.iter().find_map(|member| match member {
            Member::TriggerDeclaration { targets, .. } => targets
                .first()
                .map(|expr| serde_json::to_value(expr).unwrap()),
            _ => None,
        });
        def.insert(
            "start".to_string(),
            json!({
                "kind": "StartSpec",
                "trigger": trigger.unwrap_or(Value::Null),
                "schedule": property(&body, "schedule")
                    .map(|value| serde_json::to_value(value).unwrap())
                    .unwrap_or(Value::Null),
            }),
        );
        let mut roles = Vec::new();
        if let Some(value) = property(&body, "roles") {
            for (expr, span) in expr_items(value) {
                if let Some(id) = self.resolve_expr_id(expr, &meta.service, "Role") {
                    roles.push(self.reference("RoleRef", &id, meta.file, span));
                }
            }
        }
        def.insert("roles".to_string(), json!(roles));
        let steps = block_body(&body, "steps")
            .map(|steps| self.steps_json(steps, meta))
            .unwrap_or_default();
        def.insert("steps".to_string(), json!(steps));
        let nodes: Vec<String> = steps
            .iter()
            .filter_map(|step| step.get("name").and_then(Value::as_str).map(str::to_string))
            .collect();
        def.insert(
            "flow".to_string(),
            json!({ "kind": "FlowDef", "nodes": nodes, "edges": [] }),
        );
        let ends: Vec<Value> = body
            .iter()
            .filter_map(|member| match member {
                Member::Block {
                    keyword,
                    name,
                    span,
                    ..
                } if keyword.as_deref() == Some("end") => Some(json!({
                    "kind": "EndDef",
                    "name": name,
                    "outcome": Value::Null,
                    "loc": self.loc(meta.file, *span),
                })),
                _ => None,
            })
            .collect();
        def.insert("ends".to_string(), json!(ends));
        def.insert("messages".to_string(), json!([]));
        def.insert("decisions".to_string(), json!([]));
        def.insert(
            "sla".to_string(),
            property(&body, "sla")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn steps_json(&self, steps: &[Member], meta: &ArtifactMeta<'a>) -> Vec<Value> {
        let mut result = Vec::new();
        for member in steps {
            let Member::Block {
                name, body, span, ..
            } = member
            else {
                continue;
            };
            let mut step = Map::new();
            step.insert("kind".to_string(), json!("StepDef"));
            step.insert("name".to_string(), json!(name));
            step.insert(
                "lane".to_string(),
                property(body, "lane")
                    .and_then(expr_text)
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            );
            step.insert(
                "action".to_string(),
                property(body, "action")
                    .and_then(|expr| self.resolve_expr_id(expr, &meta.service, "Document"))
                    .map(|id| {
                        json!({
                            "kind": "ActionRef",
                            "documentId": id,
                            "actionPath": property_string(body, "action").unwrap_or_default(),
                            "loc": self.loc(meta.file, *span),
                        })
                    })
                    .unwrap_or(Value::Null),
            );
            let mut roles = Vec::new();
            if let Some(value) = property(body, "roles") {
                for (expr, span) in expr_items(value) {
                    if let Some(id) = self.resolve_expr_id(expr, &meta.service, "Role") {
                        roles.push(self.reference("RoleRef", &id, meta.file, span));
                    }
                }
            }
            step.insert("roles".to_string(), json!(roles));
            step.insert(
                "form".to_string(),
                property(body, "form")
                    .and_then(|expr| self.resolve_expr_id(expr, &meta.service, "Form"))
                    .map(|id| self.reference("FormRef", &id, meta.file, *span))
                    .unwrap_or(Value::Null),
            );
            step.insert(
                "event".to_string(),
                property(body, "event")
                    .and_then(expr_text)
                    .map(Value::String)
                    .unwrap_or(Value::Null),
            );
            step.insert("loc".to_string(), self.loc(meta.file, *span));
            result.push(Value::Object(step));
        }
        result
    }

    fn message_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "MessageDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert(
            "about".to_string(),
            property(&body, "about")
                .and_then(|expr| {
                    self.resolve_expr_id(expr, &meta.service, "Document")
                        .map(|id| self.reference("DocumentRef", &id, meta.file, expr.span()))
                })
                .unwrap_or(Value::Null),
        );
        let payload = block_body(&body, "payload")
            .map(|payload| self.fields_json(payload, meta))
            .unwrap_or_default();
        def.insert("payload".to_string(), json!(payload));
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn parameter_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "ParameterDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert(
            "type".to_string(),
            property(&body, "type")
                .map(|expr| self.type_of(expr, &meta.service, meta.file, expr.span()))
                .unwrap_or(Value::Null),
        );
        def.insert(
            "required".to_string(),
            json!(has_property_or_shorthand(&body, "required")),
        );
        for key in ["default", "min", "max"] {
            if let Some(value) = property(&body, key) {
                def.insert(key.to_string(), serde_json::to_value(value).unwrap());
            }
        }
        if let Some(number) = property(&body, "maxLength").and_then(number_value) {
            def.insert("maxLength".to_string(), json!(number));
        }
        def.insert(
            "value".to_string(),
            self.parameter_values
                .get(&meta.id)
                .cloned()
                .unwrap_or(Value::Null),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn secret_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let members = self.members_of(meta);
        let body: Vec<Member> = members.iter().map(|origin| origin.member.clone()).collect();
        let mut def = self.base_def(meta, "SecretDef");
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert(
            "type".to_string(),
            property(&body, "type")
                .map(|expr| self.type_of(expr, &meta.service, meta.file, expr.span()))
                .unwrap_or(Value::Null),
        );
        def.insert(
            "required".to_string(),
            json!(has_property_or_shorthand(&body, "required")),
        );
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn icon_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let body = self.body_of(meta);
        let mut def = self.base_def(meta, "IconDef");
        let (namespace, _) = meta
            .name
            .rsplit_once('.')
            .map(|(namespace, name)| (namespace.to_string(), name.to_string()))
            .unwrap_or_else(|| (String::new(), meta.name.clone()));
        def.insert("namespace".to_string(), json!(namespace));
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn generic_def(&self, meta: &ArtifactMeta<'a>, kind: &str) -> Value {
        let body = self.body_of(meta);
        let mut def = self.base_def(meta, kind);
        if let Some(title) = property(&body, "title") {
            def.insert("title".to_string(), self.expr_json(title, meta.file));
        }
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn integration_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        let body = self.body_of(meta);
        let mut def = self.base_def(meta, "IntegrationDef");
        def.insert(
            "integrationKind".to_string(),
            property(&body, "kind")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        def.insert(
            "endpoint".to_string(),
            property(&body, "endpoint")
                .and_then(expr_text)
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        def.insert("secrets".to_string(), json!([]));
        def.insert("parameters".to_string(), json!([]));
        def.insert("body".to_string(), self.body_json(&body));
        Value::Object(def)
    }

    fn manual_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        self.generic_def(meta, "ManualDef")
    }

    fn error_def(&self, meta: &ArtifactMeta<'a>) -> Value {
        self.generic_def(meta, "ErrorDef")
    }

    // --- application / main ----------------------------------------------

    fn application_and_main(&self) -> (Value, Value) {
        let mut roles = Vec::new();
        let mut chains: Vec<Value> = Vec::new();
        let mut parameters = Vec::new();
        let mut secrets = Vec::new();
        let mut start = Vec::new();
        let mut statements = Vec::new();
        for member in &self.app.body {
            match member {
                Member::Block { name, body, .. } if name == "roles" => {
                    for entry in body {
                        let Member::Statement { statement, .. } = entry else {
                            continue;
                        };
                        let Stmt::ExpressionStatement { expression, .. } = statement else {
                            continue;
                        };
                        match expression {
                            Expr::Binary {
                                op, left, right, ..
                            } if op == ">" => {
                                let mut chain = Vec::new();
                                flatten_hierarchy(left, &mut chain);
                                flatten_hierarchy(right, &mut chain);
                                let refs: Vec<Value> = chain
                                    .iter()
                                    .filter_map(|expr| {
                                        self.resolve_app_role(expr).map(|id| {
                                            self.reference(
                                                "RoleRef",
                                                &id,
                                                self.app_file,
                                                expr.span(),
                                            )
                                        })
                                    })
                                    .collect();
                                if !refs.is_empty() {
                                    chains.push(Value::Array(refs));
                                }
                            }
                            _ => {
                                if let Some(id) = self.resolve_app_role(expression) {
                                    roles.push(self.reference(
                                        "RoleRef",
                                        &id,
                                        self.app_file,
                                        expression.span(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Member::Block { name, body, .. } if name == "main" => {
                    let mut index = 0;
                    while index < body.len() {
                        let Member::Statement { statement, .. } = &body[index] else {
                            statements.push(serde_json::to_value(&body[index]).unwrap());
                            index += 1;
                            continue;
                        };
                        let Stmt::ExpressionStatement { expression, .. } = statement else {
                            statements.push(serde_json::to_value(&body[index]).unwrap());
                            index += 1;
                            continue;
                        };
                        if let Expr::Identifier { name, .. } = expression {
                            if name == "set" {
                                if let Some(Member::Statement { statement, .. }) =
                                    body.get(index + 1)
                                {
                                    if let Stmt::ExpressionStatement { expression, .. } = statement
                                    {
                                        if let Expr::Binary {
                                            op, left, right, ..
                                        } = expression
                                        {
                                            if op == "=" {
                                                self.set_assignment(
                                                    left,
                                                    right,
                                                    &mut parameters,
                                                    &mut secrets,
                                                );
                                                index += 2;
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }
                            if name == "start" {
                                if let Some(Member::Statement { statement, .. }) =
                                    body.get(index + 1)
                                {
                                    if let Stmt::ExpressionStatement {
                                        expression: Expr::List { elements, .. },
                                        ..
                                    } = statement
                                    {
                                        for element in elements {
                                            if let Expr::StringLiteral { value, .. } = element {
                                                if let Some(service) = self.services.get(value) {
                                                    start.push(json!({
                                                        "kind": "StartEntry",
                                                        "service": self.reference("ServiceRef", &service.id, self.app_file, element.span()),
                                                        "loc": self.loc(self.app_file, element.span()),
                                                    }));
                                                }
                                            }
                                        }
                                        index += 2;
                                        continue;
                                    }
                                }
                            }
                        }
                        statements.push(serde_json::to_value(&body[index]).unwrap());
                        index += 1;
                    }
                }
                _ => {}
            }
        }
        let application = json!({
            "kind": "ApplicationConfig",
            "roles": roles,
            "roleHierarchy": { "kind": "Hierarchy", "chains": chains },
        });
        let main = json!({
            "kind": "MainEntry",
            "parameters": parameters,
            "secrets": secrets,
            "start": start,
            "statements": statements,
        });
        (application, main)
    }

    fn set_assignment(
        &self,
        left: &Expr,
        right: &Expr,
        parameters: &mut Vec<Value>,
        secrets: &mut Vec<Value>,
    ) {
        let Some(text) = expr_text(left) else {
            return;
        };
        let Some((alias, name)) = text.rsplit_once('.') else {
            return;
        };
        let Some(service_uri) = self.app_aliases.get(alias) else {
            return;
        };
        let parameter = self.artifacts.values().find(|meta| {
            &meta.service == service_uri
                && meta.name == name
                && meta.artifact.kind == ArtifactKind::Parameter
        });
        if let Some(parameter) = parameter {
            parameters.push(json!({
                "kind": "ParameterSet",
                "parameter": self.reference("ParameterRef", &parameter.id, self.app_file, left.span()),
                "value": serde_json::to_value(right).unwrap(),
                "loc": self.loc(self.app_file, left.span()),
            }));
        }
        let secret = self.artifacts.values().find(|meta| {
            &meta.service == service_uri
                && meta.name == name
                && meta.artifact.kind == ArtifactKind::Secret
        });
        if let Some(secret) = secret {
            if let Some(secret_name) =
                expr_text(right).and_then(|text| text.strip_prefix("secret.").map(str::to_string))
            {
                secrets.push(json!({
                    "kind": "SecretSet",
                    "secret": self.reference("SecretRef", &secret.id, self.app_file, left.span()),
                    "name": secret_name,
                    "loc": self.loc(self.app_file, left.span()),
                }));
            }
        }
    }

    fn resolve_app_role(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Identifier { name, .. } => {
                for service in self.closure.application.services.iter() {
                    if let Some(meta) = self.find_artifact(&service.uri, name, "Role") {
                        if meta.artifact.kind == ArtifactKind::Role {
                            return Some(meta.id.clone());
                        }
                    }
                }
                self.artifacts
                    .values()
                    .find(|meta| meta.name == *name && meta.artifact.kind == ArtifactKind::Role)
                    .map(|meta| meta.id.clone())
            }
            Expr::MemberAccess { object, name, .. } => {
                let alias = expr_text(object)?;
                let service_uri = self.app_aliases.get(&alias)?;
                self.find_artifact(service_uri, name, "Role")
                    .filter(|meta| meta.artifact.kind == ArtifactKind::Role)
                    .map(|meta| meta.id.clone())
            }
            _ => None,
        }
    }

    // --- roles / permissions ---------------------------------------------

    fn build_role_index(&mut self) {
        let roles: Vec<&ArtifactMeta<'a>> = self
            .artifacts
            .values()
            .filter(|meta| meta.artifact.kind == ArtifactKind::Role)
            .collect();
        let mut own: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut inherits: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for role in &roles {
            let body = &role.artifact.body;
            let mut permissions = Vec::new();
            if let Some(value) = property(body, "permissions") {
                for (expr, _) in expr_items(value) {
                    if let Some(id) = self.resolve_expr_id(expr, &role.service, "Permission") {
                        permissions.push(id);
                    }
                }
            }
            let mut parents = Vec::new();
            if let Some(value) = property(body, "inherits") {
                for (expr, _) in expr_items(value) {
                    if let Some(id) = self.resolve_expr_id(expr, &role.service, "Role") {
                        parents.push(id);
                    }
                }
            }
            own.insert(role.id.clone(), permissions);
            inherits.insert(role.id.clone(), parents);
        }
        let mut effective: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for role in &roles {
            let mut seen = BTreeSet::new();
            let mut stack = vec![role.id.clone()];
            let mut result = Vec::new();
            while let Some(id) = stack.pop() {
                if !seen.insert(id.clone()) {
                    continue;
                }
                if let Some(permissions) = own.get(&id) {
                    for permission in permissions {
                        if !result.contains(permission) {
                            result.push(permission.clone());
                        }
                    }
                }
                if let Some(parents) = inherits.get(&id) {
                    stack.extend(parents.iter().cloned());
                }
            }
            result.sort_by(|left, right| self.decl_name(left).cmp(&self.decl_name(right)));
            effective.insert(role.id.clone(), result);
        }
        let mut inverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (role_id, permissions) in &effective {
            for permission in permissions {
                inverse
                    .entry(permission.clone())
                    .or_default()
                    .push(role_id.clone());
            }
        }
        for roles in inverse.values_mut() {
            roles.sort_by(|left, right| self.decl_name(left).cmp(&self.decl_name(right)));
        }
        for organogram in self
            .artifacts
            .values()
            .filter(|meta| meta.artifact.kind == ArtifactKind::Organogram)
        {
            for role in &organogram.artifact.roles {
                self.role_organogram
                    .entry(role.role_ref.clone())
                    .or_insert_with(|| organogram.id.clone());
            }
        }
        self.effective = effective;
        self.permission_roles = inverse;
    }

    fn decl_name(&self, id: &str) -> String {
        self.decls
            .get(id)
            .map(|decl| decl.name.clone())
            .or_else(|| {
                self.artifacts
                    .values()
                    .find(|meta| meta.id == id)
                    .map(|meta| meta.name.clone())
            })
            .unwrap_or_else(|| id.to_string())
    }

    // --- reintroduce ------------------------------------------------------

    fn build_reintroductions(&mut self) {
        for service in self.services.values() {
            let mut service_imports: BTreeMap<String, String> = BTreeMap::new();
            for member in &service.artifact.body {
                let Member::Block { name, body, .. } = member else {
                    continue;
                };
                if name != "imports" {
                    continue;
                }
                for entry in body {
                    if let Member::ImportDeclaration { uri, alias, .. } = entry {
                        if !alias.is_empty() {
                            service_imports.insert(alias.clone(), uri.clone());
                        }
                    }
                }
            }
            for member in &service.artifact.body {
                let Member::Reintroduce {
                    service: alias,
                    artifact: name,
                    body,
                    span,
                } = member
                else {
                    continue;
                };
                let Some(target_uri) = service_imports.get(alias) else {
                    continue;
                };
                let Some(target) = self.find_artifact(target_uri, name, "") else {
                    continue;
                };
                let merged = self.merged.entry(target.id.clone()).or_default();
                for member in body {
                    match member {
                        Member::Insert {
                            target,
                            position,
                            anchor,
                            ..
                        } => {
                            merged
                                .inserts
                                .push((target.clone(), position.clone(), anchor.clone()))
                        }
                        _ => merged.contributions.push(OriginMember {
                            member: member.clone(),
                            file: service.file.path.clone(),
                        }),
                    }
                }
                let _ = span;
            }
        }
    }

    fn finalize_merges(&mut self) {
        let ids: Vec<String> = self.merged.keys().cloned().collect();
        for id in ids {
            let Some(meta) = self.artifacts.values().find(|meta| meta.id == id) else {
                continue;
            };
            let base: Vec<Member> = meta.artifact.body.clone();
            let base_file = meta.file.path.clone();
            let contributions = self
                .merged
                .get(&id)
                .map(|merged| merged.contributions.clone())
                .unwrap_or_default();
            let inserts = self
                .merged
                .get(&id)
                .map(|merged| merged.inserts.clone())
                .unwrap_or_default();
            let mut merged = self.merged.remove(&id).unwrap_or_default();
            merged.transition_origins.clear();
            merged.action_origins.clear();
            let mut members: Vec<OriginMember> = base
                .into_iter()
                .map(|member| OriginMember {
                    member,
                    file: base_file.clone(),
                })
                .collect();
            for contribution in &contributions {
                merge_contribution(&mut members, contribution, &mut merged);
            }
            for (target, position, anchor) in &inserts {
                apply_insert(&mut members, target, position, anchor);
            }
            merged.members = Some(members);
            self.merged.insert(id, merged);
        }
    }

    // --- files / pending --------------------------------------------------

    fn files(&self) -> Vec<Value> {
        let mut files = Vec::new();
        for file in &self.workspace.files {
            if !self.closure.contains_file(&file.path) {
                continue;
            }
            let application = file.path == self.closure.application.path;
            files.push(json!({
                "kind": "FileEntry",
                "path": self.rel(&file.path),
                "serviceId": file
                    .service
                    .as_ref()
                    .and_then(|service| self.services.get(service).map(|meta| meta.id.clone())),
                "application": application,
            }));
        }
        files.sort_by(|left, right| {
            left.get("path")
                .and_then(Value::as_str)
                .cmp(&right.get("path").and_then(Value::as_str))
        });
        files
    }

    fn pending(&self) -> Vec<Value> {
        let mut pending = Vec::new();
        for file in &self.workspace.files {
            if !self.closure.contains_file(&file.path) {
                continue;
            }
            for entry in &file.ast.pending {
                pending.push(self.pending_json(file, entry));
            }
        }
        pending
    }

    fn pending_json(&self, file: &ParsedFile, pending: &Pending) -> Value {
        let mut value = Map::new();
        value.insert("kind".to_string(), json!("PendingEntry"));
        value.insert("keyword".to_string(), json!(pending.keyword));
        value.insert("message".to_string(), json!(pending.message));
        value.insert("loc".to_string(), self.loc(file, pending.span));
        if let Some(due) = &pending.due_date {
            value.insert("dueDate".to_string(), json!(due));
        }
        if let Some(text) = &pending.date_text {
            value.insert("dateText".to_string(), json!(text));
        }
        Value::Object(value)
    }

    // --- resolution -------------------------------------------------------

    fn find_artifact(&self, service: &str, name: &str, kind: &str) -> Option<&ArtifactMeta<'a>> {
        if kind.is_empty() {
            self.artifacts
                .values()
                .find(|meta| meta.service == service && meta.name == name)
                .or_else(|| self.artifacts.values().find(|meta| meta.name == name))
        } else {
            self.artifacts
                .get(&(
                    service.to_string(),
                    kind.to_ascii_lowercase(),
                    name.to_string(),
                ))
                .or_else(|| {
                    self.artifacts.values().find(|meta| {
                        meta.service == service
                            && meta.name == name
                            && meta.artifact.artifact_kind().eq_ignore_ascii_case(kind)
                    })
                })
        }
    }

    fn resolve_expr_id(&self, expr: &Expr, service: &str, kind: &str) -> Option<String> {
        match expr {
            Expr::Reference { target, name, .. } => {
                if self.id_matches_kind(target, kind) {
                    Some(target.clone())
                } else if !name.is_empty() {
                    self.resolve_artifact_id("", name, kind, service)
                } else {
                    None
                }
            }
            _ => self.resolve_artifact_id("", &expr_text(expr)?, kind, service),
        }
    }

    fn id_matches_kind(&self, id: &str, kind: &str) -> bool {
        if kind.is_empty() {
            return true;
        }
        let actual = self
            .decls
            .get(id)
            .map(|decl| decl.kind.clone())
            .or_else(|| {
                self.artifacts
                    .values()
                    .find(|meta| meta.id == id)
                    .map(|meta| meta.artifact.artifact_kind().to_ascii_lowercase())
            });
        actual
            .map(|actual| actual.eq_ignore_ascii_case(kind))
            .unwrap_or(false)
    }

    fn resolve_expr_artifact(&self, expr: &Expr, service: &str) -> Option<(String, String)> {
        let text = expr_text(expr)?;
        self.resolve_artifact_id("", &text, "", service).map(|id| {
            let kind = self
                .artifacts
                .values()
                .find(|meta| meta.id == id)
                .map(|meta| meta.artifact.artifact_kind().to_string())
                .unwrap_or_default();
            (id, kind)
        })
    }

    fn resolve_artifact_id(
        &self,
        _file: &str,
        text: &str,
        kind: &str,
        service: &str,
    ) -> Option<String> {
        if kind == "Permission" {
            if let Some((id, _)) = self.decls.iter().find(|(_, decl)| {
                decl.kind == "permission"
                    && decl.name == text
                    && decl.service.as_deref() == Some(service)
            }) {
                return Some(id.clone());
            }
        }
        let meta = if let Some((alias, name)) = text.split_once('.') {
            let target = self
                .symbols
                .imports
                .get(service)
                .and_then(|imports| imports.get(alias))
                .or_else(|| self.app_aliases.get(alias))?;
            self.find_artifact(&target, name, kind)
        } else {
            self.find_artifact(service, text, kind)
                .or_else(|| self.artifacts.values().find(|meta| meta.name == text))
        }?;
        if !kind.is_empty() && meta.artifact.kind.name() != kind {
            return None;
        }
        Some(meta.id.clone())
    }

    fn type_of(&self, expr: &Expr, service: &str, file: &ParsedFile, span: Span) -> Value {
        if let Some(id) = self.type_id_of(expr, service) {
            if self.is_type_artifact(&id) {
                return self.type_ref_by_id(&id, file, span);
            }
        }
        let ty = match expr {
            Expr::Object {
                type_ref: Some(ty), ..
            } => ty.clone(),
            Expr::Identifier { name, .. } => name.clone(),
            Expr::Reference { name, .. } => name.clone(),
            _ => String::new(),
        };
        self.type_ref_by_name(&ty, service, file, span)
    }

    fn is_type_artifact(&self, id: &str) -> bool {
        self.artifacts
            .values()
            .find(|meta| meta.id == id)
            .map(|meta| {
                matches!(
                    meta.artifact.kind,
                    ArtifactKind::Shape | ArtifactKind::Document
                )
            })
            .unwrap_or(false)
    }

    fn type_id_of(&self, expr: &Expr, service: &str) -> Option<String> {
        match expr {
            Expr::Object {
                type_ref_id: Some(id),
                ..
            } => Some(id.clone()),
            Expr::Object {
                type_ref: Some(ty), ..
            }
            | Expr::Identifier { name: ty, .. } => {
                let base = base_type(ty);
                self.resolve_artifact_id("", base, "Shape", service)
            }
            Expr::Reference { target, .. } => Some(target.clone()),
            _ => None,
        }
    }

    fn type_ref_by_name(&self, ty: &str, service: &str, file: &ParsedFile, span: Span) -> Value {
        let base = base_type(ty);
        if matches!(base, "string" | "number" | "boolean" | "stream") {
            return json!({ "kind": "PrimitiveRef", "name": base });
        }
        if base.starts_with("std.") {
            return json!({ "kind": "StdRef", "name": base });
        }
        if is_std_type(base) {
            return json!({ "kind": "StdRef", "name": format!("std.{base}") });
        }
        let candidate = self
            .artifacts
            .values()
            .filter(|meta| meta.name == base)
            .filter(|meta| {
                matches!(
                    meta.artifact.kind,
                    ArtifactKind::Shape | ArtifactKind::Document
                )
            })
            .min_by_key(|meta| {
                if meta.service == service {
                    if meta.artifact.kind == ArtifactKind::Shape {
                        0
                    } else {
                        1
                    }
                } else if meta.artifact.kind == ArtifactKind::Shape {
                    2
                } else {
                    3
                }
            });
        if let Some(meta) = candidate {
            return self.type_ref_by_id(&meta.id, file, span);
        }
        json!({ "kind": "StdRef", "name": format!("std.{base}") })
    }

    fn type_ref_by_id(&self, id: &str, file: &ParsedFile, span: Span) -> Value {
        if let Some(decl) = self.decls.get(id) {
            let kind = ref_kind_name(&decl.kind);
            return self.reference(kind, id, file, span);
        }
        if let Some(meta) = self.artifacts.values().find(|meta| meta.id == id) {
            let kind = ref_kind_name(meta.artifact.artifact_kind());
            return self.reference(kind, id, file, span);
        }
        json!({ "kind": "StdRef", "name": "std.any" })
    }

    fn role_meta(&self, id: &str) -> Option<&ArtifactMeta<'a>> {
        self.artifacts
            .values()
            .find(|meta| meta.id == id && meta.artifact.kind == ArtifactKind::Role)
    }

    fn file_of_id(&self, id: &str) -> Option<&'a ParsedFile> {
        self.artifacts
            .values()
            .find(|meta| meta.id == id)
            .map(|meta| meta.file)
    }

    fn shape_value_of_id(&self, id: &str) -> Option<(String, String)> {
        let decl = self.decls.get(id)?;
        let path = decl.entry.base_path();
        let (service, rest) = path.split_once("#shape/")?;
        let (shape, value) = rest.split_once("/value/")?;
        let shape_id = self
            .find_artifact(service, shape, "Shape")
            .map(|meta| meta.id.clone())?;
        Some((shape_id, value.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn property<'a>(members: &'a [Member], name: &str) -> Option<&'a Expr> {
    members.iter().find_map(|member| match member {
        Member::PropertyAssignment {
            name: key, value, ..
        } if key == name => Some(value),
        _ => None,
    })
}

fn block_body<'a>(members: &'a [Member], name: &str) -> Option<&'a [Member]> {
    members.iter().find_map(|member| match member {
        Member::Block {
            name: key, body, ..
        } if key == name => Some(body.as_slice()),
        _ => None,
    })
}

fn property_string(members: &[Member], name: &str) -> Option<String> {
    property(members, name).and_then(expr_text)
}

fn property_string_list(members: &[Member], name: &str) -> Vec<String> {
    property(members, name)
        .map(|value| {
            expr_items(value)
                .into_iter()
                .filter_map(|(expr, _)| expr_text(expr))
                .collect()
        })
        .unwrap_or_default()
}

fn capabilities_of(members: &[Member]) -> Vec<String> {
    let mut result = property_string_list(members, "capabilities");
    if result.is_empty() {
        if let Some(body) = block_body(members, "capabilities") {
            result.extend(body.iter().filter_map(|member| match member {
                Member::Statement { statement, .. } => match statement {
                    Stmt::ExpressionStatement { expression, .. } => expr_text(expression),
                    _ => None,
                },
                _ => None,
            }));
        }
    }
    result
}

fn expr_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::MemberAccess { object, name, .. } => Some(format!("{}.{}", expr_text(object)?, name)),
        Expr::StringLiteral { value, .. } => Some(value.clone()),
        Expr::Reference { name, .. } => Some(name.clone()),
        Expr::Binary {
            op, left, right, ..
        } if op == "-" => Some(format!("{}-{}", expr_text(left)?, expr_text(right)?)),
        _ => None,
    }
}

fn expr_items(expr: &Expr) -> Vec<(&Expr, Span)> {
    match expr {
        Expr::List { elements, .. } => elements.iter().map(|expr| (expr, expr.span())).collect(),
        _ => vec![(expr, expr.span())],
    }
}

fn base_type(ty: &str) -> &str {
    let ty = ty.split('|').next().unwrap_or(ty).trim();
    ty.trim_end_matches("[]").trim()
}

fn is_std_type(name: &str) -> bool {
    matches!(
        name,
        "DateTime"
            | "Date"
            | "Time"
            | "Duration"
            | "Count"
            | "Money"
            | "Currency"
            | "Mass"
            | "Length"
            | "Email"
            | "Phone"
            | "Image"
            | "URL"
            | "Decimal"
    )
}

fn has_shorthand(members: &[Member], name: &str) -> bool {
    members.iter().any(|member| {
        matches!(
            member,
            Member::ShorthandProperty { name: key, .. } if key == name
        )
    })
}

fn has_property_or_shorthand(members: &[Member], name: &str) -> bool {
    property(members, name).is_some() || has_shorthand(members, name)
}

fn members_contain_permission(members: &[Member], permission: &str) -> bool {
    members.iter().any(|member| match member {
        Member::PropertyAssignment { value, .. } => expr_contains_permission(value, permission),
        Member::Block { body, .. } | Member::EnumValueDeclaration { body, .. } => {
            members_contain_permission(body, permission)
        }
        Member::Statement { statement, .. } => statement_contains_permission(statement, permission),
        Member::Reintroduce { body, .. } => members_contain_permission(body, permission),
        Member::Insert { .. }
        | Member::Hierarchy { .. }
        | Member::ServiceEntry { .. }
        | Member::ImportDeclaration { .. }
        | Member::TriggerDeclaration { .. }
        | Member::ShorthandProperty { .. } => false,
    })
}

fn statement_contains_permission(statement: &Stmt, permission: &str) -> bool {
    match statement {
        Stmt::ExpressionStatement { expression, .. } => {
            expr_contains_permission(expression, permission)
        }
        Stmt::ConstDeclaration { init, .. } | Stmt::LetDeclaration { init, .. } => init
            .as_ref()
            .is_some_and(|expr| expr_contains_permission(expr, permission)),
        Stmt::IfStatement {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            expr_contains_permission(condition, permission)
                || members_contain_permission(then_branch, permission)
                || else_branch
                    .as_ref()
                    .is_some_and(|members| members_contain_permission(members, permission))
        }
        Stmt::TryStatement { body, catches, .. } => {
            members_contain_permission(body, permission)
                || catches
                    .iter()
                    .any(|catch| members_contain_permission(&catch.body, permission))
        }
        Stmt::ReturnStatement { value, .. } => value
            .as_ref()
            .is_some_and(|expr| expr_contains_permission(expr, permission)),
        Stmt::RawStatement { .. } => false,
    }
}

fn expr_contains_permission(expr: &Expr, permission: &str) -> bool {
    if expr_text(expr).as_deref() == Some(permission) {
        return true;
    }
    match expr {
        Expr::List { elements, .. } => elements
            .iter()
            .any(|element| expr_contains_permission(element, permission)),
        Expr::Object { members, .. } => members_contain_permission(members, permission),
        Expr::Call { args, .. } => args
            .iter()
            .any(|arg| expr_contains_permission(arg, permission)),
        Expr::NamedArgument { value, .. } => expr_contains_permission(value, permission),
        Expr::MemberAccess { object, .. } => expr_contains_permission(object, permission),
        Expr::Binary { left, right, .. } => {
            expr_contains_permission(left, permission)
                || expr_contains_permission(right, permission)
        }
        Expr::Unary { operand, .. } => expr_contains_permission(operand, permission),
        _ => false,
    }
}

fn bool_value(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::BooleanLiteral { value, .. } => Some(*value),
        _ => None,
    }
}

fn number_value(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::NumberLiteral { value, .. } => value.parse::<f64>().ok().map(|number| number as i64),
        _ => None,
    }
}

fn action_name(name: &str) -> String {
    let name = name.trim_start_matches("action ");
    name.split('(').next().unwrap_or(name).trim().to_string()
}

fn split_permission(name: &str) -> (&str, &str) {
    match name.rsplit_once('.') {
        Some((namespace, action)) => (namespace, action),
        None => ("", name),
    }
}

fn ref_kind_name(kind: &str) -> &'static str {
    match kind {
        "role" | "Role" => "RoleRef",
        "permission" | "Permission" => "PermissionRef",
        "document" | "Document" => "DocumentRef",
        "shape" | "Shape" => "ShapeRef",
        "process" | "Process" => "ProcessRef",
        "view" | "View" => "ViewRef",
        "form" | "Form" => "FormRef",
        "message" | "Message" => "MessageRef",
        "policy" | "Policy" => "PolicyRef",
        "decision" | "Decision" => "DecisionRef",
        "integration" | "Integration" => "IntegrationRef",
        "parameter" | "Parameter" => "ParameterRef",
        "secret" | "Secret" => "SecretRef",
        "icon" | "Icon" => "IconRef",
        "manual" | "Manual" => "ManualRef",
        "error" | "Error" => "ErrorRef",
        "organogram" | "Organogram" => "OrganogramRef",
        _ => "DocumentRef",
    }
}

fn flatten_hierarchy<'a>(expr: &'a Expr, chain: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Binary {
            op, left, right, ..
        } if op == ">" => {
            flatten_hierarchy(left, chain);
            flatten_hierarchy(right, chain);
        }
        _ => chain.push(expr),
    }
}

fn merge_contribution(
    members: &mut Vec<OriginMember>,
    contribution: &OriginMember,
    merged: &mut Merged,
) {
    match &contribution.member {
        Member::EnumValueDeclaration {
            name, body: new_body, ..
        } => {
            if let Some(existing) = members.iter_mut().find(|origin| {
                matches!(&origin.member, Member::EnumValueDeclaration { name: existing, .. } if existing == name)
            }) {
                let Member::EnumValueDeclaration { body, .. } = &mut existing.member else {
                    return;
                };
                for member in new_body {
                    if member_key(member).is_some_and(|key| {
                        body.iter().any(|existing| member_key(existing) == Some(key.clone()))
                    }) {
                        continue;
                    }
                    if let Member::Block {
                        keyword: Some(keyword),
                        name: transition,
                        ..
                    } = member
                    {
                        if keyword == "transition" {
                            merged
                                .transition_origins
                                .entry((name.clone(), transition.clone()))
                                .or_insert_with(|| contribution.file.clone());
                        }
                    }
                    body.push(member.clone());
                }
            } else {
                members.push(contribution.clone());
            }
        }
        Member::Block { name, .. } if name.starts_with("action ") => {
            let normalized = name.trim_start_matches("action ").to_string();
            let action = action_name(&normalized);
            if let Some(actions) = members.iter_mut().find_map(|origin| match &mut origin.member {
                Member::Block { name, body, .. } if name == "actions" => Some(body),
                _ => None,
            }) {
                push_action(actions, &contribution.member, &normalized, &mut merged.action_origins, &contribution.file);
            } else {
                let mut actions = Vec::new();
                push_action(&mut actions, &contribution.member, &normalized, &mut merged.action_origins, &contribution.file);
                members.push(OriginMember {
                    member: Member::Block {
                        keyword: None,
                        name: "actions".to_string(),
                        body: actions,
                        span: contribution.member.span(),
                    },
                    file: contribution.file.clone(),
                });
            }
            let _ = action;
        }
        _ => members.push(contribution.clone()),
    }
}

fn push_action(
    actions: &mut Vec<Member>,
    block: &Member,
    normalized: &str,
    action_origins: &mut BTreeMap<String, String>,
    file: &str,
) {
    let Member::Block { body, span, .. } = block else {
        return;
    };
    let name = action_name(normalized);
    if actions.iter().any(|member| {
        matches!(member, Member::Block { name: existing, .. } if action_name(existing) == name)
    }) {
        return;
    }
    actions.push(Member::Block {
        keyword: None,
        name: normalized.to_string(),
        body: body.clone(),
        span: *span,
    });
    action_origins.insert(name, file.to_string());
}

fn member_key(member: &Member) -> Option<String> {
    match member {
        Member::PropertyAssignment { name, .. } => Some(format!("property:{name}")),
        Member::Block { keyword, name, .. } => Some(format!(
            "block:{}:{name}",
            keyword.clone().unwrap_or_default()
        )),
        Member::EnumValueDeclaration { name, .. } => Some(format!("value:{name}")),
        Member::ShorthandProperty { name, .. } => Some(format!("property:{name}")),
        _ => None,
    }
}

fn apply_insert(members: &mut Vec<OriginMember>, target: &str, position: &str, anchor: &str) {
    let target_index = members.iter().position(|origin| {
        matches!(&origin.member, Member::EnumValueDeclaration { name, .. } if name == target)
    });
    let anchor_index = members.iter().position(|origin| {
        matches!(&origin.member, Member::EnumValueDeclaration { name, .. } if name == anchor)
    });
    let (Some(target_index), Some(mut anchor_index)) = (target_index, anchor_index) else {
        return;
    };
    if target_index == anchor_index {
        return;
    }
    let member = members.remove(target_index);
    if target_index < anchor_index {
        anchor_index -= 1;
    }
    let index = if position == "after" {
        anchor_index + 1
    } else {
        anchor_index
    };
    members.insert(index.min(members.len()), member);
}
