use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedServiceImplementation {
    pub implementation_id: String,
    pub service_type: String,
    pub profile: String,
    pub imports: Vec<String>,
    pub packaging_mode: String,
}

fn key(service_type: &str, profile: &str) -> (String, String) {
    (service_type.to_string(), profile.to_string())
}

pub fn builtin_registry() -> HashMap<(String, String), ResolvedServiceImplementation> {
    let mut out = HashMap::new();
    out.insert(
        key("io.http", "request.http"),
        ResolvedServiceImplementation {
            implementation_id: "builtin.io.http.v1".to_string(),
            service_type: "io.http".to_string(),
            profile: "request.http".to_string(),
            imports: vec![
                "pipe_identity".to_string(),
                "assert_truth".to_string(),
                "get_json".to_string(),
                "post_json".to_string(),
            ],
            packaging_mode: "builtin".to_string(),
        },
    );
    out.insert(
        key("io.fs", "read.text"),
        ResolvedServiceImplementation {
            implementation_id: "builtin.io.fs.v1".to_string(),
            service_type: "io.fs".to_string(),
            profile: "read.text".to_string(),
            imports: vec![
                "pipe_identity".to_string(),
                "assert_truth".to_string(),
                "read_text".to_string(),
                "write_text".to_string(),
            ],
            packaging_mode: "builtin".to_string(),
        },
    );
    out.insert(
        key("io.system", "exec.command"),
        ResolvedServiceImplementation {
            implementation_id: "builtin.io.system.v1".to_string(),
            service_type: "io.system".to_string(),
            profile: "exec.command".to_string(),
            imports: vec![
                "pipe_identity".to_string(),
                "assert_truth".to_string(),
                "exec".to_string(),
                "now".to_string(),
            ],
            packaging_mode: "builtin".to_string(),
        },
    );
    out.insert(
        key("io.mysql", "mysql.query"),
        ResolvedServiceImplementation {
            implementation_id: "builtin.io.mysql.v1".to_string(),
            service_type: "io.mysql".to_string(),
            profile: "mysql.query".to_string(),
            imports: vec!["query".to_string(), "migrate".to_string()],
            packaging_mode: "builtin".to_string(),
        },
    );
    out.insert(
        key("io.docs", "generate.docs"),
        ResolvedServiceImplementation {
            implementation_id: "builtin.io.docs.v1".to_string(),
            service_type: "io.docs".to_string(),
            profile: "generate.docs".to_string(),
            imports: vec![
                "pipe_identity".to_string(),
                "assert_truth".to_string(),
                "render_docs".to_string(),
            ],
            packaging_mode: "builtin".to_string(),
        },
    );
    out
}

pub fn resolve_exact(
    service_type: &str,
    profile: &str,
) -> Result<ResolvedServiceImplementation, String> {
    let reg = builtin_registry();
    reg.get(&(service_type.to_string(), profile.to_string()))
        .cloned()
        .ok_or_else(|| {
            format!("runtime.services.unresolved_type_profile: {service_type}/{profile}")
        })
}

pub fn validate_imports_subset(
    impl_ref: &ResolvedServiceImplementation,
    declared_imports: &[String],
) -> Result<(), String> {
    for name in declared_imports {
        if !impl_ref.imports.iter().any(|allowed| allowed == name) {
            return Err(format!(
                "runtime.services.import_not_supported: {}:{}:{}",
                impl_ref.service_type, impl_ref.profile, name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_exact_known_profile() {
        let resolved = resolve_exact("io.http", "request.http").expect("expected resolve");
        assert_eq!(resolved.implementation_id, "builtin.io.http.v1");
    }

    #[test]
    fn resolve_exact_unknown_profile_fails() {
        let err = resolve_exact("io.http", "unknown.profile").expect_err("expected fail");
        assert!(err.contains("runtime.services.unresolved_type_profile"));
    }
}
