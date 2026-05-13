use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::devlog;
use crate::llm::tools::{dispatch, tool_schemas, BuildContext, ToolCtx};
use crate::pob::watcher::PobWatcher;

const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    #[serde(default = "jsonrpc_default")]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

fn jsonrpc_default() -> String {
    "2.0".to_string()
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcErrorObj>,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorObj {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub async fn run_stdio_server() -> Result<()> {
    devlog::log_str("mcp", "event", "server_start");

    // PoB watcher feeds the build context exactly like the TUI does.
    // If no PoB install is detected, we keep going — get_active_build
    // simply returns the no_build payload.
    let watcher = match PobWatcher::start() {
        Ok(w) => Some(w),
        Err(e) => {
            eprintln!("[bestel-mcp] PoB watcher failed to start: {e}");
            None
        }
    };
    let build_ctx = BuildContext::new();
    if let Some(w) = &watcher {
        if let Some(b) = w.initial_build() {
            build_ctx.set(b);
        }
        let mut rx = w.subscribe();
        let ctx_clone = build_ctx.clone();
        tokio::spawn(async move {
            while let Ok(b) = rx.recv().await {
                ctx_clone.set(b);
            }
        });
    }

    let tool_ctx = ToolCtx::new(build_ctx)?;
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = BufWriter::new(stdout);

    while let Some(line) = reader.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        devlog::log_str("mcp_in", "line", trimmed);
        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let resp = error_response(Value::Null, -32700, &format!("parse error: {e}"));
                write_response(&mut writer, &resp).await?;
                continue;
            }
        };
        let is_notification = req.id.is_none();
        let resp_opt = handle(&req, &tool_ctx).await;
        match (is_notification, resp_opt) {
            (true, _) => continue,
            (false, Some(resp)) => write_response(&mut writer, &resp).await?,
            (false, None) => {
                let resp = error_response(
                    req.id.clone().unwrap_or(Value::Null),
                    -32603,
                    "no response produced",
                );
                write_response(&mut writer, &resp).await?;
            }
        }
        if req.method == "shutdown" {
            break;
        }
    }
    devlog::log_str("mcp", "event", "server_stop");
    Ok(())
}

async fn handle(req: &JsonRpcRequest, ctx: &ToolCtx) -> Option<JsonRpcResponse> {
    let id = req.id.clone().unwrap_or(Value::Null);
    match req.method.as_str() {
        "initialize" => Some(success(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "bestel-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        "notifications/initialized" | "initialized" => None,
        "ping" => Some(success(id, json!({}))),
        "tools/list" => {
            let tools: Vec<Value> = tool_schemas()
                .into_iter()
                .map(|s| {
                    let name = s.get("name").cloned().unwrap_or(Value::Null);
                    let desc = s.get("description").cloned().unwrap_or(Value::Null);
                    let schema = s
                        .get("input_schema")
                        .cloned()
                        .unwrap_or(json!({"type": "object"}));
                    json!({
                        "name": name,
                        "description": desc,
                        "inputSchema": schema,
                    })
                })
                .collect();
            Some(success(id, json!({ "tools": tools })))
        }
        "tools/call" => {
            let name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let args = req
                .params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            devlog::log_value(
                "mcp_tool_call",
                json!({ "name": name, "arguments": args.clone() }),
            );
            match dispatch(name, &args, ctx).await {
                Ok(text) => Some(success(
                    id,
                    json!({
                        "content": [{"type": "text", "text": text}],
                        "isError": false
                    }),
                )),
                Err(e) => Some(success(
                    id,
                    json!({
                        "content": [{"type": "text", "text": format!("error: {e}")}],
                        "isError": true
                    }),
                )),
            }
        }
        "shutdown" => Some(success(id, json!({}))),
        "exit" => None,
        other => Some(error_response(
            id,
            -32601,
            &format!("method not found: {other}"),
        )),
    }
}

fn success(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    }
}

fn error_response(id: Value, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcErrorObj {
            code,
            message: message.to_string(),
            data: None,
        }),
    }
}

async fn write_response<W: AsyncWriteExt + Unpin>(
    writer: &mut BufWriter<W>,
    resp: &JsonRpcResponse,
) -> Result<()> {
    let bytes = serde_json::to_vec(resp)?;
    writer.write_all(&bytes).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    devlog::log_str("mcp_out", "line", &String::from_utf8_lossy(&bytes));
    Ok(())
}
