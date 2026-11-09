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

/// Haqiqiy dastur ochilmaydi: faqat rad etish, tasdiq so‘rash va ro‘yxat yo‘llari sinaladi.
#[test]
fn open_requests_are_validated_through_the_listing_grant() {
    use commandor_core::{EntryId, LocationId, OpenOutcome, OpenWith};
    let fixture = tempfile::tempdir().expect("temp");
    std::fs::create_dir(fixture.path().join("papka")).expect("mkdir");
    std::fs::write(fixture.path().join("o‘rnatish.sh"), "#!/bin/sh\n").expect("write");
    std::fs::write(fixture.path().join("rasm.png"), "png").expect("write");
    let (handle, events, worker) =
        commandor_broker::start_at(|| {}, fixture.path().into()).expect("broker");
    let wait = |request| {
        let Some(Event::Reply { request: id, reply }) =
            events.recv_timeout(Duration::from_secs(20))
        else {
            panic!("javob kelmadi");
        };
        assert_eq!(id, request);
        reply
    };
    let listing = handle
        .dispatch(Command::ReadDirectory {
            target: DirectoryTarget::Home,
            options: ListOptions::default(),
        })
        .expect("dispatch");
    let Ok(Response::Directory(listing)) = wait(listing) else {
        panic!("listing");
    };
    let id = |name: &str| {
        listing
            .entries
            .iter()
            .find(|entry| entry.name == name)
            .expect("entry")
            .id
    };
    let open = |entry: EntryId, location: LocationId| Command::OpenEntry {
        location,
        entry,
        with: OpenWith::Default,
        allow_executable: false,
    };
    let location = listing.location.id;

    let dir = handle
        .dispatch(open(id("papka"), location))
        .expect("dispatch");
    assert_eq!(wait(dir).expect_err("dir").code, ErrorCode::InvalidArgument);

    let stale = handle
        .dispatch(open(id("rasm.png"), LocationId::from_raw(777)))
        .expect("dispatch");
    assert_eq!(
        wait(stale).expect_err("stale").code,
        ErrorCode::StaleReference
    );

    let script = handle
        .dispatch(open(id("o‘rnatish.sh"), location))
        .expect("dispatch");
    assert_eq!(
        wait(script),
        Ok(Response::Opened(OpenOutcome::NeedsConfirmation))
    );

    let list = handle
        .dispatch(Command::ListOpenWith {
            location,
            entry: id("rasm.png"),
        })
        .expect("dispatch");
    let Ok(Response::OpenWithApps(apps)) = wait(list) else {
        panic!("open with ro‘yxati");
    };
    assert_eq!(apps.system_chooser, cfg!(windows));

    // Launcher ishlaridan keyin ham broker oddiy buyruqlarga javob beradi.
    let ping = handle.dispatch(Command::Ping).expect("ping");
    assert_eq!(wait(ping), Ok(Response::Pong));
    drop(handle);
    worker.join();
}

/// Reja → bajarish broker orqali (yangi papka; savat brokerda real, shuning uchun bu yerda sinalmaydi).
#[test]
fn plans_roundtrip_through_the_file_thread() {
    use commandor_core::{PlanAction, PlanRequest};
    let fixture = tempfile::tempdir().expect("temp");
    let (handle, events, worker) =
        commandor_broker::start_at(|| {}, fixture.path().into()).expect("broker");
    let wait = |request| {
        let Some(Event::Reply { request: id, reply }) = events.recv_timeout(Duration::from_secs(5))
        else {
            panic!("javob kelmadi");
        };
        assert_eq!(id, request);
        reply
    };
    let listing = handle
        .dispatch(Command::ReadDirectory {
            target: DirectoryTarget::Home,
            options: ListOptions::default(),
        })
        .expect("dispatch");
    let Ok(Response::Directory(listing)) = wait(listing) else {
        panic!("listing");
    };
    let request = handle
        .dispatch(Command::CreatePlan(PlanRequest::CreateDirectory {
            location: listing.location.id,
            name: "Yangi".into(),
        }))
        .expect("dispatch");
    let Ok(Response::Plan(plan)) = wait(request) else {
        panic!("plan");
    };
    assert_eq!(plan.action, PlanAction::CreateDirectory);
    assert!(!fixture.path().join("Yangi").exists());
    let request = handle
        .dispatch(Command::ExecutePlan {
            plan: plan.id,
            hash: plan.hash,
        })
        .expect("dispatch");
    let Ok(Response::Executed(report)) = wait(request) else {
        panic!("executed");
    };
    assert_eq!(report.done, ["Yangi"]);
    assert!(fixture.path().join("Yangi").is_dir());
    let request = handle
        .dispatch(Command::DiscardPlan { plan: plan.id })
        .expect("dispatch");
    assert_eq!(wait(request), Ok(Response::Discarded));
    drop(handle);
    worker.join();
}

/// Nusxalash fon vazifasi: JobStarted → (JobProgress) → JobFinished; noma’lum vazifani bekor qilish.
#[test]
fn copy_runs_as_a_background_job() {
    use commandor_core::{ConflictPolicy, JobId, PlanRequest};
    let fixture = tempfile::tempdir().expect("temp");
    let src = fixture.path().join("a");
    let dst = fixture.path().join("b");
    std::fs::create_dir_all(src.join("ichki")).expect("mkdir");
    std::fs::create_dir(&dst).expect("mkdir");
    std::fs::write(src.join("ichki/f.bin"), vec![3_u8; 2 << 20]).expect("write");
    let (handle, events, worker) =
        commandor_broker::start_at(|| {}, fixture.path().into()).expect("broker");
    let reply = |request| loop {
        match events.recv_timeout(Duration::from_secs(10)) {
            Some(Event::Reply { request: id, reply }) if id == request => break reply,
            Some(_) => {}
            None => panic!("javob kelmadi"),
        }
    };
    let list = |path: &std::path::Path| {
        let request = handle
            .dispatch(Command::ReadDirectory {
                target: DirectoryTarget::Path(path.to_string_lossy().into_owned()),
                options: ListOptions::default(),
            })
            .expect("dispatch");
        let Ok(Response::Directory(listing)) = reply(request) else {
            panic!("listing");
        };
        listing
    };
    let source = list(&src);
    let target = list(&dst);
    let request = handle
        .dispatch(Command::CreatePlan(PlanRequest::Copy {
            location: source.location.id,
            entries: vec![source.entries[0].id],
            target: target.location.id,
            policy: ConflictPolicy::Ask,
        }))
        .expect("dispatch");
    let Ok(Response::Plan(plan)) = reply(request) else {
        panic!("plan");
    };
    let request = handle
        .dispatch(Command::ExecutePlan {
            plan: plan.id,
            hash: plan.hash,
        })
        .expect("dispatch");
    let Ok(Response::JobStarted(job)) = reply(request) else {
        panic!("job");
    };
    let report = loop {
        match events.recv_timeout(Duration::from_secs(20)) {
            Some(Event::JobFinished { job: id, result }) if id == job => break result,
            Some(Event::JobProgress(progress)) => assert_eq!(progress.job, job),
            Some(_) => {}
            None => panic!("vazifa tugamadi"),
        }
    }
    .expect("report");
    assert_eq!(report.done, ["ichki"]);
    assert!(dst.join("ichki/f.bin").is_file());

    let request = handle
        .dispatch(Command::CancelJob { job: JobId::new() })
        .expect("dispatch");
    assert_eq!(
        reply(request).expect_err("unknown").code,
        ErrorCode::NotFound
    );
    drop(handle);
    worker.join();
}
