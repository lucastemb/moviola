use std::process::ExitCode;

fn main() -> ExitCode {
    match moviola::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
