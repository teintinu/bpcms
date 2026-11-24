#!/usr/bin/env python3
"""Archol LSP smoke test: initialize, hover, completion, AST, links.

Usage: python3 archol/scripts/smoke_lsp.py [binary]

The default binary is archol/target/release/archol (or debug). The server is
started with the `lsp` subcommand, just like the VS Code plugin.
"""

import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EXAMPLES = os.path.join(ROOT, "examples")
BIN = (
    sys.argv[1]
    if len(sys.argv) > 1
    else os.path.join(ROOT, "archol", "target", "release", "archol")
)
if not os.path.exists(BIN):
    BIN = os.path.join(ROOT, "archol", "target", "debug", "archol")

proc = subprocess.Popen(
    [BIN, "lsp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
)


def send(message):
    body = json.dumps(message).encode()
    proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    proc.stdin.flush()


def read():
    headers = {}
    while True:
        line = proc.stdout.readline()
        if not line:
            return None
        line = line.strip()
        if not line:
            break
        key, _, value = line.decode().partition(":")
        headers[key.strip().lower()] = value.strip()
    return json.loads(proc.stdout.read(int(headers["content-length"])))


def request(id_, method, params):
    send({"jsonrpc": "2.0", "id": id_, "method": method, "params": params})
    while True:
        message = read()
        if message is None:
            raise SystemExit("server exited")
        if message.get("id") == id_:
            return message


def main():
    assert "capabilities" in request(
        1,
        "initialize",
        {
            "rootUri": "file://" + EXAMPLES,
            "capabilities": {},
            "initializationOptions": {"application": "example.com/app/backoffice-loja"},
        },
    )["result"]
    send({"jsonrpc": "2.0", "method": "initialized", "params": {}})

    shapes = os.path.join(
        EXAMPLES, "services/example.com/services/billing/billing-shapes.archol"
    )
    text = open(shapes, encoding="utf-8").read()
    uri = "file://" + shapes
    send(
        {
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "archol",
                    "version": 1,
                    "text": text,
                }
            },
        }
    )
    line = next(
        i for i, l in enumerate(text.split("\n")) if l.startswith("shape TaxRegimeCode")
    )
    hover = request(
        2,
        "textDocument/hover",
        {"textDocument": {"uri": uri}, "position": {"line": line, "character": 8}},
    )["result"]
    assert "TaxRegimeCode" in hover["contents"]["value"]
    assert "id: `" in hover["contents"]["value"]
    assert "command:archol.showAst" in hover["contents"]["value"]

    temp = os.path.join(
        EXAMPLES,
        "services/example.com/services/billing/tmp-smoke-completion.archol",
    )
    content = "process Novo {\n  const a: TaxRegimeCode = \n}\n"
    with open(temp, "w", encoding="utf-8") as handle:
        handle.write(content)
    try:
        send(
            {
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file://" + temp,
                        "languageId": "archol",
                        "version": 1,
                        "text": content,
                    }
                },
            }
        )
        items = request(
            3,
            "textDocument/completion",
            {
                "textDocument": {"uri": "file://" + temp},
                "position": {"line": 1, "character": 26},
            },
        )["result"]["items"]
        assert {"SIMPLES", "LUCRO_PRESUMIDO", "LUCRO_REAL"} <= {
            item["label"] for item in items
        }
    finally:
        try:
            os.remove(temp)
        except FileNotFoundError:
            pass

    ast = request(4, "archol/applicationAst", {"noBody": True})["result"]["ast"]
    assert ast["kind"] == "ArcholApplication" and ast["artifact"]
    links = request(5, "archol/links", {})["result"]["links"]
    assert links["roles"] and links["enums"]

    dunning = os.path.join(
        EXAMPLES,
        "services/example.com/services/billing/processes/billing-dunning-process.archol",
    )
    dunning_text = open(dunning, encoding="utf-8").read()
    dunning_uri = "file://" + dunning
    send(
        {
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": dunning_uri,
                    "languageId": "archol",
                    "version": 1,
                    "text": dunning_text,
                }
            },
        }
    )

    def position_of(text, offset):
        line = text.count("\n", 0, offset)
        line_start = text.rfind("\n", 0, offset) + 1
        return {"line": line, "character": len(text[line_start:offset].encode("utf-16-le")) // 2}

    condition = dunning_text.index("invoice.status == Overdue")
    status_offset = condition + len("invoice.")
    status_def = request(
        6,
        "textDocument/definition",
        {"textDocument": {"uri": dunning_uri}, "position": position_of(dunning_text, status_offset)},
    )["result"]
    assert status_def["uri"].endswith("billing-invoice-document.archol"), status_def
    target_lines = open(
        "/".join(status_def["uri"].split("/")[2:]), encoding="utf-8"
    ).read().split("\n")
    assert target_lines[status_def["range"]["start"]["line"]].strip().startswith(
        "status: InvoiceStatus"
    ), target_lines[status_def["range"]["start"]["line"]]

    overdue_offset = condition + len("invoice.status == ")
    overdue_def = request(
        7,
        "textDocument/definition",
        {"textDocument": {"uri": dunning_uri}, "position": position_of(dunning_text, overdue_offset)},
    )["result"]
    assert overdue_def["uri"].endswith("billing-invoice-document.archol"), overdue_def
    overdue_lines = open(
        "/".join(overdue_def["uri"].split("/")[2:]), encoding="utf-8"
    ).read().split("\n")
    assert overdue_lines[overdue_def["range"]["start"]["line"]].strip().startswith(
        "value Overdue"
    ), overdue_lines[overdue_def["range"]["start"]["line"]]

    fanout_offset = dunning_text.index("where status == Overdue") + len("where status == ")
    fanout_def = request(
        8,
        "textDocument/definition",
        {"textDocument": {"uri": dunning_uri}, "position": position_of(dunning_text, fanout_offset)},
    )["result"]
    assert fanout_def["uri"].endswith("billing-invoice-document.archol"), fanout_def

    send({"jsonrpc": "2.0", "id": 9, "method": "shutdown", "params": {}})
    send({"jsonrpc": "2.0", "method": "exit", "params": {}})
    print("LSP smoke: OK")


if __name__ == "__main__":
    main()
