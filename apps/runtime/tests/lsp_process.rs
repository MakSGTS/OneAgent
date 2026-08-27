use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use oneagent_runtime::workspace_root_uri;
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

fn initialize(root_uri: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {"general": {"positionEncodings": ["utf-16"]}},
            "workspaceFolders": [{"uri": root_uri, "name": "workspace"}]
        }
    })
    .to_string()
}

fn notification(method: &str) -> String {
    json!({"jsonrpc": "2.0", "method": method}).to_string()
}

fn request(id: u64, method: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method}).to_string()
}

fn request_with_params(id: u64, method: &str, params: &Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

async fn run_process(root: &Path, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_oneagent-lsp"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("LSP process must spawn");
    let mut stdin = child.stdin.take().expect("piped stdin must exist");
    stdin.write_all(input).await.expect("test input must write");
    stdin.flush().await.expect("test input must flush");
    drop(stdin);
    timeout(PROCESS_TIMEOUT, child.wait_with_output())
        .await
        .expect("LSP process must not hang")
        .expect("LSP process must be waitable")
}

fn decode_frames(mut bytes: &[u8]) -> Vec<Value> {
    let mut values = Vec::new();
    while !bytes.is_empty() {
        let delimiter = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("response header delimiter");
        let header = std::str::from_utf8(&bytes[..delimiter]).expect("ASCII response header");
        let length = header
            .strip_prefix("Content-Length: ")
            .expect("Content-Length response header")
            .parse::<usize>()
            .expect("response length");
        let start = delimiter + 4;
        let end = start + length;
        values.push(serde_json::from_slice(&bytes[start..end]).expect("response JSON"));
        bytes = &bytes[end..];
    }
    values
}

#[tokio::test]
async fn public_lsp_process_completes_lifecycle_with_pure_framed_stdout() {
    let root = tempdir().expect("temporary Workspace must be created");
    let root_uri = workspace_root_uri(root.path()).expect("root URI must encode");
    let input = [
        frame(&initialize(&root_uri)),
        frame(&notification("initialized")),
        frame(&request(2, "shutdown")),
        frame(&notification("exit")),
    ]
    .concat();

    let first = run_process(root.path(), &input).await;
    let repeated = run_process(root.path(), &input).await;
    assert!(
        first.status.success(),
        "status={:?}, stdout={}, stderr={}",
        first.status,
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, repeated.stdout);
    assert_eq!(first.stderr, repeated.stderr);
    let responses = decode_frames(&first.stdout);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "oneagent");
    assert_eq!(
        responses[0]["result"]["capabilities"],
        json!({
            "positionEncoding": "utf-16",
            "textDocumentSync": 0,
            "workspaceSymbolProvider": true,
            "diagnosticProvider": {
                "identifier": "oneagent",
                "interFileDependencies": true,
                "workspaceDiagnostics": false
            }
        })
    );
    assert_eq!(responses[1]["result"], Value::Null);
}

#[tokio::test]
async fn public_lsp_process_rejects_non_lsp_integer_fields() {
    let root = tempdir().expect("temporary Workspace must be created");
    let root_uri = workspace_root_uri(root.path()).expect("root URI must encode");
    let mut invalid_process_id: Value =
        serde_json::from_str(&initialize(&root_uri)).expect("initialize request JSON");
    invalid_process_id["id"] = json!(2);
    invalid_process_id["params"]["processId"] = json!(i64::from(i32::MAX) + 1);
    let input = [
        frame(
            &json!({
                "jsonrpc": "2.0",
                "id": i64::from(i32::MAX) + 1,
                "method": "initialize",
                "params": invalid_process_id["params"].clone()
            })
            .to_string(),
        ),
        frame(&invalid_process_id.to_string()),
        frame(&initialize(&root_uri)),
        frame(&notification("initialized")),
        frame(
            &json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "workspace/symbol",
                "params": {"query": "", "workDoneToken": 1.5}
            })
            .to_string(),
        ),
        frame(
            &json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "workspace/symbol",
                "params": {
                    "query": "",
                    "partialResultToken": i64::from(i32::MAX) + 1
                }
            })
            .to_string(),
        ),
        frame(&request(5, "shutdown")),
        frame(&notification("exit")),
    ]
    .concat();

    let output = run_process(root.path(), &input).await;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let responses = decode_frames(&output.stdout);
    assert_eq!(responses.len(), 6);
    assert_eq!(responses[0]["id"], Value::Null);
    assert_eq!(responses[0]["error"]["code"], -32600);
    assert_eq!(responses[1]["error"]["code"], -32602);
    assert_eq!(responses[2]["result"]["serverInfo"]["name"], "oneagent");
    for response in &responses[3..5] {
        assert_eq!(response["error"]["code"], -32602);
    }
    assert_eq!(responses[5]["result"], Value::Null);
}

