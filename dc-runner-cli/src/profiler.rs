use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn now_iso_utc_fallback() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn level_enabled(level: &str) -> bool {
    !matches!(level, "" | "off")
}

fn sanitize_attrs(attrs: Value) -> Value {
    fn redact_key(k: &str) -> bool {
        let low = k.to_ascii_lowercase();
        low.contains("token")
            || low.contains("secret")
            || low.contains("password")
            || low.contains("authorization")
            || low.contains("cookie")
            || low.contains("key")
    }
    match attrs {
        Value::Object(mut obj) => {
            let keys = obj.keys().cloned().collect::<Vec<_>>();
            for k in keys {
                let v = obj.remove(&k).unwrap_or(Value::Null);
                if redact_key(&k) {
                    obj.insert(k, Value::String("<redacted>".to_string()));
                } else {
                    obj.insert(k, sanitize_attrs(v));
                }
            }
            Value::Object(obj)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sanitize_attrs).collect()),
        other => other,
    }
}

#[derive(Debug, Clone)]
pub struct ProfileOptions {
    pub level: String,
    pub runner_impl: String,
    pub command: String,
    pub args: Vec<String>,
}

pub struct RunProfiler {
    enabled: bool,
    run_id: String,
    runner_impl: String,
    command: String,
    args: Vec<String>,
    started_at: String,
    started_monotonic: Instant,
    next_id: u64,
    spans: Vec<Value>,
    events: Vec<Value>,
    run_span_id: Option<String>,
}

impl RunProfiler {
    pub fn from_options(opts: &ProfileOptions) -> Self {
        let enabled = level_enabled(opts.level.as_str());
        let mut profiler = Self {
            enabled,
            run_id: format!(
                "run-{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            ),
            runner_impl: opts.runner_impl.clone(),
            command: opts.command.clone(),
            args: opts.args.clone(),
            started_at: now_iso_utc_fallback(),
            started_monotonic: Instant::now(),
            next_id: 0,
            spans: Vec::new(),
            events: Vec::new(),
            run_span_id: None,
        };
        if enabled {
            let sid = profiler.start_span(
                "run.total",
                "run",
                "run.total",
                None,
                json!({"runner_impl": opts.runner_impl}),
            );
            profiler.run_span_id = sid;
        }
        profiler
    }

    fn next_span_id(&mut self) -> String {
        self.next_id += 1;
        format!("s{}", self.next_id)
    }

    fn now_ns(&self) -> i64 {
        self.started_monotonic.elapsed().as_nanos() as i64
    }

    pub fn start_span(
        &mut self,
        name: &str,
        kind: &str,
        phase: &str,
        parent_span_id: Option<String>,
        attrs: Value,
    ) -> Option<String> {
        if !self.enabled {
            return None;
        }
        let sid = self.next_span_id();
        self.spans.push(json!({
            "span_id": sid,
            "parent_span_id": parent_span_id,
            "kind": kind,
            "name": name,
            "phase": phase,
            "start_ns": self.now_ns(),
            "end_ns": Value::Null,
            "duration_ms": Value::Null,
            "status": "ok",
            "attrs": sanitize_attrs(attrs),
            "error": Value::Null
        }));
        Some(sid)
    }

    pub fn finish_span(&mut self, span_id: &str, status: &str, error: Option<Value>) {
        if !self.enabled {
            return;
        }
        let end_ns = self.now_ns();
        for item in self.spans.iter_mut().rev() {
            if item.get("span_id").and_then(Value::as_str) != Some(span_id) {
                continue;
            }
            if !item.get("end_ns").unwrap_or(&Value::Null).is_null() {
                return;
            }
            let start_ns = item.get("start_ns").and_then(Value::as_i64).unwrap_or(0);
            let duration_ms = ((end_ns - start_ns) as f64) / 1_000_000.0;
            if let Some(obj) = item.as_object_mut() {
                obj.insert("end_ns".to_string(), json!(end_ns));
                obj.insert(
                    "duration_ms".to_string(),
                    json!((duration_ms * 1000.0).round() / 1000.0),
                );
                obj.insert("status".to_string(), json!(status));
                obj.insert(
                    "error".to_string(),
                    sanitize_attrs(error.unwrap_or(Value::Null)),
                );
            }
            return;
        }
    }

    pub fn event(&mut self, kind: &str, span_id: Option<&str>, attrs: Value) {
        if !self.enabled {
            return;
        }
        self.events.push(json!({
            "ts_ns": self.now_ns(),
            "kind": kind,
            "span_id": span_id,
            "attrs": sanitize_attrs(attrs),
        }));
    }

    pub fn close(
        &mut self,
        status: &str,
        exit_code: i32,
        root: &Path,
        out_path_raw: &str,
        summary_out_path_raw: &str,
    ) {
        if !self.enabled {
            return;
        }
        if let Some(sid) = self.run_span_id.clone() {
            self.finish_span(&sid, status, None);
        }
        let out_path = root.join(out_path_raw.trim_start_matches('/'));
        let summary_path = root.join(summary_out_path_raw.trim_start_matches('/'));
        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Some(parent) = summary_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let mut env_profile = Map::new();
        for key in [
            "SPEC_RUNNER_PROFILE_LEVEL",
            "SPEC_RUNNER_PROFILE_HEARTBEAT_MS",
            "SPEC_RUNNER_PROFILE_STALL_THRESHOLD_MS",
            "SPEC_RUNNER_LIVENESS_LEVEL",
            "SPEC_RUNNER_LIVENESS_STALL_MS",
            "SPEC_RUNNER_LIVENESS_MIN_EVENTS",
            "SPEC_RUNNER_LIVENESS_HARD_CAP_MS",
            "SPEC_RUNNER_LIVENESS_KILL_GRACE_MS",
            "SPEC_RUNNER_IMPL",
        ] {
            let raw = std::env::var(key).unwrap_or_default();
            env_profile.insert(
                key.to_string(),
                json!({
                    "set": !raw.is_empty(),
                    "length": raw.len(),
                }),
            );
        }
        let payload = json!({
            "version": 1,
            "run_id": self.run_id,
            "runner_impl": self.runner_impl,
            "started_at": self.started_at,
            "ended_at": now_iso_utc_fallback(),
            "status": status,
            "exit_code": exit_code,
            "command": self.command,
            "args": self.args,
            "env_profile": env_profile,
            "spans": self.spans,
            "events": self.events,
            "summary": {
                "span_count": self.spans.len(),
                "event_count": self.events.len()
            }
        });
        let _ = fs::write(
            &out_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
            ),
        );

        let mut spans_sorted = self.spans.clone();
        spans_sorted.sort_by(|a, b| {
            let aa = a.get("duration_ms").and_then(Value::as_f64).unwrap_or(0.0);
            let bb = b.get("duration_ms").and_then(Value::as_f64).unwrap_or(0.0);
            bb.partial_cmp(&aa).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut md = String::new();
        md.push_str("# Run Trace Summary\n\n");
        md.push_str(&format!("- run_id: `{}`\n", self.run_id));
        md.push_str(&format!("- status: `{}`\n", status));
        md.push_str(&format!("- runner_impl: `{}`\n", self.runner_impl));
        md.push_str(&format!("- span_count: `{}`\n", self.spans.len()));
        md.push_str(&format!("- event_count: `{}`\n\n", self.events.len()));
        md.push_str("## Slowest Spans\n\n");
        md.push_str("| span_id | name | phase | status | duration_ms |\n");
        md.push_str("|---|---|---|---|---|\n");
        for row in spans_sorted.iter().take(20) {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
                row.get("span_id").and_then(Value::as_str).unwrap_or(""),
                row.get("name").and_then(Value::as_str).unwrap_or(""),
                row.get("phase").and_then(Value::as_str).unwrap_or(""),
                row.get("status").and_then(Value::as_str).unwrap_or(""),
                row.get("duration_ms")
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            ));
        }
        md.push_str("\n## Suggested Next Command\n\n");
        md.push_str("- rerun with `--profile-level debug --profile-heartbeat-ms 250 --profile-stall-threshold-ms 2000`\n");
        let _ = fs::write(&summary_path, md);
    }
}

