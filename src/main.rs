mod app;
mod compression;
mod embedder;
mod extractor;
mod prompt;

use std::process::ExitCode;

fn main() -> ExitCode {
    match app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            prompt::wait_before_exit();
            ExitCode::FAILURE
        }
    }
}
