use crate::service_registry::ResolvedServiceImplementation;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub mod plugin_host;

pub const SUPPORTED_PLUGIN_API_VERSION: &str = "1";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServicePluginManifest {
    pub plugin_id: String,
    pub version: String,
    pub api_version: String,
    pub provided_services: Vec<ServiceCoverage>,
    pub binary: PluginBinary,
    pub signing: PluginSigning,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceCoverage {
    #[serde(rename = "type")]
    pub service_type: String,
    pub profiles: Vec<String>,
    pub imports_by_profile: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginBinary {
    pub path: String,
    pub r#ref: Option<String>,
    pub platform: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginSigning {
    pub publisher: String,
    pub scheme: String,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServicePluginLock {
    pub plugins: Vec<ServicePluginLockEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServicePluginLockEntry {
    pub plugin_id: String,
    pub version: String,
    pub digest: String,
    pub signature: String,
    pub source: PluginSource,
    pub services: Vec<ServiceTypeProfile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginSource {
    pub r#ref: String,
    pub kind: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServiceTypeProfile {
    #[serde(rename = "type")]
    pub service_type: String,
    pub profile: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceImplementationKind {
    Builtin,
    Plugin,
}

#[derive(Clone, Debug)]
pub struct ResolvedImplementation {
    pub kind: ServiceImplementationKind,
    pub implementation_id: String,
    pub service_type: String,
    pub profile: String,
    pub plugin_id: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub imports: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ServiceInvocationRequest {
    pub service_action_id: String,
    pub service_type: String,
    pub profile: String,
    pub import_name: String,
    pub input_payloads: HashMap<String, JsonValue>,
    pub config: JsonValue,
}

#[derive(Clone, Debug)]
pub struct ServiceInvocationResponse {
    pub output: JsonValue,
    pub diagnostics: HashMap<String, JsonValue>,
}

#[allow(dead_code)]
pub trait ServiceExecutor {
    fn invoke(
        &self,
        implementation: &ResolvedImplementation,
        request: &ServiceInvocationRequest,
    ) -> Result<ServiceInvocationResponse, String>;
}

#[derive(Default)]
pub struct BuiltinExecutor;

impl ServiceExecutor for BuiltinExecutor {
    fn invoke(
        &self,
        implementation: &ResolvedImplementation,
        request: &ServiceInvocationRequest,
    ) -> Result<ServiceInvocationResponse, String> {
        // Built-in implementation placeholder for phase-3 plumbing; concrete service
        // behavior stays in service-specific modules.
        let mut diagnostics = HashMap::new();
        diagnostics.insert(
            "implementation_id".to_string(),
            JsonValue::String(implementation.implementation_id.clone()),
        );
        diagnostics.insert(
            "service_action_id".to_string(),
            JsonValue::String(request.service_action_id.clone()),
        );
        diagnostics.insert("mode".to_string(), JsonValue::String("builtin".to_string()));
        Ok(ServiceInvocationResponse {
            output: JsonValue::Object(
                [
                    (
                        "import".to_string(),
                        JsonValue::String(request.import_name.clone()),
                    ),
                    (
                        "payloads".to_string(),
                        json_map_from_hashmap(&request.input_payloads),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            diagnostics,
        })
    }
}

pub struct PluginExecutor {
    pub plugin_bin: String,
    pub timeout_ms: u64,
}

impl ServiceExecutor for PluginExecutor {
    fn invoke(
        &self,
        implementation: &ResolvedImplementation,
        request: &ServiceInvocationRequest,
    ) -> Result<ServiceInvocationResponse, String> {
        let params = serde_json::json!({
            "service_action_id": request.service_action_id,
            "service_type": request.service_type,
            "profile": request.profile,
            "import": request.import_name,
            "input_payloads": request.input_payloads,
            "config": request.config,
        });
        let output = plugin_host::call_json_rpc_with_timeout(
            &self.plugin_bin,
            "service.invoke",
            params,
            self.timeout_ms,
        )?;
        let mut diagnostics = HashMap::new();
        diagnostics.insert("mode".to_string(), JsonValue::String("plugin".to_string()));
        diagnostics.insert(
            "service_action_id".to_string(),
            JsonValue::String(request.service_action_id.clone()),
        );
        if let Some(pid) = &implementation.plugin_id {
            diagnostics.insert("plugin_id".to_string(), JsonValue::String(pid.clone()));
        }
        if let Some(ver) = &implementation.version {
            diagnostics.insert("version".to_string(), JsonValue::String(ver.clone()));
        }
        if let Some(digest) = &implementation.digest {
            diagnostics.insert("digest".to_string(), JsonValue::String(digest.clone()));
        }
        Ok(ServiceInvocationResponse {
            output,
            diagnostics,
        })
    }
}

fn json_map_from_hashmap(input: &HashMap<String, JsonValue>) -> JsonValue {
    JsonValue::Object(
        input
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<serde_json::Map<String, JsonValue>>(),
    )
}

pub fn parse_manifest(path: &Path) -> Result<ServicePluginManifest, String> {
    let raw = fs::read_to_string(path).map_err(|e| {
        format!(
            "runtime.plugin.manifest.read_failed {}: {e}",
            path.display()
        )
    })?;
    serde_yaml::from_str::<ServicePluginManifest>(&raw).map_err(|e| {
        format!(
            "runtime.plugin.manifest.parse_failed {}: {e}",
            path.display()
        )
    })
}

pub fn parse_lock(path: &Path) -> Result<ServicePluginLock, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("runtime.plugin.lock.read_failed {}: {e}", path.display()))?;
    serde_yaml::from_str::<ServicePluginLock>(&raw)
        .map_err(|e| format!("runtime.plugin.lock.parse_failed {}: {e}", path.display()))
}

pub fn validate_manifest(manifest: &ServicePluginManifest) -> Result<(), String> {
    if manifest.plugin_id.trim().is_empty() {
        return Err("runtime.plugin.manifest.plugin_id_required".to_string());
    }
    if manifest.api_version.trim().is_empty() {
        return Err("runtime.plugin.manifest.api_version_required".to_string());
    }
    if manifest.api_version.trim() != SUPPORTED_PLUGIN_API_VERSION {
        return Err(format!(
            "runtime.plugin.manifest.api_version_unsupported: got={} expected={}",
            manifest.api_version, SUPPORTED_PLUGIN_API_VERSION
        ));
    }
    if manifest.provided_services.is_empty() {
        return Err("runtime.plugin.manifest.provided_services_required".to_string());
    }
    if manifest.signing.signature.trim().is_empty() {
        return Err("runtime.plugin.manifest.signature_required".to_string());
    }
    if manifest.binary.path.trim().is_empty() {
        return Err("runtime.plugin.manifest.binary.path_required".to_string());
    }
    Ok(())
}

pub fn validate_lock(lock: &ServicePluginLock) -> Result<(), String> {
    if lock.plugins.is_empty() {
        return Err("runtime.plugin.lock.plugins_required".to_string());
    }
    let mut seen = HashSet::new();
    for p in &lock.plugins {
        let key = format!("{}@{}", p.plugin_id, p.version);
        if !seen.insert(key.clone()) {
            return Err(format!("runtime.plugin.lock.duplicate_plugin_entry: {key}"));
        }
        if p.digest.trim().is_empty() || p.signature.trim().is_empty() {
            return Err(format!(
                "runtime.plugin.lock.digest_signature_required: {}@{}",
                p.plugin_id, p.version
            ));
        }
        if p.services.is_empty() {
            return Err(format!(
                "runtime.plugin.lock.services_required: {}@{}",
                p.plugin_id, p.version
            ));
        }
    }
    Ok(())
}

pub fn verify_local_binary_digest(
    binary_path: &Path,
    expected_sha256_hex: &str,
) -> Result<(), String> {
    let bytes = fs::read(binary_path).map_err(|e| {
        format!(
            "runtime.plugin.binary.read_failed {}: {e}",
            binary_path.display()
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let got = format!("{:x}", hasher.finalize());
    if got != expected_sha256_hex {
        return Err(format!(
            "runtime.plugin.binary.digest_mismatch {} expected={} got={}",
            binary_path.display(),
            expected_sha256_hex,
            got
        ));
    }
    Ok(())
}

pub fn find_lock_entry<'a>(
    lock: &'a ServicePluginLock,
    service_type: &str,
    profile: &str,
) -> Option<&'a ServicePluginLockEntry> {
    lock.plugins.iter().find(|p| {
        p.services
            .iter()
            .any(|s| s.service_type == service_type && s.profile == profile)
    })
}

pub fn plugin_imports_for_profile(
    manifest: &ServicePluginManifest,
    service_type: &str,
    profile: &str,
) -> Result<Vec<String>, String> {
    let Some(coverage) = manifest
        .provided_services
        .iter()
        .find(|c| c.service_type == service_type)
    else {
        return Err(format!(
            "runtime.plugin.manifest.missing_service_type: {service_type}"
        ));
    };
    coverage
        .imports_by_profile
        .get(profile)
        .cloned()
        .ok_or_else(|| {
            format!(
                "runtime.plugin.manifest.missing_profile_imports: {}/{}",
                service_type, profile
            )
        })
}

pub fn resolve_implementation_lock_opt_in(
    service_type: &str,
    profile: &str,
    builtin: &ResolvedServiceImplementation,
    lock: Option<&ServicePluginLock>,
    manifest: Option<&ServicePluginManifest>,
) -> Result<ResolvedImplementation, String> {
    let Some(lock_ref) = lock else {
        return Ok(ResolvedImplementation {
            kind: ServiceImplementationKind::Builtin,
            implementation_id: builtin.implementation_id.clone(),
            service_type: service_type.to_string(),
            profile: profile.to_string(),
            plugin_id: None,
            version: None,
            digest: None,
            imports: builtin.imports.clone(),
        });
    };

    let Some(entry) = find_lock_entry(lock_ref, service_type, profile) else {
        return Ok(ResolvedImplementation {
            kind: ServiceImplementationKind::Builtin,
            implementation_id: builtin.implementation_id.clone(),
            service_type: service_type.to_string(),
            profile: profile.to_string(),
            plugin_id: None,
            version: None,
            digest: None,
            imports: builtin.imports.clone(),
        });
    };

    let manifest = manifest.ok_or_else(|| {
        format!(
            "runtime.plugin.manifest.required_for_locked_service: {}/{}",
            service_type, profile
        )
    })?;
    if manifest.plugin_id != entry.plugin_id || manifest.version != entry.version {
        return Err(format!(
            "runtime.plugin.manifest_lock_mismatch: manifest={}@{} lock={}@{}",
            manifest.plugin_id, manifest.version, entry.plugin_id, entry.version
        ));
    }
    if manifest.signing.signature.trim().is_empty() || entry.signature.trim().is_empty() {
        return Err("runtime.plugin.signature_required".to_string());
    }
    let plugin_imports = plugin_imports_for_profile(manifest, service_type, profile)?;
    Ok(ResolvedImplementation {
        kind: ServiceImplementationKind::Plugin,
        implementation_id: format!("plugin.{}@{}", entry.plugin_id, entry.version),
        service_type: service_type.to_string(),
        profile: profile.to_string(),
        plugin_id: Some(entry.plugin_id.clone()),
        version: Some(entry.version.clone()),
        digest: Some(entry.digest.clone()),
        imports: plugin_imports,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_registry::resolve_exact;

    #[test]
    fn resolve_lock_opt_in_uses_builtin_without_lock() {
        let builtin = resolve_exact("io.http", "request.http").expect("builtin");
        let resolved =
            resolve_implementation_lock_opt_in("io.http", "request.http", &builtin, None, None)
                .expect("resolved");
        assert_eq!(resolved.kind, ServiceImplementationKind::Builtin);
        assert_eq!(resolved.implementation_id, "builtin.io.http.v1");
    }

    #[test]
    fn resolve_lock_opt_in_uses_builtin_when_lock_has_no_match() {
        let builtin = resolve_exact("io.http", "request.http").expect("builtin");
        let lock = ServicePluginLock {
            plugins: vec![ServicePluginLockEntry {
                plugin_id: "plugin-a".to_string(),
                version: "1.0.0".to_string(),
                digest: "abc".to_string(),
                signature: "sig".to_string(),
                source: PluginSource {
                    r#ref: "https://example.invalid/plugin".to_string(),
                    kind: Some("url".to_string()),
                },
                services: vec![ServiceTypeProfile {
                    service_type: "io.fs".to_string(),
                    profile: "read.text".to_string(),
                }],
            }],
        };
        let resolved = resolve_implementation_lock_opt_in(
            "io.http",
            "request.http",
            &builtin,
            Some(&lock),
            None,
        )
        .expect("resolved");
        assert_eq!(resolved.kind, ServiceImplementationKind::Builtin);
    }

    #[test]
    fn resolve_lock_opt_in_requires_manifest_for_locked_plugin() {
        let builtin = resolve_exact("io.http", "request.http").expect("builtin");
        let lock = ServicePluginLock {
            plugins: vec![ServicePluginLockEntry {
                plugin_id: "plugin-a".to_string(),
                version: "1.0.0".to_string(),
                digest: "abc".to_string(),
                signature: "sig".to_string(),
                source: PluginSource {
                    r#ref: "https://example.invalid/plugin".to_string(),
                    kind: Some("url".to_string()),
                },
                services: vec![ServiceTypeProfile {
                    service_type: "io.http".to_string(),
                    profile: "request.http".to_string(),
                }],
            }],
        };
        let err = resolve_implementation_lock_opt_in(
            "io.http",
            "request.http",
            &builtin,
            Some(&lock),
            None,
        )
        .expect_err("expected error");
        assert!(err.contains("manifest.required_for_locked_service"));
    }

    #[test]
    fn resolve_lock_opt_in_selects_plugin_on_match() {
        let builtin = resolve_exact("io.http", "request.http").expect("builtin");
        let lock = ServicePluginLock {
            plugins: vec![ServicePluginLockEntry {
                plugin_id: "plugin-a".to_string(),
                version: "1.0.0".to_string(),
                digest: "abc".to_string(),
                signature: "sig".to_string(),
                source: PluginSource {
                    r#ref: "https://example.invalid/plugin".to_string(),
                    kind: Some("url".to_string()),
                },
                services: vec![ServiceTypeProfile {
                    service_type: "io.http".to_string(),
                    profile: "request.http".to_string(),
                }],
            }],
        };
        let manifest = ServicePluginManifest {
            plugin_id: "plugin-a".to_string(),
            version: "1.0.0".to_string(),
            api_version: "1".to_string(),
            provided_services: vec![ServiceCoverage {
                service_type: "io.http".to_string(),
                profiles: vec!["request.http".to_string()],
                imports_by_profile: [(
                    "request.http".to_string(),
                    vec!["pipe_identity".to_string(), "get_json".to_string()],
                )]
                .into_iter()
                .collect(),
            }],
            binary: PluginBinary {
                path: "./plugin-http".to_string(),
                r#ref: None,
                platform: None,
            },
            signing: PluginSigning {
                publisher: "example".to_string(),
                scheme: "sigstore".to_string(),
                signature: "sig".to_string(),
            },
        };

        let resolved = resolve_implementation_lock_opt_in(
            "io.http",
            "request.http",
            &builtin,
            Some(&lock),
            Some(&manifest),
        )
        .expect("resolved");
        assert_eq!(resolved.kind, ServiceImplementationKind::Plugin);
        assert_eq!(resolved.plugin_id.as_deref(), Some("plugin-a"));
        assert!(resolved.imports.iter().any(|i| i == "get_json"));
    }
}
