# Archol preview for DeepSeek Harness

DeepSeek Harness plugin (bundle) that opens `.archol` files in the right
Sidebar with:

- syntax highlighting from the Archol lexer;
- an artifact outline that jumps to the declaration's source line;
- the current file's diagnostics, with the same jump;
- semantic diagnostics and the application URI when the file belongs to an
  application closure;
- plain-source fallback when the compiler is not available.

The plugin is read-only. It reads the file through the Harness workspace file
Remote and runs the local compiler; it sends nothing off the machine.

## Requirements

- DeepSeek Harness with the Web right Sidebar document preview
  (`@deepseek-ai/dsh-client-ui-sidebar-documentpreview`).
- The `archol` compiler with the `view` command. Build it from this repository:

  ```bash
  cd archol && cargo build --release -p archol-cli
  ```

  The Host route looks for the binary in this order: `ARCHOL_COMPILER`, then
  `archol/target/{release,debug}/archol` two levels above this plugin, and
  finally `archol` on `PATH`. It never executes a binary found under the
  previewed workspace.

## Install

From the workspace root:

```bash
# In the Harness UI: Plugins > Add plugin > this directory
# (or through the plugin_manager tool with action install_bundle)
archol/editors/dsh
```

The bundle inserts one Host row (`archol-preview`) that serves
`GET /archol-preview/view?path=<absolute .archol path>&app=<optional application>`
and one Client half that registers the `archol` document type.

## How it fits together

```text
Right Sidebar file tab (.archol)
  └─ text preview (paged source from workspaceFiles)
       └─ archol document body
            ├─ workspaceFiles.stat  -> absolute path
            ├─ GET /archol-preview/view?path=...
            │     ├─ application chosen by the Host (see below)
            │     └─ archol view <file> --json [--app <uri|path>]
            └─ tokens + outline + diagnostics
```

`archol view --json` is the compiler parameter set this plugin consumes. It
emits one compact object with `tokens` (highlight class + byte span), `outline`
(artifact kind, name, line/column), the pending-comment count, the file's
`diagnostics` and, with `--app`, the application URI/path.

### Application selection

For semantic diagnostics the Host needs an application. It uses, in order:

1. the `app` query parameter (URI or absolute application path);
2. `ARCHOL_APPLICATION` in the Harness process environment;
3. the application that accepted the same file in an earlier request;
4. the first application in the workspace whose closure contains the file.

When no application accepts the file, the Host falls back to the syntax-only
payload instead of failing, so highlighting keeps working for files outside
every closure. The chosen application appears at the bottom of the Structure
panel.

## Behavior

- The Structure chip (artifact count) toggles the outline panel; outline and
  diagnostic rows navigate through the Sidebar's own line navigation, which
  loads the relevant page, scrolls and marks the line.
- The diagnostics chip (count) toggles the diagnostics list. Every diagnostic
  is listed with `line:column severity` and its full message.
- A failed analysis leaves plain source visible, names the failure, and offers
  **Retry**; retrying does not poison the Host's compiler choice.
- The viewer is chosen automatically for `.archol`; plain text stays available
  in the viewer menu.

## Limitations

- The Host route reads any absolute `.archol` path that the local process can
  read; keep the Harness Web server bound to loopback.
- Remote Sessions resolve paths in a remote execution world that the local
  compiler cannot read; remote profiles fall back to plain source.
- Highlighting covers the pages loaded so far; the plain-text preview owns
  paging and reload.
- The compiler runs on the file on disk, not on unsaved editor buffers.