#[tokio::test]
async fn public_lsp_process_projects_edt_and_designer_workspace_symbols() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service");
    let root_uri = workspace_root_uri(&root).expect("fixture root URI must encode");
    let input = [
        frame(&initialize(&root_uri)),
        frame(&notification("initialized")),
        frame(&request_with_params(
            2,
            "workspace/symbol",
            &json!({"query": ""}),
        )),
        frame(&request_with_params(
            3,
            "workspace/symbol",
            &json!({"query": "fillsecurity"}),
        )),
        frame(&request_with_params(
            4,
            "workspace/symbol",
            &json!({"query": "absent"}),
        )),
        frame(&request_with_params(
            5,
            "workspace/symbol",
            &json!({"query": "", "unknown": true}),
        )),
        frame(&request(6, "shutdown")),
        frame(&notification("exit")),
    ]
    .concat();

    let first = run_process(&root, &input).await;
    let repeated = run_process(&root, &input).await;
    assert!(first.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, repeated.stdout);
    let responses = decode_frames(&first.stdout);
    assert_eq!(responses.len(), 6);

    let symbols = responses[1]["result"]
        .as_array()
        .expect("workspace symbols must be an array");
    assert_eq!(symbols.len(), 4);
    assert_eq!(
        symbols
            .iter()
            .map(|symbol| symbol["name"].as_str().expect("symbol name"))
            .collect::<Vec<_>>(),
        [
            "FillSecurityCollection",
            "Posting",
            "Query",
            "ReadMissingCatalog"
        ]
    );
    assert!(symbols.iter().all(|symbol| {
        matches!(symbol["kind"].as_u64(), Some(12 | 19))
            && symbol["location"]["uri"]
                .as_str()
                .is_some_and(|uri| uri.starts_with(&root_uri))
            && symbol["location"]["range"]["start"]["character"] == 0
            && symbol["location"]["range"]["start"] == symbol["location"]["range"]["end"]
    }));
    assert_eq!(
        symbols.iter().filter(|symbol| symbol["kind"] == 19).count(),
        1
    );

    let designer = responses[2]["result"]
        .as_array()
        .expect("filtered symbols must be an array");
    assert_eq!(designer.len(), 1);
    assert!(
        designer[0]["location"]["uri"]
            .as_str()
            .expect("Designer symbol URI")
            .contains("/designer/CommonModules/")
    );
    assert_eq!(responses[3]["result"], json!([]));
    assert_eq!(
        responses[4]["error"],
        json!({"code": -32602, "message": "Invalid params"})
    );
}

