//! Reja → bajarish oqimi sintetik temp papkalarda. Savat soxta: fayllar temp ichidagi `bin/`ga ko‘chadi.

use commandor_core::{
    AppError, DirectoryListing, DirectoryTarget, EntryId, ErrorCode, ListOptions, PlanAction,
    PlanRequest,
};
use commandor_fs::{FileService, Trasher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Savat o‘rniga: yo‘lni `bin/` ga ko‘chiradi va chaqiruvlarni yozadi.
struct FakeTrash {
    bin: PathBuf,
    log: Arc<Mutex<Vec<String>>>,
    fail_on: Option<String>,
}

impl Trasher for FakeTrash {
    fn trash(&self, path: &Path) -> Result<(), AppError> {
        let name = path
            .file_name()
            .expect("name")
            .to_string_lossy()
            .into_owned();
        if self.fail_on.as_deref() == Some(name.as_str()) {
            return Err(ErrorCode::TrashUnavailable.into());
        }
        fs::rename(path, self.bin.join(&name)).map_err(|_| ErrorCode::Internal)?;
        self.log.lock().expect("lock").push(name);
        Ok(())
    }
}

struct Fixture {
    root: TempDir,
    service: FileService,
    log: Arc<Mutex<Vec<String>>>,
}

fn fixture(fail_on: Option<&str>) -> Fixture {
    let root = tempfile::tempdir().expect("temp");
    let work = root.path().join("ish");
    fs::create_dir_all(work.join("papka")).expect("mkdir");
    fs::write(work.join("hisobot.txt"), "salom").expect("write");
    fs::write(work.join("rasm.png"), "png").expect("write");
    let bin = root.path().join("bin");
    fs::create_dir(&bin).expect("bin");
    let log = Arc::new(Mutex::new(Vec::new()));
    let trash = FakeTrash {
        bin,
        log: Arc::clone(&log),
        fail_on: fail_on.map(str::to_owned),
    };
    let service = FileService::with_trasher(work, Box::new(trash));
    Fixture { root, service, log }
}

fn list(service: &mut FileService) -> DirectoryListing {
    service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("list")
}

fn id(listing: &DirectoryListing, name: &str) -> EntryId {
    listing
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.id)
        .expect("entry")
}

#[test]
fn create_directory_plan_then_execute_once() {
    let mut f = fixture(None);
    let listing = list(&mut f.service);
    let plan = f
        .service
        .create_plan(PlanRequest::CreateDirectory {
            location: listing.location.id,
            name: "Yangi papka".into(),
        })
        .expect("plan");
    assert_eq!(plan.action, PlanAction::CreateDirectory);
    assert!(!plan.requires_approval);
    let target = f.root.path().join("ish/Yangi papka");
    assert!(!target.exists(), "reja diskni o‘zgartirmaydi");

    let report = f.service.execute_plan(plan.id, plan.hash).expect("execute");
    assert_eq!(report.done, ["Yangi papka"]);
    assert!(target.is_dir());
    let again = f.service.execute_plan(plan.id, plan.hash);
    assert_eq!(
        again.expect_err("once").code,
        ErrorCode::PlanExpired,
        "reja bir marta"
    );

    let exists = f.service.create_plan(PlanRequest::CreateDirectory {
        location: listing.location.id,
        name: "papka".into(),
    });
    assert_eq!(
        exists.expect_err("exists").code,
        ErrorCode::DestinationExists
    );
    let bad = f.service.create_plan(PlanRequest::CreateDirectory {
        location: listing.location.id,
        name: "../qochish".into(),
    });
    assert_eq!(bad.expect_err("bad").code, ErrorCode::InvalidName);
}

#[test]
fn wrong_hash_and_folder_appearing_meanwhile_are_rejected() {
    let mut f = fixture(None);
    let listing = list(&mut f.service);
    let plan = f
        .service
        .create_plan(PlanRequest::CreateDirectory {
            location: listing.location.id,
            name: "keyin".into(),
        })
        .expect("plan");
    let wrong = f.service.execute_plan(plan.id, plan.hash ^ 1);
    assert_eq!(wrong.expect_err("hash").code, ErrorCode::PlanExpired);

    let plan = f
        .service
        .create_plan(PlanRequest::CreateDirectory {
            location: listing.location.id,
            name: "keyin".into(),
        })
        .expect("plan");
    fs::write(f.root.path().join("ish/keyin"), "boshqa jarayon").expect("race");
    let raced = f.service.execute_plan(plan.id, plan.hash);
    assert_eq!(
        raced.expect_err("exists").code,
        ErrorCode::DestinationExists
    );
    assert_eq!(
        fs::read_to_string(f.root.path().join("ish/keyin")).expect("read"),
        "boshqa jarayon",
        "mavjud fayl tegilmaydi"
    );
}

