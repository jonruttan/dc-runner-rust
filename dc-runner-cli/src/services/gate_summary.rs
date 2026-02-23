#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateSummary {
    pub status: String,
    pub failed_step: Option<String>,
}

pub fn summarize(status: &str, failed_step: Option<&str>) -> GateSummary {
    GateSummary {
        status: status.to_string(),
        failed_step: failed_step.map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_records_status_and_failure() {
        let s = summarize("fail", Some("governance"));
        assert_eq!(s.status, "fail");
        assert_eq!(s.failed_step.as_deref(), Some("governance"));
    }
}
