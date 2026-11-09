use commandor_core::{Command, DirectoryTarget, ErrorCode, Event, ListOptions, Response};
use std::time::Duration;

#[test]
fn filesystem_requests_roundtrip_and_preserve_request_ids() {
    let fixture = tempfile::tempdir().expect("temp");
    std::fs::write(fixture.path().join("hisobot.txt"), "fixture").expect("write");
    let (handle, events, worker) =
        commandor_broker::start_at(|| {}, fixture.path().into()).expect("broker");
    let request = handle
        .dispatch(Command::ReadDirectory {
            target: DirectoryTarget::Home,
            options: ListOptions::default(),
        })
        .expect("dispatch");
    let Some(Event::Reply {
        request: reply_id,
        reply: Ok(Response::Directory(listing)),
    }) = events.recv_timeout(Duration::from_secs(5))
    else {
        panic!("listing reply");
    };
    assert_eq!(request, reply_id);
    assert_eq!(listing.entries[0].name, "hisobot.txt");
    let bad = handle
        .dispatch(Command::ReadDirectory {
            target: DirectoryTarget::Path("relative".into()),
            options: ListOptions::default(),
        })
        .expect("dispatch");
    let Some(Event::Reply {
        request,
        reply: Err(error),
    }) = events.recv_timeout(Duration::from_secs(5))
    else {
        panic!("error reply");
    };
    assert_eq!(request, bad);
    assert_eq!(error.code, ErrorCode::InvalidArgument);
    let ping = handle.dispatch(Command::Ping).expect("ping");
    assert_eq!(
        events.recv_timeout(Duration::from_secs(5)),
        Some(Event::Reply {
            request: ping,
            reply: Ok(Response::Pong)
        })
    );
    drop(handle);
    worker.join();
}