#[test]
fn rename_checks_target_and_source_identity() {
    let mut f = fixture(None);
    let listing = list(&mut f.service);
    // Har bir `EntryId` faqat o‘z listingining `location`i bilan amal qiladi.
    let rename =
        |service: &mut FileService, listing: &DirectoryListing, name: &str, new_name: &str| {
            service.create_plan(PlanRequest::Rename {
                location: listing.location.id,
                entry: id(listing, name),
                new_name: new_name.into(),
            })
        };
    let taken = rename(&mut f.service, &listing, "hisobot.txt", "rasm.png");
    assert_eq!(taken.expect_err("taken").code, ErrorCode::DestinationExists);
    let same = rename(&mut f.service, &listing, "hisobot.txt", "hisobot.txt");
    assert_eq!(same.expect_err("same").code, ErrorCode::InvalidName);

    let plan = rename(&mut f.service, &listing, "hisobot.txt", "Hisobot 2026.txt").expect("plan");
    assert_eq!(plan.items[0].new_name.as_deref(), Some("Hisobot 2026.txt"));
    let report = f.service.execute_plan(plan.id, plan.hash).expect("execute");
    assert_eq!(report.done, ["Hisobot 2026.txt"]);
    let work = f.root.path().join("ish");
    assert_eq!(
        fs::read_to_string(work.join("Hisobot 2026.txt")).expect("read"),
        "salom"
    );
    assert!(!work.join("hisobot.txt").exists());

    // Manba reja va bajarish orasida almashtirilsa — rad etiladi, yangi fayl tegilmaydi.
    let listing = list(&mut f.service);
    let plan = rename(&mut f.service, &listing, "rasm.png", "surat.png").expect("plan");
    fs::remove_file(work.join("rasm.png")).expect("rm");
    fs::write(work.join("rasm.png"), "boshqa fayl").expect("replace");
    let stale = f.service.execute_plan(plan.id, plan.hash);
    if cfg!(unix) {
        assert_eq!(stale.expect_err("stale").code, ErrorCode::StaleReference);
        assert!(!work.join("surat.png").exists());
    }
}

#[test]
fn case_only_rename_is_allowed() {
    let mut f = fixture(None);
    let listing = list(&mut f.service);
    let plan = f
        .service
        .create_plan(PlanRequest::Rename {
            location: listing.location.id,
            entry: id(&listing, "rasm.png"),
            new_name: "RASM.png".into(),
        })
        .expect("case-only reja (registrga befarq FSda ham)");
    f.service.execute_plan(plan.id, plan.hash).expect("execute");
    let names: Vec<_> = list(&mut f.service)
        .entries
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert!(names.contains(&"RASM.png".to_owned()), "{names:?}");
    assert!(!names.contains(&"rasm.png".to_owned()));
}

#[test]
fn trash_needs_approval_reports_each_item_and_never_deletes_on_failure() {
    let mut f = fixture(Some("rasm.png"));
    let listing = list(&mut f.service);
    let entries = vec![
        id(&listing, "hisobot.txt"),
        id(&listing, "rasm.png"),
        id(&listing, "papka"),
        id(&listing, "papka"),
    ];
    let plan = f
        .service
        .create_plan(PlanRequest::Trash {
            location: listing.location.id,
            entries,
        })
        .expect("plan");
    assert!(plan.requires_approval);
    assert_eq!(plan.items.len(), 3, "takror ID bir marta");
    assert_eq!(plan.items[0].size_bytes, Some(5));
    assert!(
        f.log.lock().expect("lock").is_empty(),
        "reja hech narsani savatga tashlamaydi"
    );

    let report = f.service.execute_plan(plan.id, plan.hash).expect("execute");
    assert_eq!(report.done, ["hisobot.txt", "papka"]);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].name, "rasm.png");
    assert_eq!(report.failed[0].error.code, ErrorCode::TrashUnavailable);
    let work = f.root.path().join("ish");
    assert!(
        work.join("rasm.png").exists(),
        "savat ishlamasa fayl joyida qoladi"
    );
    assert!(f.root.path().join("bin/hisobot.txt").exists());

    let empty = f.service.create_plan(PlanRequest::Trash {
        location: listing.location.id,
        entries: vec![],
    });
    assert_eq!(empty.expect_err("empty").code, ErrorCode::InvalidArgument);
}

#[test]
fn discard_and_stale_grant() {
    let mut f = fixture(None);
    let listing = list(&mut f.service);
    let plan = f
        .service
        .create_plan(PlanRequest::CreateDirectory {
            location: listing.location.id,
            name: "x".into(),
        })
        .expect("plan");
    f.service.discard_plan(plan.id);
    let gone = f.service.execute_plan(plan.id, plan.hash);
    assert_eq!(gone.expect_err("discarded").code, ErrorCode::PlanExpired);
    let stale = f.service.create_plan(PlanRequest::Trash {
        location: commandor_core::LocationId::from_raw(424_242),
        entries: vec![id(&listing, "papka")],
    });
    assert_eq!(stale.expect_err("grant").code, ErrorCode::StaleReference);
}
