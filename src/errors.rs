use slog::Logger;

pub fn log_error(logger: &Logger, e: &Error) {
    use std::fmt::Write;

    let mut message = String::new();
    let error_msg = "Error writing error to log";

    writeln!(message, "error: {}", e)
        .expect(error_msg);

    for e in e.iter().skip(1) {
        writeln!(message, "caused by: {}", e)
            .expect(error_msg);
    }

    if let Some(backtrace) = e.backtrace() {
        writeln!(message, "backtrace: {:?}", backtrace)
            .expect(error_msg);
    }

    error!(logger, "{}", message);
}

error_chain! {}
