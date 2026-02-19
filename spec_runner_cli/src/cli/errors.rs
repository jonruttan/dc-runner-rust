#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CliError {
    Usage(String),
    Runtime(String),
    Internal(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Runtime(_) | CliError::Internal(_) => 1,
        }
    }

    #[allow(dead_code)]
    pub fn message(&self) -> &str {
        match self {
            CliError::Usage(m) | CliError::Runtime(m) | CliError::Internal(m) => m,
        }
    }
}
