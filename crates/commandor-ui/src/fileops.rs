//! Fayl amallari UI holati: yangi papka (F7), rename (Shift+F6), savat (F8),
//! nusxalash (F5) va ko‘chirish (F6).
//!
//! Oqim: so‘rov → broker rejasi → (kerak bo‘lsa tasdiq) → bajarish → panelni yangilash.
//! Nusxalash/ko‘chirish fon vazifasi: `JobStarted` → `JobProgress` → `JobFinished`.
//! `apply` dispatch qila olmaydi, shuning uchun keyingi qadamlar navbatga qo‘yiladi va
//! [`AppState::pump`] ularni yuboradi (UI har kadrda chaqiradi).

use crate::opening::{Notice, NoticeKind};
use crate::state::{AppState, Dispatch, Side};
use commandor_core::{
    AppError, Command, ConflictPolicy, EntryId, ErrorCode, Event, JobId, JobProgress, LocationId,
    OperationPlan, OperationReport, PlanAction, PlanRequest, Reply, RequestId, Response,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameKind {
    NewFolder,
    Rename { entry: EntryId, original: String },
}

/// Nom kiritish oynasi (yangi papka yoki rename).
#[derive(Debug, Clone, PartialEq)]
pub struct NameDialog {
    pub side: Side,
    pub location: LocationId,
    pub kind: NameKind,
    pub text: String,
    /// Reja yoki bajarish javobi kutilmoqda.
    pub pending: Option<RequestId>,
    pub error: Option<AppError>,
    /// Oyna yangi ochildi: UI fokus beradi va nomni (kengaytmasiz) belgilaydi.
    pub fresh: bool,
}

/// Nusxalash/ko‘chirish so‘rovi: konflikt siyosati o‘zgarsa shu bilan qayta reja tuziladi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferRequest {
    pub action: PlanAction,
    pub location: LocationId,
    pub entries: Vec<EntryId>,
    pub target: LocationId,
}

impl TransferRequest {
    fn plan(&self, policy: ConflictPolicy) -> PlanRequest {
        let (location, entries, target) = (self.location, self.entries.clone(), self.target);
        if self.action == PlanAction::Move {
            PlanRequest::Move {
                location,
                entries,
                target,
                policy,
            }
        } else {
            PlanRequest::Copy {
                location,
                entries,
                target,
                policy,
            }
        }
    }
}

/// Tasdiq talab qiladigan reja (savat, nusxalash, ko‘chirish).
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDialog {
    pub side: Side,
    pub plan: OperationPlan,
    /// Bajarish javobi kutilmoqda.
    pub pending: Option<RequestId>,
    /// Nusxalash/ko‘chirish bo‘lsa — qayta reja uchun asl so‘rov.
    pub transfer: Option<TransferRequest>,
    /// Konflikt siyosati o‘zgardi, yangi reja kutilmoqda.
    pub replanning: Option<RequestId>,
}

impl PlanDialog {
    pub fn conflicts(&self) -> usize {
        self.plan.items.iter().filter(|item| item.conflict).count()
    }

