use std::path::Path;

#[allow(dead_code)]
pub trait ProcessRunner {
    fn run_capture(&self, program: &str, args: &[String], root: &Path) -> Result<i32, String>;
}