#[tokio::test]
async fn public_lsp_process_pulls_located_diagnostics_and_empty_full_reports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/workspace_service");
    let root_uri = workspace_root_uri(&root).expect("fixture root URI must encode");
    let edt_uri = format!("{root_uri}/edt/src/Documents/RefundOfPaymentByOrder/ObjectModule.bsl");
    let designer_uri =
        format!("{root_uri}/designer/CommonModules/DynamicSecurityOverridable/Ext/Module.bsl");
    let diagnostic_params = |uri: &str| json!({"textDocument": {"uri": uri}});
    let input = [
        frame(&initialize(&root_uri)),
        frame(&notification("initialized")),
        frame(&request_with_params(
            2,
            "textDocument/diagnostic",
            &diagnostic_params(&edt_uri),
        )),
        frame(&request_with_params(
            3,
            "textDocument/diagnostic",
            &json!({
                "textDocument": {"uri": edt_uri},
                "identifier": "oneagent",
                "previousResultId": "ignored"
            }),
        )),
        frame(&request_with_params(
            4,
            "textDocument/diagnostic",
            &diagnostic_params(&designer_uri),
        )),
        frame(&request_with_params(
            5,
            "textDocument/diagnostic",
            &diagnostic_params(&format!("{root_uri}/missing.bsl")),
        )),
        frame(&request_with_params(
            6,
            "textDocument/diagnostic",
            &diagnostic_params("file:///outside.bsl"),
        )),
        frame(&request_with_params(
            7,
            "textDocument/diagnostic",
            &diagnostic_params(&format!("{root_uri}/bad%2fname.bsl")),
        )),
        frame(&request_with_params(
            8,
            "textDocument/diagnostic",
            &diagnostic_params(&format!("{root_uri}/sub%5C..%5Coutside.bsl")),
        )),
        frame(&request(9, "shutdown")),
        frame(&notification("exit")),
    ]
    .concat();

    let first = run_process(&root, &input).await;
    let repeated = run_process(&root, &input).await;
    assert!(
        first.status.success(),
        "stdout={}, stderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, repeated.stdout);
    let responses = decode_frames(&first.stdout);
    assert_eq!(responses.len(), 9);

    assert_diagnostic_responses(&responses);
}

fn assert_diagnostic_responses(responses: &[Value]) {
    let edt_report = &responses[1]["result"];
    assert_eq!(edt_report["kind"], "full");
    assert!(edt_report.get("resultId").is_none());
    let diagnostics = edt_report["items"]
        .as_array()
        .expect("diagnostic items must be an array");
    assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic["severity"] == 1
            && diagnostic["source"] == "oneagent"
            && diagnostic["range"]["start"]["character"] == 0
            && diagnostic["range"]["start"] == diagnostic["range"]["end"]
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["code"] == "semantic.reference.unresolved"
            && diagnostic["message"] == "semantic reference target could not be resolved"
    }));
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic["code"].as_str().expect("diagnostic code"),
                diagnostic["message"].as_str().expect("diagnostic message"),
                diagnostic["range"]["start"]["line"]
                    .as_u64()
                    .expect("diagnostic line")
            ))
            .collect::<Vec<_>>(),
        [
            (
                "semantic.reference.unresolved",
                "semantic reference target could not be resolved",
                0
            ),
            (
                "semantic.reference.unresolved",
                "semantic reference target could not be resolved",
                0
            ),
            (
                "semantic.reference.unresolved",
                "query source metadata target could not be resolved",
                6
            )
        ]
    );
    assert_eq!(responses[2]["result"], *edt_report);
    assert_eq!(responses[3]["result"], json!({"kind": "full", "items": []}));
    assert_eq!(responses[4]["result"], json!({"kind": "full", "items": []}));
    for response in &responses[5..8] {
        assert_eq!(
            response["error"],
            json!({"code": -32602, "message": "Invalid params"})
        );
    }
}

#[tokio::test]
async fn public_lsp_process_rejects_conflicting_root_and_early_exit() {
    let root = tempdir().expect("temporary Workspace must be created");
    let input = [
        frame(&initialize("file:///different")),
        frame(&notification("exit")),
    ]
    .concat();
    let output = run_process(root.path(), &input).await;
    assert!(!output.status.success());
    assert_eq!(output.stderr, b"oneagent-lsp: lifecycle failure\n");
    let responses = decode_frames(&output.stdout);
    assert_eq!(responses.len(), 1);
    assert_eq!(
        responses[0]["error"],
        json!({"code": -32602, "message": "Invalid params"})
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains(root.path().to_string_lossy().as_ref())
    );
}

#[tokio::test]
async fn public_lsp_process_reports_bounded_terminal_categories() {
    let root = tempdir().expect("temporary Workspace must be created");
    let eof = run_process(root.path(), b"").await;
    assert!(!eof.status.success());
    assert!(eof.stdout.is_empty());
    assert_eq!(eof.stderr, b"oneagent-lsp: unexpected end of input\n");

    let malformed = run_process(root.path(), b"Content-Length: 2\n\n{}").await;
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert_eq!(malformed.stderr, b"oneagent-lsp: invalid header\n");
}
