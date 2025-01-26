//! Logging utils.
//!
//! This tool runs in raw mode, so we can't log to STDOUT or STDERR naively,
//! instead listen to the file in another terminal using `tail -f debug.fifo`
//! and call `log` or `debug` to send the messages.

use std::{fmt, io::Write, path::Path};

use fs_err as fs;

const LOG_FILE_PATH: &str = "debug.fifo";

#[allow(unused)]
#[track_caller]
pub fn log(message: impl fmt::Display) {
    writeln!(open_logging_fifo(), "{message}");
}

#[allow(unused)]
#[track_caller]
pub fn debug(message: impl fmt::Debug) {
    writeln!(open_logging_fifo(), "{message:#?}");
}

#[track_caller]
fn open_logging_fifo() -> fs::File {
    if !Path::new(LOG_FILE_PATH)
        .try_exists()
        .expect("failed to check if logging fifo exists")
    {
        unix_named_pipe::create(LOG_FILE_PATH, None).expect("failed to create logging fifo");
    }

    fs::OpenOptions::new()
        .write(true)
        .open(LOG_FILE_PATH)
        .expect("failed to open logging fifo")
}
