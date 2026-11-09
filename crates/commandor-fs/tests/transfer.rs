//! Nusxalash/ko‘chirish sintetik temp papkalarda.

use commandor_core::{
    ConflictPolicy, DirectoryListing, DirectoryTarget, EntryId, ErrorCode, ListOptions,
    OperationPlan, OperationReport, PlanAction, PlanRequest,
};
use commandor_fs::{FileService, Progress};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::TempDir;

struct Fixture {
    _root: TempDir,
    src: PathBuf,
    dst: PathBuf,
    service: FileService,
}

fn tree(src: &Path) {
    fs::create_dir_all(src.join("papka/ichki")).expect("mkdir");
    fs::write(src.join("hisobot.txt"), "salom dunyo").expect("w");
    fs::write(src.join("papka/a.md"), "# a").expect("w");
    fs::write(src.join("papka/ichki/b.bin"), vec![7_u8; 4096]).expect("w");
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("temp");
    let src = root.path().join("manba");
    let dst = root.path().join("maqsad");
    tree(&src);
    fs::create_dir(&dst).expect("mkdir");
    let service = FileService::new(root.path().into());
    Fixture {
        _root: root,
        src,
        dst,
        service,
    }
}

fn list(service: &mut FileService, path: &Path) -> DirectoryListing {
    service
        .read(
            DirectoryTarget::Path(path.to_string_lossy().into_owned()),
            ListOptions::default(),
        )
        .expect("list")
}

fn ids(listing: &DirectoryListing, names: &[&str]) -> Vec<EntryId> {
    names
        .iter()
        .map(|name| {
            listing
                .entries
                .iter()
                .find(|entry| entry.name == *name)
                .expect("entry")
                .id
        })
        .collect()
}

fn plan(
    f: &mut Fixture,
    action: PlanAction,
    names: &[&str],
    policy: ConflictPolicy,
) -> Result<OperationPlan, commandor_core::AppError> {
    let (src, dst) = (f.src.clone(), f.dst.clone());
    let source = list(&mut f.service, &src);
    let target = list(&mut f.service, &dst);
    let entries = ids(&source, names);
    let (location, target) = (source.location.id, target.location.id);
    let request = match action {
        PlanAction::Copy => PlanRequest::Copy {
            location,
            entries,
            target,
            policy,
        },
        _ => PlanRequest::Move {
            location,
            entries,
            target,
            policy,
        },
    };
    f.service.create_plan(request)
}

fn run(f: &mut Fixture, plan: &OperationPlan) -> (OperationReport, Vec<Progress>) {
    let job = f.service.start_transfer(plan.id, plan.hash).expect("start");
    let cancel = AtomicBool::new(false);
    let mut seen = Vec::new();
    let report = job.run(&cancel, &mut |progress| seen.push(progress.clone()));
    (report, seen)
}

#[test]
fn copy_tree_with_progress_matching_totals() {
    let mut f = fixture();
    let plan = plan(
        &mut f,
        PlanAction::Copy,
        &["hisobot.txt", "papka"],
        ConflictPolicy::Ask,
    )
    .expect("plan");
    assert!(plan.requires_approval);
    let canonical = fs::canonicalize(&f.dst).expect("canonical");
    assert_eq!(
        plan.target.as_deref(),
        Some(canonical.to_string_lossy().as_ref())
    );
    let totals = plan.totals.expect("totals");
    assert_eq!(totals.bytes, 11 + 3 + 4096);
    assert_eq!(totals.items, 5, "fayl + papka + 2 fayl + ichki papka");
    assert_eq!(
        plan.items[1].size_bytes,
        Some(4099),
        "papka hajmi daraxt bo‘yicha"
    );
    assert!(
        !f.dst.join("hisobot.txt").exists(),
        "reja diskni o‘zgartirmaydi"
    );

    let (report, progress) = run(&mut f, &plan);
    assert_eq!(report.done, ["hisobot.txt", "papka"]);
    assert!(report.failed.is_empty() && !report.cancelled);
    let last = progress.last().expect("progress");
    assert_eq!(
        (last.done_items, last.done_bytes),
        (totals.items, totals.bytes)
    );
    assert_eq!(
        fs::read(f.dst.join("papka/ichki/b.bin")).expect("r"),
        vec![7_u8; 4096]
    );
    assert!(
        f.src.join("papka/a.md").exists(),
        "nusxalash manbani saqlaydi"
    );
    let leftovers: Vec<_> = fs::read_dir(&f.dst)
        .expect("dir")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("commandor-part"))
        .collect();
    assert!(leftovers.is_empty(), "vaqtinchalik fayl qolmaydi");
}

