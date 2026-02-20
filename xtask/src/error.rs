use anyhow::Error as AnyError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Runtime(AnyError),
}

impl AppError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Runtime(_) => 1,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Usage(message) => message.clone(),
            Self::Runtime(error) => format!("{error:#}"),
        }
    }
}

impl From<AnyError> for AppError {
    fn from(value: AnyError) -> Self {
        Self::Runtime(value)
    }
}
