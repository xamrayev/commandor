use commandor_core::{DirectoryTarget, EntryKind, ErrorCode, ListOptions, LocationId};
use commandor_fs::{FileService, LIST_LIMIT};
use std::fs;
use tempfile::TempDir;

fn fixture() -> (TempDir, FileService) {
    let dir = tempfile::tempdir().expect("temp");
    fs::create_dir(dir.path().join("O‘zbek papka")).expect("mkdir");
    fs::write(dir.path().join("hisobot.txt"), "Salom").expect("write");
    fs::write(dir.path().join(".hidden"), "private fixture").expect("write");
    let service = FileService::new(dir.path().into());
    (dir, service)
}

#[test]
fn real_metadata_hidden_filter_and_unicode_navigation() {
    let (_fixture, mut service) = fixture();
    let listing = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("list");
    assert_eq!(listing.entries.len(), 2);
    assert_eq!(listing.entries[0].kind, EntryKind::Directory);
    assert_eq!(listing.entries[1].size_bytes, Some(5));
    assert!(listing.entries[1].modified_unix_seconds.is_some());
    let child = service
        .read(
            DirectoryTarget::Child {
                location: listing.location.id,
                entry: listing.entries[0].id,
            },
            ListOptions::default(),
        )
        .expect("child");
    assert!(child.location.display_path.ends_with("O‘zbek papka"));
    assert!(child.entries.is_empty());
    let parent = service
        .read(
            DirectoryTarget::Parent(child.location.id),
            ListOptions {
                hidden: true,
                filter: String::new(),
                ..Default::default()
            },
        )
        .expect("parent");
    assert_eq!(parent.entries.len(), 3);
    let filtered = service
        .read(
            DirectoryTarget::Refresh(parent.location.id),
            ListOptions {
                hidden: false,
                filter: "HISOBOT".into(),
                ..Default::default()
            },
        )
        .expect("filter");
    assert_eq!(filtered.entries.len(), 1);
    assert_eq!(filtered.entries[0].name, "hisobot.txt");
}

#[test]
fn other_panel_refresh_does_not_invalidate_entry_grants() {
    let (_fixture, mut service) = fixture();
    let first = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("first");
    let other = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("second");
    assert_ne!(first.location.id, other.location.id);
    service
        .read(
            DirectoryTarget::Refresh(other.location.id),
            ListOptions::default(),
        )
        .expect("refresh");
    assert!(
        service
            .read(
                DirectoryTarget::Child {
                    location: first.location.id,
                    entry: first.entries[0].id
                },
                ListOptions::default()
            )
            .is_ok()
    );
    let error = service
        .read(
            DirectoryTarget::Child {
                location: first.location.id,
                entry: other.entries[0].id,
            },
            ListOptions::default(),
        )
        .expect_err("cross-location ID");
    assert_eq!(error.code, ErrorCode::StaleReference);
}

