mod app;
mod cli;
mod domain;
mod governance;
mod infra;
mod job_helpers;
mod migrators;
mod ports;
mod profiler;
mod service_registry;
mod service_runtime;
mod services;
mod spec_lang;

fn main() {
    app::run();
}
