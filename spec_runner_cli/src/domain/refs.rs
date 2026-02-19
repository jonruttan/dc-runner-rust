pub fn parse_spec_ref(spec_ref: &str) -> Result<(String, Option<String>), String> {
    let raw = spec_ref.trim();
    if raw.is_empty() {
        return Err("spec ref must not be empty".to_string());
    }
    let mut parts = raw.splitn(2, '#');
    let path = parts.next().unwrap_or("").trim().to_string();
    let frag = parts.next().map(|s| s.trim().to_string());
    if path.is_empty() {
        return Err(format!("spec ref must include path: {spec_ref}"));
    }
    if let Some(f) = &frag {
        if f.is_empty() {
            return Err(format!("spec ref has empty case id fragment: {spec_ref}"));
        }
    }
    Ok((path, frag))
}

pub fn parse_job_ref(job_ref: &str) -> Result<(Option<String>, String), String> {
    let raw = job_ref.trim();
    if raw.is_empty() {
        return Err("job ref must not be empty".to_string());
    }
    if let Some(frag) = raw.strip_prefix('#') {
        let id = frag.trim();
        if id.is_empty() {
            return Err(format!("job ref has empty case id fragment: {job_ref}"));
        }
        return Ok((None, id.to_string()));
    }
    let (path, frag) = parse_spec_ref(raw)?;
    let Some(case_id) = frag else {
        return Err(format!("job ref must include case id fragment: {job_ref}"));
    };
    Ok((Some(path), case_id))
}

pub fn extract_spec_test_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::<String>::new();
    let mut in_block = false;
    let mut cur = String::new();
    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if !in_block && trimmed == "```yaml contract-spec" {
            in_block = true;
            cur.clear();
            continue;
        }
        if in_block && trimmed == "```" {
            blocks.push(cur.clone());
            in_block = false;
            cur.clear();
            continue;
        }
        if in_block {
            cur.push_str(line);
            cur.push('\n');
        }
    }
    blocks
}

pub fn block_id(block: &str) -> Option<String> {
    for line in block.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("id:") {
            let v = rest.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spec_ref_accepts_path_and_fragment() {
        let got = parse_spec_ref("/specs/example.spec.md#CASE-1").expect("parse");
        assert_eq!(got.0, "/specs/example.spec.md");
        assert_eq!(got.1.as_deref(), Some("CASE-1"));
    }

    #[test]
    fn parse_spec_ref_accepts_path_only() {
        let got = parse_spec_ref("/specs/example.spec.md").expect("parse");
        assert_eq!(got.0, "/specs/example.spec.md");
        assert!(got.1.is_none());
    }

    #[test]
    fn parse_spec_ref_rejects_empty_fragment() {
        let err = parse_spec_ref("/specs/example.spec.md#").expect_err("expected error");
        assert!(err.contains("empty case id fragment"));
    }

    #[test]
    fn parse_spec_ref_rejects_empty_path() {
        let err = parse_spec_ref("#CASE-1").expect_err("expected error");
        assert!(err.contains("must include path"));
    }

    #[test]
    fn parse_job_ref_accepts_path_and_fragment() {
        let got =
            parse_job_ref("/specs/impl/rust/jobs/script_jobs.spec.md#DCIMPL-RUST-JOB-001")
                .expect("parse");
        assert_eq!(
            got.0.as_deref(),
            Some("/specs/impl/rust/jobs/script_jobs.spec.md")
        );
        assert_eq!(got.1, "DCIMPL-RUST-JOB-001");
    }

    #[test]
    fn parse_job_ref_accepts_same_doc_fragment() {
        let got = parse_job_ref("#DCIMPL-RUST-JOB-001").expect("parse");
        assert!(got.0.is_none());
        assert_eq!(got.1, "DCIMPL-RUST-JOB-001");
    }

    #[test]
    fn extract_spec_test_blocks_finds_tagged_yaml_blocks() {
        let md = r#"
before
```yaml contract-spec
id: CASE-1
type: contract.check
```
middle
```yaml
id: NOT-A-SPEC
```
```yaml contract-spec
id: CASE-2
```
after
"#;
        let blocks = extract_spec_test_blocks(md);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("id: CASE-1"));
        assert!(blocks[1].contains("id: CASE-2"));
    }

    #[test]
    fn block_id_extracts_id() {
        let block = "id: DCTEST-001\ncheck: runtime.foo\n";
        assert_eq!(block_id(block).as_deref(), Some("DCTEST-001"));
    }
}