    /// Konflikt bo‘lsa siyosat tanlanmaguncha tasdiqlab bo‘lmaydi.
    pub fn can_confirm(&self) -> bool {
        self.pending.is_none()
            && self.replanning.is_none()
            && (self.conflicts() == 0 || self.plan.policy != ConflictPolicy::Ask)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpDialog {
    Name(NameDialog),
    Confirm(PlanDialog),
}

#[derive(Debug, Clone, PartialEq)]
struct Refresh {
    directory: String,
    side: Side,
    focus: Option<String>,
}

/// Ishlayotgan fon vazifasi.
#[derive(Debug, Clone, PartialEq)]
pub struct JobView {
    pub job: JobId,
    pub action: PlanAction,
    pub side: Side,
    /// Ko‘rsatish uchun: birinchi yozuv nomi va jami soni.
    pub first: String,
    pub count: usize,
    pub progress: Option<JobProgress>,
    pub cancelling: bool,
}

impl JobView {
    /// 0..=1 ulush (jami ma’lum bo‘lsa). Baytlar bo‘lsa baytlar bo‘yicha, aks holda yozuvlar.
    pub fn fraction(&self) -> Option<f32> {
        let progress = self.progress.as_ref()?;
        let totals = progress.totals?;
        let (done, total) = if totals.bytes > 0 {
            (progress.done_bytes, totals.bytes)
        } else {
            (progress.done_items, totals.items)
        };
        (total > 0).then(|| (done as f32 / total as f32).clamp(0.0, 1.0))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OpsState {
    pub dialog: Option<OpDialog>,
    /// Tasdiq talab qiladigan reja so‘ralgan (oyna reja kelgach ochiladi).
    plan_request: Option<(RequestId, Side, Option<TransferRequest>)>,
    /// Tasdiq talab qilmaydigan reja keldi: `pump` uni bajaradi.
    auto_execute: Option<OperationPlan>,
    refresh: Vec<Refresh>,
    pub jobs: Vec<JobView>,
}

impl OpsState {
    pub fn is_modal(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn is_busy(&self) -> bool {
        self.plan_request.is_some() || self.dialog.as_ref().is_some_and(OpDialog::is_busy)
    }
}

impl OpDialog {
    fn is_busy(&self) -> bool {
        match self {
            Self::Name(dialog) => dialog.pending.is_some(),
            Self::Confirm(dialog) => dialog.pending.is_some() || dialog.replanning.is_some(),
        }
    }
}

impl AppState {
    /// F7: faol paneldagi papkada yangi papka nomini so‘raydi.
    pub fn start_new_folder(&mut self) {
        let side = self.active;
        let Some(location) = self.panel(side).listing.as_ref().map(|l| l.location.id) else {
            return;
        };
        self.close_overlays();
        self.ops.dialog = Some(OpDialog::Name(NameDialog {
            side,
            location,
            kind: NameKind::NewFolder,
            text: String::new(),
            pending: None,
            error: None,
            fresh: true,
        }));
    }

    /// Shift+F6: kursordagi yozuv nomini o‘zgartirish.
    pub fn start_rename(&mut self) {
        let side = self.active;
        let panel = self.panel(side);
        let Some(listing) = &panel.listing else {
            return;
        };
        let Some(entry) = listing.entries.iter().find(|e| Some(e.id) == panel.cursor) else {
            return;
        };
        let dialog = NameDialog {
            side,
            location: listing.location.id,
            kind: NameKind::Rename {
                entry: entry.id,
                original: entry.name.clone(),
            },
            text: entry.name.clone(),
            pending: None,
            error: None,
            fresh: true,
        };
        self.close_overlays();
        self.ops.dialog = Some(OpDialog::Name(dialog));
    }

    /// Tanlangan yozuvlar (listing tartibida); tanlov bo‘lmasa — kursordagi.
    fn targets(&self, side: Side) -> Option<(LocationId, Vec<EntryId>)> {
        let panel = self.panel(side);
        let listing = panel.listing.as_ref()?;
        let mut entries: Vec<EntryId> = listing
            .entries
            .iter()
            .filter(|entry| panel.selected.contains(&entry.id))
            .map(|entry| entry.id)
            .collect();
        if entries.is_empty() {
            entries.extend(
                panel
                    .cursor
                    .filter(|id| listing.entries.iter().any(|e| e.id == *id)),
            );
        }
        (!entries.is_empty()).then_some((listing.location.id, entries))
    }

    fn request_plan(
        &mut self,
        dispatch: &impl Dispatch,
        side: Side,
        request: PlanRequest,
        transfer: Option<TransferRequest>,
    ) {
        self.close_overlays();
        match dispatch.dispatch(Command::CreatePlan(request)) {
            Some(id) => self.ops.plan_request = Some((id, side, transfer)),
            None => self.op_notice(String::new(), 1, 1, ErrorCode::Internal.into()),
        }
    }

    /// F8: tanlangan yozuvlar (tanlov bo‘lmasa — kursordagi) uchun savat rejasi.
    pub fn start_trash(&mut self, dispatch: &impl Dispatch) {
        if self.ops.is_busy() {
            return;
        }
        let side = self.active;
        if let Some((location, entries)) = self.targets(side) {
            let request = PlanRequest::Trash { location, entries };
            self.request_plan(dispatch, side, request, None);
        }
    }

    /// F5/F6: faol panel tanlovini boshqa panelning joriy papkasiga nusxalash/ko‘chirish.
    pub fn start_transfer(&mut self, dispatch: &impl Dispatch, action: PlanAction) {
        if self.ops.is_busy() || !matches!(action, PlanAction::Copy | PlanAction::Move) {
            return;
        }
        let side = self.active;
        let Some((location, entries)) = self.targets(side) else {
            return;
        };
        let Some(target) = self
            .panel(side.other())
            .listing
            .as_ref()
            .map(|l| l.location.id)
        else {
            return;
        };
        let transfer = TransferRequest {
            action,
            location,
            entries,
            target,
        };
        let request = transfer.plan(ConflictPolicy::Ask);
        self.request_plan(dispatch, side, request, Some(transfer));
    }

    /// Konflikt siyosatini tanlash: eski reja unutiladi va yangisi so‘raladi.
    pub fn choose_policy(&mut self, dispatch: &impl Dispatch, policy: ConflictPolicy) {
        let Some(OpDialog::Confirm(dialog)) = self.ops.dialog.as_mut() else {
            return;
        };
        let Some(transfer) = &dialog.transfer else {
            return;
        };
        if dialog.pending.is_some() || dialog.replanning.is_some() || dialog.plan.policy == policy {
            return;
        }
        let _ = dispatch.dispatch(Command::DiscardPlan {
            plan: dialog.plan.id,
        });
        dialog.replanning = dispatch.dispatch(Command::CreatePlan(transfer.plan(policy)));
    }

    /// Nom oynasida Enter / tugma: reja so‘raladi (bajarish reja kelgach `pump` orqali).
    pub fn submit_name(&mut self, dispatch: &impl Dispatch) {
        let Some(OpDialog::Name(dialog)) = self.ops.dialog.as_mut() else {
            return;
        };
        if dialog.pending.is_some() {
            return;
        }
        let name = dialog.text.trim().to_owned();
        let request = match &dialog.kind {
            NameKind::NewFolder => PlanRequest::CreateDirectory {
                location: dialog.location,
                name,
            },
            NameKind::Rename { entry, .. } => PlanRequest::Rename {
                location: dialog.location,
                entry: *entry,
                new_name: name,
            },
        };
        dialog.error = None;
        dialog.pending = dispatch.dispatch(Command::CreatePlan(request));
        if dialog.pending.is_none() {
            dialog.error = Some(ErrorCode::Internal.into());
        }
    }

    /// Tasdiq oynasidagi asosiy tugma. Konflikt bo‘lsa siyosat tanlanmaguncha ishlamaydi.
    pub fn confirm_plan(&mut self, dispatch: &impl Dispatch) {
        if let Some(OpDialog::Confirm(dialog)) = self.ops.dialog.as_mut()
            && dialog.can_confirm()
        {
            dialog.pending = dispatch.dispatch(Command::ExecutePlan {
                plan: dialog.plan.id,
                hash: dialog.plan.hash,
            });
        }
    }

    /// Esc / Bekor qilish. Tasdiqlanmagan reja brokerda ham unutiladi.
    pub fn cancel_op(&mut self, dispatch: &impl Dispatch) {
        match self.ops.dialog.take() {
            Some(other) if other.is_busy() => {
                // Bajarish yoki qayta reja ketmoqda: oyna yopilmaydi, natija kutiladi.
                self.ops.dialog = Some(other);
            }
            Some(OpDialog::Confirm(dialog)) => {
                let _ = dispatch.dispatch(Command::DiscardPlan {
                    plan: dialog.plan.id,
                });
            }
            _ => {}
        }
    }

    /// Fon vazifasini bekor qilish (natija `JobFinished` bilan keladi).
    pub fn cancel_job(&mut self, dispatch: &impl Dispatch, job: JobId) {
        if let Some(view) = self.ops.jobs.iter_mut().find(|view| view.job == job)
            && !view.cancelling
        {
            view.cancelling = dispatch.dispatch(Command::CancelJob { job }).is_some();
        }
    }

    fn close_overlays(&mut self) {
        self.palette = None;
        self.volume_menu = None;
        self.opening.menu = None;
        self.opening.notice = None;
    }

    fn op_notice(&mut self, name: String, failed: usize, total: usize, error: AppError) {
        self.opening.notice = Some(Notice {
            kind: NoticeKind::Operation { failed, total },
            file: name,
            error,
        });
    }

    /// Navbatdagi keyingi qadamlar: tasdiqsiz rejani bajarish va papkalarni yangilash.
    pub fn pump(&mut self, dispatch: &impl Dispatch) {
        if let Some(plan) = self.ops.auto_execute.take()
            && let Some(OpDialog::Name(dialog)) = self.ops.dialog.as_mut()
        {
            dialog.pending = dispatch.dispatch(Command::ExecutePlan {
                plan: plan.id,
                hash: plan.hash,
            });
            if dialog.pending.is_none() {
                dialog.error = Some(ErrorCode::Internal.into());
            }
        }
        for refresh in std::mem::take(&mut self.ops.refresh) {
            for (index, tabs) in self.panels.iter_mut().enumerate() {
                let current = tabs.current;
                for (tab_index, panel) in tabs.tabs.iter_mut().enumerate() {
                    if panel.current_path() != Some(refresh.directory.as_str()) {
                        continue;
                    }
                    if index == refresh.side.index() && tab_index == current {
                        panel.focus_name.clone_from(&refresh.focus);
                    }
                    panel.refresh(dispatch);
                }
            }
        }
    }

    fn finish(&mut self, side: Side, report: &OperationReport) {
        let total = report.done.len() + report.failed.len() + report.skipped.len();
        if let Some(first) = report.failed.first() {
            self.op_notice(
                first.name.clone(),
                report.failed.len(),
                total,
                first.error.clone(),
            );
        } else if report.cancelled {
            self.opening.notice = Some(Notice {
                kind: NoticeKind::Cancelled {
                    done: report.done.len(),
                },
                file: String::new(),
                error: ErrorCode::Cancelled.into(),
            });
        }
        let transfer = matches!(report.action, PlanAction::Copy | PlanAction::Move);
        self.ops.refresh.push(Refresh {
            directory: report.directory.clone(),
            side,
            // Nusxalashda kursor manbada qoladi; yangi nom maqsad panelida ko‘rsatiladi.
            focus: (!transfer).then(|| report.done.first().cloned()).flatten(),
        });
        if let Some(target) = &report.target
            && *target != report.directory
        {
            self.ops.refresh.push(Refresh {
                directory: target.clone(),
                side: side.other(),
                focus: report.done.first().cloned(),
            });
        }
    }

    /// Fon vazifasi hodisalari (`Event::JobProgress`/`JobFinished`).
    pub(crate) fn apply_job_event(&mut self, event: Event) {
        match event {
            Event::JobProgress(progress) => {
                if let Some(view) = self.ops.jobs.iter_mut().find(|v| v.job == progress.job) {
                    view.progress = Some(progress);
                }
            }
            Event::JobFinished { job, result } => {
                let Some(index) = self.ops.jobs.iter().position(|v| v.job == job) else {
                    return;
                };
                let view = self.ops.jobs.remove(index);
                match result {
                    Ok(report) => self.finish(view.side, &report),
                    Err(error) => self.op_notice(view.first, view.count, view.count, error),
                }
            }
            _ => {}
        }
    }

    /// Fayl amaliga oid javobni qo‘llaydi; boshqa so‘rovga tegishli bo‘lsa javobni qaytaradi.
    pub(crate) fn apply_ops(&mut self, request: RequestId, reply: Reply) -> Option<Reply> {
        if let Some((pending, side, _)) = &self.ops.plan_request
            && *pending == request
        {
            let side = *side;
            let transfer = self.ops.plan_request.take().and_then(|(_, _, t)| t);
            match reply {
                Ok(Response::Plan(plan)) => {
                    self.ops.dialog = Some(OpDialog::Confirm(PlanDialog {
                        side,
                        plan,
                        pending: None,
                        transfer,
                        replanning: None,
                    }));
                }
                Ok(_) => self.op_notice(String::new(), 1, 1, ErrorCode::Internal.into()),
                Err(error) => self.op_notice(String::new(), 1, 1, error),
            }
            return None;
        }
        match self.ops.dialog.take() {
            Some(OpDialog::Name(mut dialog)) if dialog.pending == Some(request) => {
                dialog.pending = None;
                let side = dialog.side;
                match reply {
                    Ok(Response::Plan(plan)) if !plan.requires_approval => {
                        self.ops.auto_execute = Some(plan);
                        // `pump` bajarishni yuborguncha oyna "band" ko‘rinadi.
                        dialog.pending = Some(request);
                        self.ops.dialog = Some(OpDialog::Name(dialog));
                    }
                    Ok(Response::Executed(report)) => self.finish(side, &report),
                    Ok(_) => {
                        dialog.error = Some(ErrorCode::Internal.into());
                        self.ops.dialog = Some(OpDialog::Name(dialog));
                    }
                    Err(error) => {
                        dialog.error = Some(error);
                        self.ops.dialog = Some(OpDialog::Name(dialog));
                    }
                }
                None
            }
            Some(OpDialog::Confirm(mut dialog)) if dialog.replanning == Some(request) => {
                dialog.replanning = None;
                match reply {
                    Ok(Response::Plan(plan)) => {
                        dialog.plan = plan;
                        self.ops.dialog = Some(OpDialog::Confirm(dialog));
                    }
                    // Eski reja allaqachon unutilgan: oyna yopiladi, sabab xabarda.
                    Ok(_) => self.op_notice(String::new(), 1, 1, ErrorCode::Internal.into()),
                    Err(error) => self.op_notice(String::new(), 1, 1, error),
                }
                None
            }
            Some(OpDialog::Confirm(dialog)) if dialog.pending == Some(request) => {
                let first = dialog.plan.items.first().map(|i| i.name.clone());
                let count = dialog.plan.items.len();
                match reply {
                    Ok(Response::Executed(report)) => self.finish(dialog.side, &report),
                    Ok(Response::JobStarted(job)) => self.ops.jobs.push(JobView {
                        job,
                        action: dialog.plan.action,
                        side: dialog.side,
                        first: first.unwrap_or_default(),
                        count,
                        progress: None,
                        cancelling: false,
                    }),
                    Ok(_) => self.op_notice(String::new(), 1, 1, ErrorCode::Internal.into()),
                    // Reja bajarishda iste’mol qilinadi: oyna yopiladi, sabab xabarda.
                    Err(error) => self.op_notice(first.unwrap_or_default(), count, count, error),
                }
                None
            }
            other => {
                self.ops.dialog = other;
                Some(reply)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PaletteCommand;
    use commandor_core::{
        DirectoryListing, EntryKind, FileEntry, ItemFailure, Location, PlanId, PlanItem, Totals,
    };
    use std::cell::RefCell;

    struct Recorder(RefCell<Vec<Command>>);
    impl Dispatch for Recorder {
        fn dispatch(&self, command: Command) -> Option<RequestId> {
            let mut log = self.0.borrow_mut();
            log.push(command);
            Some(RequestId::from_raw(500 + log.len() as u64))
        }
    }

    fn last(rec: &Recorder) -> (RequestId, Command) {
        let log = rec.0.borrow();
        (
            RequestId::from_raw(500 + log.len() as u64),
            log.last().cloned().expect("command"),
        )
    }

    fn entry(id: u64, name: &str) -> FileEntry {
        FileEntry {
            id: EntryId::from_raw(id),
            name: name.into(),
            kind: EntryKind::File,
            size_bytes: Some(10),
            modified_unix_seconds: None,
        }
    }

    fn listing(names: &[&str]) -> DirectoryListing {
        DirectoryListing {
            location: Location {
                id: LocationId::from_raw(9),
                display_path: "/ish".into(),
                can_go_up: true,
            },
            entries: names
                .iter()
                .enumerate()
                .map(|(i, name)| entry(i as u64 + 1, name))
                .collect(),
            truncated: false,
            limit: 100,
            skipped_entries: 0,
        }
    }

    fn ready(rec: &Recorder) -> AppState {
        let mut s = AppState::start(rec);
        for side in [Side::Left, Side::Right] {
            let request = s.panel(side).pending.expect("pending");
            s.apply(Event::Reply {
                request,
                reply: Ok(Response::Directory(listing(&["a.txt", "b.txt"]))),
            });
        }
        s
    }

    fn plan(action: PlanAction, names: &[&str], approval: bool) -> OperationPlan {
        OperationPlan {
            id: PlanId::from_raw(77),
            hash: 1234,
            action,
            directory: "/ish".into(),
            items: names
                .iter()
                .map(|name| PlanItem {
                    name: (*name).into(),
                    new_name: None,
                    kind: EntryKind::File,
                    size_bytes: Some(10),
                    conflict: false,
                })
                .collect(),
            requires_approval: approval,
            target: None,
            policy: ConflictPolicy::Ask,
            totals: None,
        }
    }

    fn reply(s: &mut AppState, request: RequestId, reply: Reply) {
        s.apply(Event::Reply { request, reply });
    }

    #[test]
    fn new_folder_plans_executes_without_extra_confirmation_and_refreshes_both_panels() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.run_command(&rec, PaletteCommand::NewFolder);
        let Some(OpDialog::Name(dialog)) = s.ops.dialog.as_mut() else {
            panic!("dialog");
        };
        dialog.text = "  Yangi  ".into();
        s.submit_name(&rec);
        let (request, command) = last(&rec);
        assert_eq!(
            command,
            Command::CreatePlan(PlanRequest::CreateDirectory {
                location: LocationId::from_raw(9),
                name: "Yangi".into(),
            }),
            "chetdagi bo‘shliqlar olib tashlanadi"
        );
        assert!(s.ops.is_busy());
        reply(
            &mut s,
            request,
            Ok(Response::Plan(plan(
                PlanAction::CreateDirectory,
                &["Yangi"],
                false,
            ))),
        );
        s.pump(&rec);
        let (request, command) = last(&rec);
        assert_eq!(
            command,
            Command::ExecutePlan {
                plan: PlanId::from_raw(77),
                hash: 1234
            }
        );
        reply(
            &mut s,
            request,
            Ok(Response::Executed(OperationReport {
                action: PlanAction::CreateDirectory,
                directory: "/ish".into(),
                done: vec!["Yangi".into()],
                failed: vec![],
                skipped: vec![],
                target: None,
                cancelled: false,
            })),
        );
        assert!(s.ops.dialog.is_none());
        let before = rec.0.borrow().len();
        s.pump(&rec);
        assert_eq!(
            rec.0.borrow().len(),
            before + 2,
            "shu papkani ko‘rsatgan ikkala panel yangilanadi"
        );
        assert_eq!(s.panel(Side::Left).focus_name.as_deref(), Some("Yangi"));
        assert_eq!(s.panel(Side::Right).focus_name, None);

        // Yangilangan listingda yangi papka kursorga olinadi.
        let request = s.panel(Side::Left).pending.expect("refresh");
        reply(
            &mut s,
            request,
            Ok(Response::Directory(listing(&["a.txt", "Yangi", "b.txt"]))),
        );
        assert_eq!(s.panel(Side::Left).cursor, Some(EntryId::from_raw(2)));
        assert!(s.panel(Side::Left).reveal_cursor);
    }

    #[test]
    fn rename_error_keeps_dialog_open_for_correction() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.start_rename();
        assert!(s.ops.dialog.is_none(), "kursor yo‘q — rename yo‘q");
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(2));
        s.run_command(&rec, PaletteCommand::Rename);
        let Some(OpDialog::Name(dialog)) = &s.ops.dialog else {
            panic!("dialog");
        };
        assert_eq!(dialog.text, "b.txt");
        s.submit_name(&rec);
        let (request, command) = last(&rec);
        assert!(matches!(
            command,
            Command::CreatePlan(PlanRequest::Rename { .. })
        ));
        reply(&mut s, request, Err(ErrorCode::DestinationExists.into()));
        let Some(OpDialog::Name(dialog)) = &s.ops.dialog else {
            panic!("dialog qoladi");
        };
        assert_eq!(
            dialog.error.as_ref().map(|e| e.code),
            Some(ErrorCode::DestinationExists)
        );
        assert!(!s.ops.is_busy());
        s.cancel_op(&rec);
        assert!(s.ops.dialog.is_none());
    }

    #[test]
    fn trash_uses_selection_needs_confirmation_and_reports_failures() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.start_trash(&rec);
        assert!(!s.ops.is_busy(), "kursor va tanlov yo‘q");
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.panel_mut(Side::Left)
            .selected
            .insert(EntryId::from_raw(2));
        s.run_command(&rec, PaletteCommand::Trash);
        let (request, command) = last(&rec);
        assert_eq!(
            command,
            Command::CreatePlan(PlanRequest::Trash {
                location: LocationId::from_raw(9),
                entries: vec![EntryId::from_raw(2)],
            }),
            "tanlov bo‘lsa kursor emas, tanlov"
        );
        reply(
            &mut s,
            request,
            Ok(Response::Plan(plan(PlanAction::Trash, &["b.txt"], true))),
        );
        assert!(
            matches!(s.ops.dialog, Some(OpDialog::Confirm(_))),
            "avval tasdiq"
        );
        s.pump(&rec);
        assert!(
            !matches!(last(&rec).1, Command::ExecutePlan { .. }),
            "tasdiqsiz bajarilmaydi"
        );
        s.confirm_plan(&rec);
        let (request, command) = last(&rec);
        assert!(matches!(command, Command::ExecutePlan { .. }));
        s.cancel_op(&rec);
        assert!(
            s.ops.dialog.is_some(),
            "bajarilayotganda Esc oynani yopmaydi"
        );
        reply(
            &mut s,
            request,
            Ok(Response::Executed(OperationReport {
                action: PlanAction::Trash,
                directory: "/ish".into(),
                done: vec![],
                failed: vec![ItemFailure {
                    name: "b.txt".into(),
                    error: ErrorCode::TrashUnavailable.into(),
                }],
                skipped: vec![],
                target: None,
                cancelled: false,
            })),
        );
        assert!(s.ops.dialog.is_none());
        let notice = s.opening.notice.clone().expect("notice");
        assert_eq!(
            notice.kind,
            NoticeKind::Operation {
                failed: 1,
                total: 1
            }
        );
        assert_eq!(notice.error.code, ErrorCode::TrashUnavailable);
    }

    #[test]
    fn cancelling_a_confirmation_discards_the_plan() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.start_trash(&rec);
        let (request, _) = last(&rec);
        reply(
            &mut s,
            request,
            Ok(Response::Plan(plan(PlanAction::Trash, &["a.txt"], true))),
        );
        s.cancel_op(&rec);
        assert_eq!(
            last(&rec).1,
            Command::DiscardPlan {
                plan: PlanId::from_raw(77)
            }
        );
        assert!(s.ops.dialog.is_none());
        let before = s.clone();
        reply(&mut s, RequestId::from_raw(1), Ok(Response::Discarded));
        assert_eq!(s.ops, before.ops, "Discarded javobi holatga tegmaydi");
    }

    fn conflicted_plan(policy: ConflictPolicy) -> OperationPlan {
        let mut plan = plan(PlanAction::Copy, &["a.txt"], true);
        plan.items[0].conflict = true;
        plan.policy = policy;
        plan.target = Some("/boshqa".into());
        plan.totals = Some(Totals {
            items: 1,
            bytes: 200,
        });
        plan
    }

    #[test]
    fn copy_targets_other_panel_and_conflicts_need_a_policy() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.run_command(&rec, PaletteCommand::Copy);
        let (request, command) = last(&rec);
        assert_eq!(
            command,
            Command::CreatePlan(PlanRequest::Copy {
                location: LocationId::from_raw(9),
                entries: vec![EntryId::from_raw(1)],
                target: LocationId::from_raw(9),
                policy: ConflictPolicy::Ask,
            })
        );
        reply(
            &mut s,
            request,
            Ok(Response::Plan(conflicted_plan(ConflictPolicy::Ask))),
        );
        let Some(OpDialog::Confirm(dialog)) = &s.ops.dialog else {
            panic!("dialog");
        };
        assert_eq!(dialog.conflicts(), 1);
        assert!(!dialog.can_confirm(), "siyosat tanlanmaguncha tasdiq yo‘q");
        let sent = rec.0.borrow().len();
        s.confirm_plan(&rec);
        assert_eq!(rec.0.borrow().len(), sent);

        s.choose_policy(&rec, ConflictPolicy::KeepBoth);
        let log = rec.0.borrow().clone();
        assert_eq!(
            log[log.len() - 2],
            Command::DiscardPlan {
                plan: PlanId::from_raw(77)
            },
            "eski reja unutiladi"
        );
        let (request, command) = last(&rec);
        assert!(matches!(
            command,
            Command::CreatePlan(PlanRequest::Copy {
                policy: ConflictPolicy::KeepBoth,
                ..
            })
        ));
        s.cancel_op(&rec);
        assert!(
            s.ops.dialog.is_some(),
            "qayta reja kutilayotganda Esc yopmaydi"
        );
        reply(
            &mut s,
            request,
            Ok(Response::Plan(conflicted_plan(ConflictPolicy::KeepBoth))),
        );
        let Some(OpDialog::Confirm(dialog)) = &s.ops.dialog else {
            panic!("dialog");
        };
        assert!(dialog.can_confirm());
    }

    #[test]
    fn copy_job_reports_progress_can_be_cancelled_and_refreshes_both_folders() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.start_transfer(&rec, PlanAction::Copy);
        let (request, _) = last(&rec);
        let mut plan = conflicted_plan(ConflictPolicy::Skip);
        plan.items[0].conflict = false;
        reply(&mut s, request, Ok(Response::Plan(plan)));
        s.confirm_plan(&rec);
        let (request, command) = last(&rec);
        assert!(matches!(command, Command::ExecutePlan { .. }));
        let job = JobId::new();
        reply(&mut s, request, Ok(Response::JobStarted(job)));
        assert!(s.ops.dialog.is_none(), "vazifa fonda, oyna yopiladi");
        assert_eq!(s.ops.jobs.len(), 1);
        assert_eq!(s.ops.jobs[0].fraction(), None);

        s.apply(Event::JobProgress(JobProgress {
            job,
            action: PlanAction::Copy,
            done_items: 0,
            done_bytes: 50,
            totals: Some(Totals {
                items: 1,
                bytes: 200,
            }),
            current: "a.txt".into(),
        }));
        assert_eq!(s.ops.jobs[0].fraction(), Some(0.25));
        s.apply(Event::JobProgress(JobProgress {
            job: JobId::new(),
            action: PlanAction::Copy,
            done_items: 9,
            done_bytes: 9,
            totals: None,
            current: String::new(),
        }));
        assert_eq!(
            s.ops.jobs[0].fraction(),
            Some(0.25),
            "begona vazifa hisobga olinmaydi"
        );

        s.cancel_job(&rec, job);
        assert_eq!(last(&rec).1, Command::CancelJob { job });
        assert!(s.ops.jobs[0].cancelling);
        let sent = rec.0.borrow().len();
        s.cancel_job(&rec, job);
        assert_eq!(rec.0.borrow().len(), sent, "ikki marta yuborilmaydi");

        // Boshqa panel boshqa papkada: maqsad shu panelda yangilanadi.
        let request = s.panel(Side::Right).pending;
        assert!(request.is_none());
        s.panel_mut(Side::Right)
            .listing
            .as_mut()
            .expect("listing")
            .location
            .display_path = "/boshqa".into();
        s.apply(Event::JobFinished {
            job,
            result: Ok(OperationReport {
                action: PlanAction::Copy,
                directory: "/ish".into(),
                done: vec![],
                failed: vec![],
                skipped: vec![],
                target: Some("/boshqa".into()),
                cancelled: true,
            }),
        });
        assert!(s.ops.jobs.is_empty());
        assert_eq!(
            s.opening.notice.as_ref().map(|n| n.kind),
            Some(NoticeKind::Cancelled { done: 0 })
        );
        s.pump(&rec);
        assert!(s.panel(Side::Left).pending.is_some(), "manba yangilandi");
        assert!(s.panel(Side::Right).pending.is_some(), "maqsad yangilandi");
    }

    #[test]
    fn move_without_other_listing_or_targets_does_nothing() {
        let rec = Recorder(RefCell::new(Vec::new()));
        let mut s = ready(&rec);
        let sent = rec.0.borrow().len();
        s.start_transfer(&rec, PlanAction::Move);
        assert_eq!(rec.0.borrow().len(), sent, "kursor/tanlov yo‘q");
        s.panel_mut(Side::Left).cursor = Some(EntryId::from_raw(1));
        s.panel_mut(Side::Right).listing = None;
        s.start_transfer(&rec, PlanAction::Move);
        assert_eq!(rec.0.borrow().len(), sent, "maqsad panel tayyor emas");
        s.start_transfer(&rec, PlanAction::Trash);
        assert_eq!(rec.0.borrow().len(), sent, "faqat copy/move");
    }
}
