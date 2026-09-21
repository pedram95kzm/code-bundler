mod app;
mod compression;
mod embedder;
mod extractor;
mod prompt;

use std::process::ExitCode;

fn main() -> ExitCode {
    let exit_code = match app::run() {
        Ok(()) => {
            println!("\nSuccess! The operation completed successfully.");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    };

    // Keep the result visible, including when the executable was double-clicked.
    prompt::wait_before_exit();
    exit_code
}
