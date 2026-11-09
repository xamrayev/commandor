//! Worker jarayoni tarafi: stdin’dan so‘rov o‘qiydi, stdout’ga javob yozadi.

use crate::frame::{FrameError, read_frame, write_frame};
use crate::protocol::{
    DIAG_ENV, PROTOCOL_VERSION, WorkerKind, WorkerMessage, WorkerRequest, WorkerResponse,
};
use std::io::{Read, Write};
use std::time::Duration;

/// Worker siklini standart oqimlarda ishga tushiradi. Supervisor stdin’ni yopsa normal tugaydi.
pub fn run_stdio(kind: WorkerKind) -> Result<(), FrameError> {
    let diag = std::env::var(DIAG_ENV).is_ok_and(|v| v == "1");
    run(
        kind,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        diag,
    )
}

pub fn run<R: Read, W: Write>(
    kind: WorkerKind,
    input: &mut R,
    output: &mut W,
    diag: bool,
) -> Result<(), FrameError> {
    write_frame(
        output,
        &WorkerMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            kind,
        },
    )?;
    loop {
        let message: WorkerMessage = match read_frame(input) {
            Ok(m) => m,
            Err(FrameError::Closed) => return Ok(()),
            Err(e) => return Err(e),
        };
        let WorkerMessage::Request { id, request } = message else {
            continue; // Workerga faqat so‘rov keladi; boshqasi e’tiborsiz.
        };
        let response = match request {
            WorkerRequest::Ping => WorkerResponse::Pong,
            WorkerRequest::Echo(text) => WorkerResponse::Echo(text),
            WorkerRequest::Sleep { .. } | WorkerRequest::Abort if !diag => {
                WorkerResponse::Rejected("diagnostika o‘chiq".into())
            }
            WorkerRequest::Sleep { millis } => {
                std::thread::sleep(Duration::from_millis(millis));
                WorkerResponse::Slept
            }
            WorkerRequest::Abort => std::process::abort(),
        };
        write_frame(output, &WorkerMessage::Response { id, response })?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn in_memory_session_greets_and_answers() {
        let mut input = Vec::new();
        write_frame(
            &mut input,
            &WorkerMessage::Request {
                id: 7,
                request: WorkerRequest::Ping,
            },
        )
        .expect("write");
        write_frame(
            &mut input,
            &WorkerMessage::Request {
                id: 8,
                request: WorkerRequest::Abort,
            },
        )
        .expect("write");
        let mut output = Vec::new();
        run(
            WorkerKind::Diag,
            &mut Cursor::new(input),
            &mut output,
            false,
        )
        .expect("run");

        let mut out = Cursor::new(output);
        let hello: WorkerMessage = read_frame(&mut out).expect("hello");
        assert_eq!(
            hello,
            WorkerMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                kind: WorkerKind::Diag
            }
        );
        let pong: WorkerMessage = read_frame(&mut out).expect("pong");
        assert_eq!(
            pong,
            WorkerMessage::Response {
                id: 7,
                response: WorkerResponse::Pong
            }
        );
        let rejected: WorkerMessage = read_frame(&mut out).expect("rejected");
        assert!(matches!(
            rejected,
            WorkerMessage::Response {
                id: 8,
                response: WorkerResponse::Rejected(_)
            }
        ));
    }
}