#[test]
fn conflicts_require_a_decision_and_each_policy_behaves() {
    let mut f = fixture();
    fs::write(f.dst.join("hisobot.txt"), "eski").expect("w");
    fs::create_dir(f.dst.join("papka")).expect("mkdir");
    fs::write(f.dst.join("papka/eski.txt"), "qoladi").expect("w");

    let ask = plan(
        &mut f,
        PlanAction::Copy,
        &["hisobot.txt", "papka"],
        ConflictPolicy::Ask,
    )
    .expect("plan");
    assert!(ask.items.iter().all(|item| item.conflict));
    let refused = f.service.start_transfer(ask.id, ask.hash);
    assert_eq!(refused.expect_err("ask").code, ErrorCode::Conflict);

    let skip = plan(
        &mut f,
        PlanAction::Copy,
        &["hisobot.txt"],
        ConflictPolicy::Skip,
    )
    .expect("plan");
    let (report, _) = run(&mut f, &skip);
    assert_eq!(report.skipped, ["hisobot.txt"]);
    assert_eq!(
        fs::read_to_string(f.dst.join("hisobot.txt")).expect("r"),
        "eski"
    );

    let both = plan(
        &mut f,
        PlanAction::Copy,
        &["hisobot.txt"],
        ConflictPolicy::KeepBoth,
    )
    .expect("plan");
    let (report, _) = run(&mut f, &both);
    assert_eq!(report.done, ["hisobot (2).txt"]);
    assert_eq!(
        fs::read_to_string(f.dst.join("hisobot.txt")).expect("r"),
        "eski"
    );
    assert_eq!(
        fs::read_to_string(f.dst.join("hisobot (2).txt")).expect("r"),
        "salom dunyo"
    );

    let replace = plan(
        &mut f,
        PlanAction::Copy,
        &["hisobot.txt", "papka"],
        ConflictPolicy::Replace,
    )
    .expect("plan");
    let (report, _) = run(&mut f, &replace);
    assert!(report.failed.is_empty(), "{:?}", report.failed);
    assert_eq!(
        fs::read_to_string(f.dst.join("hisobot.txt")).expect("r"),
        "salom dunyo"
    );
    assert_eq!(
        fs::read_to_string(f.dst.join("papka/eski.txt")).expect("r"),
        "qoladi",
        "papka almashtirilmaydi, birlashtiriladi"
    );
    assert!(f.dst.join("papka/ichki/b.bin").exists());
}

#[test]
fn move_within_a_volume_renames_and_guards_bad_targets() {
    let mut f = fixture();
    let moved = plan(&mut f, PlanAction::Move, &["papka"], ConflictPolicy::Ask).expect("plan");
    let (report, progress) = run(&mut f, &moved);
    assert_eq!(report.done, ["papka"]);
    assert!(!f.src.join("papka").exists());
    assert!(f.dst.join("papka/ichki/b.bin").exists());
    assert_eq!(progress.last().map(|p| p.done_items), Some(4));

    // Papkani o‘z ichiga ko‘chirish/nusxalash va o‘sha papkaga ko‘chirish rad etiladi.
    fs::create_dir_all(f.src.join("yangi/ichida")).expect("mkdir");
    let src = f.src.clone();
    let source = list(&mut f.service, &src);
    let inner = list(&mut f.service, &src.join("yangi/ichida"));
    let into_itself = f.service.create_plan(PlanRequest::Copy {
        location: source.location.id,
        entries: ids(&source, &["yangi"]),
        target: inner.location.id,
        policy: ConflictPolicy::Ask,
    });
    assert_eq!(
        into_itself.expect_err("itself").code,
        ErrorCode::InvalidArgument
    );
    let same_dir = f.service.create_plan(PlanRequest::Move {
        location: source.location.id,
        entries: ids(&source, &["hisobot.txt"]),
        target: source.location.id,
        policy: ConflictPolicy::Ask,
    });
    assert_eq!(same_dir.expect_err("same").code, ErrorCode::InvalidArgument);
}

