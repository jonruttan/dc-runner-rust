use serde_json::{json, Value};

pub fn ok(command: &str, message: &str, json_mode: bool, details: Value) {
    if json_mode {
        println!(
            "{}",
            json!({
                "ok": true,
                "command": command,
                "message": message,
                "details": details,
            })
        );
    } else {
        println!("OK: {message}");
    }
}

pub fn warn(message: &str) {
    eprintln!("WARN: {message}");
}

pub fn err(command: &str, message: &str, json_mode: bool, details: Value) {
    if json_mode {
        eprintln!(
            "{}",
            json!({
                "ok": false,
                "command": command,
                "error": {
                    "message": message,
                    "details": details,
                },
            })
        );
    } else {
        eprintln!("ERROR: {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_ok_shape_is_stable() {
        let payload = json!({
            "ok": true,
            "command": "spec sync",
            "message": "done",
            "details": {"file_count": 1},
        });
        assert_eq!(payload["ok"], true);
        assert_eq!(payload["command"], "spec sync");
    }
}
