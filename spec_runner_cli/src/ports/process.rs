use std::path::Path;

pub trait ProcessRunner {
    fn run_capture(&self, program: &str, args: &[String], root: &Path) -> Result<i32, String>;
}