#[test]
fn invalid_missing_and_unknown_locations_are_explicit_errors() {
    let (fixture, mut service) = fixture();
    for target in [
        DirectoryTarget::Path("../escape".into()),
        DirectoryTarget::Path("/bad\0name".into()),
        DirectoryTarget::Path(
            fixture
                .path()
                .join("hisobot.txt")
                .to_string_lossy()
                .into_owned(),
        ),
    ] {
        assert_eq!(
            service
                .read(target, ListOptions::default())
                .expect_err("invalid")
                .code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        service
            .read(
                DirectoryTarget::Refresh(LocationId::from_raw(u64::MAX)),
                ListOptions::default()
            )
            .expect_err("unknown")
            .code,
        ErrorCode::StaleReference
    );
    let error = service
        .read(
            DirectoryTarget::Path(
                fixture
                    .path()
                    .join("missing")
                    .to_string_lossy()
                    .into_owned(),
            ),
            ListOptions::default(),
        )
        .expect_err("missing");
    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(
        error.details.is_empty(),
        "disk path must not leak in errors"
    );
    assert_eq!(
        service
            .read(
                DirectoryTarget::Home,
                ListOptions {
                    hidden: false,
                    filter: "a".repeat(1025),
                    ..Default::default()
                }
            )
            .expect_err("filter")
            .code,
        ErrorCode::InvalidArgument
    );
}

#[cfg(unix)]
#[test]
fn symlink_cycle_and_replaced_grants_are_rejected() {
    use std::os::unix::fs::symlink;
    let (fixture, mut service) = fixture();
    symlink(fixture.path(), fixture.path().join("cycle")).expect("symlink");
    let listing = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("list");
    let link = listing
        .entries
        .iter()
        .find(|e| e.name == "cycle")
        .expect("link");
    assert_eq!(link.kind, EntryKind::Symlink);
    assert_eq!(
        service
            .read(
                DirectoryTarget::Child {
                    location: listing.location.id,
                    entry: link.id
                },
                ListOptions::default()
            )
            .expect_err("no follow")
            .code,
        ErrorCode::PolicyBlocked
    );
    let child = service
        .read(
            DirectoryTarget::Child {
                location: listing.location.id,
                entry: listing.entries[0].id,
            },
            ListOptions::default(),
        )
        .expect("child");
    fs::rename(
        fixture.path().join("O‘zbek papka"),
        fixture.path().join("old"),
    )
    .expect("rename");
    fs::create_dir(fixture.path().join("O‘zbek papka")).expect("replace");
    assert_eq!(
        service
            .read(
                DirectoryTarget::Refresh(child.location.id),
                ListOptions::default()
            )
            .expect_err("replaced")
            .code,
        ErrorCode::StaleReference
    );
}

// APFS refuses invalid UTF-8 filenames. This on-disk fixture runs on Linux CI.
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_directory_is_navigated_by_id_without_lossy_roundtrip() {
    use std::os::unix::ffi::OsStringExt;
    let fixture = tempfile::tempdir().expect("temp");
    let raw_name = std::ffi::OsString::from_vec(vec![b'n', 0xff]);
    fs::create_dir(fixture.path().join(&raw_name)).expect("mkdir");
    fs::write(fixture.path().join(raw_name).join("inside.txt"), "data").expect("write");
    let mut service = FileService::new(fixture.path().into());
    let listing = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("list");
    assert!(listing.entries[0].name.contains('\u{fffd}'));
    let child = service
        .read(
            DirectoryTarget::Child {
                location: listing.location.id,
                entry: listing.entries[0].id,
            },
            ListOptions::default(),
        )
        .expect("child");
    assert_eq!(child.entries[0].name, "inside.txt");
    assert!(
        service
            .read(
                DirectoryTarget::Refresh(child.location.id),
                ListOptions::default()
            )
            .is_ok()
    );
}

#[test]
fn large_listing_is_bounded_and_filter_can_find_an_omitted_entry() {
    let fixture = tempfile::tempdir().expect("temp");
    for index in 0..=LIST_LIMIT {
        fs::write(fixture.path().join(format!("file-{index:05}")), []).expect("file");
    }
    let mut service = FileService::new(fixture.path().into());
    let listing = service
        .read(DirectoryTarget::Home, ListOptions::default())
        .expect("list");
    assert_eq!(listing.entries.len(), LIST_LIMIT);
    assert!(listing.truncated);
    let missing = (0..=LIST_LIMIT)
        .map(|i| format!("file-{i:05}"))
        .find(|name| !listing.entries.iter().any(|entry| entry.name == *name))
        .expect("omitted entry");
    let filtered = service
        .read(
            DirectoryTarget::Refresh(listing.location.id),
            ListOptions {
                hidden: false,
                filter: missing.clone(),
                ..Default::default()
            },
        )
        .expect("filtered");
    assert_eq!(filtered.entries.len(), 1);
    assert_eq!(filtered.entries[0].name, missing);
    assert!(!filtered.truncated);
}

#[test]
fn sorting_by_name_size_and_modified() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir(temp.path().join("z_dir")).expect("dir");
    fs::create_dir(temp.path().join("a_dir")).expect("dir");
    fs::write(temp.path().join("small.txt"), "a").expect("write");
    fs::write(temp.path().join("large.txt"), "hello world 1234567890").expect("write");

    let mut service = FileService::new(temp.path().to_path_buf());

    // Nom bo'yicha kamayish tartibida (Descending)
    let listing = service
        .read(
            DirectoryTarget::Home,
            ListOptions {
                sort: commandor_core::SortCriteria {
                    field: commandor_core::SortField::Name,
                    direction: commandor_core::SortDirection::Descending,
                },
                ..Default::default()
            },
        )
        .expect("read");
    let names: Vec<_> = listing.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["z_dir", "a_dir", "small.txt", "large.txt"]);

    // Hajm bo'yicha o'sish tartibida (Ascending)
    let listing = service
        .read(
            DirectoryTarget::Home,
            ListOptions {
                sort: commandor_core::SortCriteria {
                    field: commandor_core::SortField::Size,
                    direction: commandor_core::SortDirection::Ascending,
                },
                ..Default::default()
            },
        )
        .expect("read");
    let names: Vec<_> = listing.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names[2], "small.txt");
    assert_eq!(names[3], "large.txt");
}