pub fn profile_options_from_env(command: &str, args: &[String]) -> ProfileOptions {
    ProfileOptions {
        level: std::env::var("SPEC_RUNNER_PROFILE_LEVEL").unwrap_or_else(|_| "off".to_string()),
        runner_impl: std::env::var("SPEC_RUNNER_IMPL").unwrap_or_else(|_| "rust".to_string()),
        command: command.to_string(),
        args: args.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiler_records_span_and_event() {
        let opts = ProfileOptions {
            level: "basic".to_string(),
            runner_impl: "rust".to_string(),
            command: "governance".to_string(),
            args: vec![],
        };
        let mut p = RunProfiler::from_options(&opts);
        let sid = p
            .start_span(
                "runner.dispatch",
                "runner",
                "runner.dispatch",
                None,
                json!({}),
            )
            .expect("span");
        p.event("checkpoint", Some(&sid), json!({"ok":true}));
        p.finish_span(&sid, "ok", None);
        assert!(p.spans.len() >= 2);
        assert!(!p.events.is_empty());
    }

    #[test]
    fn profiler_close_writes_artifacts() {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("spec-runner-profiler-test-{uniq}"));
        fs::create_dir_all(&root).expect("mkdir");
        let opts = ProfileOptions {
            level: "basic".to_string(),
            runner_impl: "rust".to_string(),
            command: "governance".to_string(),
            args: vec![],
        };
        let mut p = RunProfiler::from_options(&opts);
        p.close(
            "pass",
            0,
            &root,
            "/.artifacts/run-trace.json",
            "/.artifacts/run-trace-summary.md",
        );
        assert!(root.join(".artifacts/run-trace.json").exists());
        assert!(root.join(".artifacts/run-trace-summary.md").exists());
    }
}
