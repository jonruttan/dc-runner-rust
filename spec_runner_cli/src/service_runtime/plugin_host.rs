use serde_json::{json, Value as JsonValue};
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub fn call_json_rpc(
    plugin_bin: &str,
    method: &str,
    params: JsonValue,
) -> Result<JsonValue, String> {
    call_json_rpc_with_timeout(plugin_bin, method, params, 30_000)
}

pub fn call_json_rpc_with_timeout(
    plugin_bin: &str,
    method: &str,
    params: JsonValue,
    timeout_ms: u64,
) -> Result<JsonValue, String> {
    let mut child = Command::new(plugin_bin)
        .arg("--plugin-rpc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("runtime.plugin.process.spawn_failed {plugin_bin}: {e}"))?;

    let payload = json!({
        "jsonrpc": "2.0",
        "id": "1",
        "method": method,
        "params": params
    })
    .to_string();

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("runtime.plugin.process.write_failed: {e}"))?;
    } else {
        return Err("runtime.plugin.process.stdin_unavailable".to_string());
    }
    // Ensure plugin sees EOF on stdin for single-request mode.
    drop(child.stdin.take());

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if start.elapsed() > Duration::from_millis(timeout_ms) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "runtime.plugin.process.timeout method={} timeout_ms={}",
                        method, timeout_ms
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(format!("runtime.plugin.process.wait_failed: {e}")),
        }
    }

    let out = child
        .wait_with_output()
        .map_err(|e| format!("runtime.plugin.process.wait_failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "runtime.plugin.process.non_zero_exit code={} stderr={}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    serde_json::from_slice::<JsonValue>(&out.stdout)
        .map_err(|e| format!("runtime.plugin.process.invalid_json_response: {e}"))
}