#[test]
fn cancel_leaves_no_partial_file_and_untouched_remaining_items() {
    let mut f = fixture();
    fs::write(f.src.join("katta.bin"), vec![1_u8; 3 << 20]).expect("w");
    let plan = plan(
        &mut f,
        PlanAction::Copy,
        &["katta.bin", "hisobot.txt"],
        ConflictPolicy::Ask,
    )
    .expect("plan");
    let job = f.service.start_transfer(plan.id, plan.hash).expect("start");
    let cancel = AtomicBool::new(false);
    let report = job.run(&cancel, &mut |progress| {
        if progress.done_bytes >= 1 << 20 {
            cancel.store(true, Ordering::Relaxed);
        }
    });
    assert!(report.cancelled);
    assert!(report.done.is_empty());
    assert!(
        !f.dst.join("katta.bin").exists(),
        "yarim fayl to‘liq deb qolmaydi"
    );
    assert!(
        !f.dst.join("hisobot.txt").exists(),
        "keyingi yozuvga tegilmaydi"
    );
    let names: Vec<_> = fs::read_dir(&f.dst).expect("dir").flatten().collect();
    assert!(names.is_empty(), "vaqtinchalik fayl ham o‘chiriladi");
}

#[test]
fn replaced_source_is_stale_and_plan_is_single_use() {
    let mut f = fixture();
    let plan = plan(
        &mut f,
        PlanAction::Move,
        &["hisobot.txt"],
        ConflictPolicy::Ask,
    )
    .expect("plan");
    fs::remove_file(f.src.join("hisobot.txt")).expect("rm");
    fs::write(f.src.join("hisobot.txt"), "boshqa").expect("w");
    let (report, _) = run(&mut f, &plan);
    if cfg!(unix) {
        assert_eq!(report.failed[0].error.code, ErrorCode::StaleReference);
        assert!(!f.dst.join("hisobot.txt").exists());
    }
    let again = f.service.start_transfer(plan.id, plan.hash);
    assert_eq!(again.expect_err("used").code, ErrorCode::PlanExpired);
}

#[cfg(unix)]
#[test]
fn symlinks_are_copied_as_links_not_followed() {
    let mut f = fixture();
    std::os::unix::fs::symlink("/etc", f.src.join("havola")).expect("link");
    let plan = plan(&mut f, PlanAction::Copy, &["havola"], ConflictPolicy::Ask).expect("plan");
    assert_eq!(plan.totals.map(|t| t.items), Some(1));
    let (report, _) = run(&mut f, &plan);
    assert_eq!(report.done, ["havola"]);
    let copied = f.dst.join("havola");
    assert!(fs::symlink_metadata(&copied).expect("meta").is_symlink());
    assert_eq!(fs::read_link(copied).expect("link"), PathBuf::from("/etc"));
}

/// Disklararo ko‘chirish: tizim diskidagi temp → loyiha diskidagi `target/`. Qurilmalar bir xil
/// bo‘lsa (masalan, CI) test faqat oddiy ko‘chirishni tekshiradi.
#[test]
fn cross_device_move_copies_verifies_and_removes_source() {
    let a = tempfile::tempdir().expect("temp a");
    let b = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp b");
    tree(a.path());
    let mut service = FileService::new(a.path().into());
    let source = list(&mut service, a.path());
    let target = list(&mut service, b.path());
    let plan = service
        .create_plan(PlanRequest::Move {
            location: source.location.id,
            entries: ids(&source, &["papka", "hisobot.txt"]),
            target: target.location.id,
            policy: ConflictPolicy::Ask,
        })
        .expect("plan");
    let job = service.start_transfer(plan.id, plan.hash).expect("start");
    let report = job.run(&AtomicBool::new(false), &mut |_| {});
    assert_eq!(report.done, ["papka", "hisobot.txt"], "{:?}", report.failed);
    assert!(!a.path().join("papka").exists() && !a.path().join("hisobot.txt").exists());
    assert_eq!(
        fs::read(b.path().join("papka/ichki/b.bin")).expect("r"),
        vec![7_u8; 4096]
    );
}
