# Commandor — interfeyslar va ma’lumotlar modeli (Rust)

Versiya: 2.1 · 2026-11-07. Bu hujjat maqsadli implementatsiya shartnomasi; 1-bo‘lim va 8-bo‘lim xato kodlari kodda mavjud, qolgan metodlar keyingi bosqichlarda. Turlar `commandor-core` crate’ida, servis metodlari `commandor-broker` fasadi orqali amalga oshiriladi ([arxitektura](ARCHITECTURE.uz.md#jarayonlar-modeli)). Bajarilish holati [TODO](../TODO.md)da. [Texnik topshiriq](TECHNICAL-SPEC.uz.md) bilan birga o‘qiladi.

## 1. Umumiy kontrakt

UI → `AppHandle` (broker) → servis. UI faqat typed `Command` enum variantlarini yuboradi; erkin “kanal + istalgan argument” ko‘prigi yo‘q. Har command `RequestId` oladi; uzoq ishda `JobId`, o‘zgartiruvchi amalda `IdempotencyKey`. Natijalar va progress `Event` kanali orqali keladi. Quyidagi jadvallardagi `fs.list` kabi nomlar mantiqiy metod nomi; Rustda ular `Command::FsList { .. }` va `Event::FsListBatch { .. }` ko‘rinishida bo‘ladi.

```rust
// commandor-core — amaldagi shakl (crates/commandor-core/src/protocol.rs, error.rs, revision.rs)
pub const API_VERSION: u32 = 1;

pub struct RequestId(u64);              // opaque; broker monoton beradi
pub struct JobId(Uuid);

pub struct Envelope<T> { pub request: RequestId, pub api_version: u32, pub body: T }

#[non_exhaustive]
pub enum Command {
    Ping,
    GetAppInfo,
    ReadDirectory { target: DirectoryTarget, options: ListOptions },   // file thread
    ListVolumes,                                                       // file thread
    // Fayl listing granti orqali aniqlanadi (UI yo‘l yubormaydi); ish launcher threadida.
    OpenEntry { location: LocationId, entry: EntryId, with: OpenWith, allow_executable: bool },
    ListOpenWith { location: LocationId, entry: EntryId },
    // Reja → tasdiq → bajarish (2.1-bo‘lim): CreateDirectory / Rename / Trash / Copy / Move.
    // Copy/Move: `policy: ConflictPolicy { Ask, Skip, KeepBoth, Replace }`; Ask + konflikt — bajarilmaydi.
    CreatePlan(PlanRequest),                // disk o‘zgarmaydi; broker nomni OS qoidasi bilan tekshiradi
    ExecutePlan { plan: PlanId, hash: u64 },// bir marta; xesh, muddat (10 min), grant va inode qayta tekshiriladi
    DiscardPlan { plan: PlanId },
    CancelJob { job: JobId },               // fon vazifasi; .part fayl o‘chiriladi, qolganiga tegilmaydi
    LoadSettings,                           // yo‘q/buzilgan fayl — sukut + problem (UX-09)
    SaveSettings(Box<Settings>),            // atomik: .tmp → sync_all → rename
}

pub enum OpenWith { Default, App(AppId), SystemChooser }   // AppId — oxirgi ListOpenWith doirasida
pub enum OpenOutcome { Launched, NeedsConfirmation }      // bajariladigan fayl tasdiqsiz ishga tushmaydi

#[non_exhaustive]
pub enum Response {
    Pong, AppInfo(AppInfo), Directory(DirectoryListing), Volumes(Vec<Volume>),
    Opened(OpenOutcome), OpenWithApps(OpenWithList),
    Plan(OperationPlan), Executed(OperationReport), Discarded,   // report: done/failed/skipped/cancelled
    JobStarted(JobId), CancelRequested,     // Copy/Move ExecutePlan javobi
    Settings(Box<SettingsLoad>), SettingsSaved,
}

pub type Reply = Result<Response, AppError>;

#[non_exhaustive]
pub enum Event {
    Reply { request: RequestId, reply: Reply },
    JobProgress(JobProgress),               // ≤ 10/s: done_items/bytes, totals, current
    JobFinished { job: JobId, result: Result<OperationReport, AppError> },
}

pub struct AppError {
    pub code: ErrorCode,                   // #[non_exhaustive]; 8-bo‘lim
    pub retryable: bool,                   // sukut: ErrorCode::default_retryable()
    pub details: BTreeMap<String, DetailValue>, // yo‘l/stack/maxfiy ma’lumotsiz
}
// Tarjima kaliti: ErrorCode::message_key() → "error-..." (resources/i18n/*/main.ftl).

pub struct Revision {
    pub content_sha256: Option<[u8; 32]>,  // mutatsiya uchun snapshotdan olinadi
    pub size_bytes: u64,
    pub modified_ns: i128,                 // aniqlik yo‘qolmaydi
    pub file_identity: Option<FileIdentity>, // (volume, file id) mavjud bo‘lsa
}
```

UI `AppHandle::dispatch(Command) -> Option<RequestId>` chaqiradi (broker to‘xtagan bo‘lsa `None`), javobni `EventReceiver::drain()` orqali kadr ichida oladi. UI faqat o‘zi kutayotgan `RequestId` javobini qabul qiladi; eskirgani e’tiborsiz.

Servis xatolari ichki `thiserror` turlari bilan ifodalanadi va broker chegarasida `AppError`ga xaritalanadi; `std::io::Error` matni yoki stack UIga xom uzatilmaydi. UI arbitrary path uzatish o‘rniga broker bergan `EntryId`/`DocumentId` va grant ishlatadi. Path kiritish, picker va drive ochish broker orqali yangi grant yaratadi. AI tool doirasi foydalanuvchining joriy grantidan oshmaydi.

Katta ma’lumot (`Vec<u8>` sahifa rasmi, uzun matn) `Event` ichida nusxalanmaydi: `Arc<[u8]>`, chunklangan oqim yoki cache faylga token beriladi. Worker jarayoni bilan protokol ([commandor-ipc](../crates/commandor-ipc/src/lib.rs)): 4 bayt LE uzunlik + JSON freym (≤ 16 MiB), boshida `Hello { protocol_version, kind }`, so‘rovlar `Request { id, .. }`/`Response { id, .. }`; noma’lum versiya yoki maydon (`deny_unknown_fields`) rad etiladi. Supervisor crash, timeout va restart limitini boshqaradi. Server/runtime tokeni UI holatida saqlanmaydi.

## 2. Fayl servisi

| Metod | Kirish | Chiqish va semantika |
|---|---|---|
| `fs.resolveLocation` | User path yoki OS picker natijasi | Canonical location va access grant; OS ruxsati tekshiriladi |
| `fs.list` | locationId, cursor, sort, hidden, filter | Entry batch + nextCursor + listingRevision; oqimli metadata |
| `fs.watch` / `fs.unwatch` | locationId / subscriptionId | Qo‘shildi/o‘zgardi/o‘chdi/overflow eventlari |
| `fs.createPlan` | copy/move/rename/mkdir/trash, source IDs, target, conflict policy | Canonical, brokerga tegishli immutable plan |
| `fs.executePlan` | planId, approvedPlanHash, idempotencyKey | jobId; revision va ruxsatlar qayta tekshiriladi |
| `fs.cancelJob` | jobId | Cancellation acknowledged; keyin cleanup natijasi |
| `fs.readText` | entryId, encoding?, range? | Matn/parcha, encoding/BOM/EOL, revision, truncated |
| `fs.saveText` | entryId, baseRevision, text, encoding/EOL | Yangi revision yoki conflict; lock/stage/commit orqali |
| `fs.saveAs` | document session, picker orqali maqsad | Mustaqil fayl; mavjud target bo‘lsa alohida replace policy |
| `fs.getOperation` | operationId | Per-item status, error, rollback/undo imkoniyati |

`FileEntry`: id, name (`OsString` + ko‘rsatish uchun lossy matn), kind, path display, directory/symlink flags, size, modified, permissions summary, volumeId. Symlink targetni resolver alohida tekshiradi. Directory listingda og‘ir content hashing qilinmaydi. Cursor eski listing revisionga tegishli bo‘lsa qayta list talabi beriladi.

### 2.1. Amal rejasi

```json
{
  "planId": "broker-generated-id",
  "origin": "ai",
  "action": "copy",
  "items": [{"sourceId": "entry-1", "baseRevision": "snapshot-reference", "destinationName": "hisobot.docx"}],
  "targetDirectoryId": "directory-2",
  "conflictPolicy": "ask",
  "expiresAt": "2026-11-06T12:00:00Z",
  "planHash": "broker-computed-hash",
  "requiresApproval": true
}
```

Bu misoldagi ID/hashlar placeholder; haqiqiy xesh canonical serializatsiya va barcha ta’sir qiladigan maydonlarni qamraydi. Model planId/hash/approval yarata olmaydi. `executePlan` foydalanuvchining tasdiq harakatidan keladi. Manba yoki maqsad o‘zgarsa plan bekor bo‘ladi va yangi diff ko‘rsatiladi. Idempotency bir xil so‘rovning ikki marta nusxalashiga yo‘l qo‘ymaydi; qayta urinish natijani jurnaldan oladi.

### 2.2. Save protokoli

1. Session va ruxsatni tekshirish; actual path/file identityni qayta resolve qilish.
2. Base revision bilan solishtirish; conflict bo‘lsa yozishni boshlamaslik.
3. Shu filesystemdagi vaqtinchalik faylga yozish; kerakli mode/metadata siyosatini qo‘llash.
4. Flush/fsync, bytes/hash tekshiruvi; originalni qayta tekshirish va qo‘llab-quvvatlansa platforma locki.
5. Backup/recovery yozuvi, atomik replace yoki platformaga mos xavfsiz commit.
6. Parent directory durability imkoni bo‘lsa sync; jurnalni yakunlash; watcher/preview/index invalidation.

Tashqi dasturlarning hammasi lockka rioya qilmaydi. Shuning uchun “barcha tashqi yozuvchilarga qarshi mutlaq atomik compare-and-swap” kafolati berilmaydi; OS/file system cheklovi, backup va Save As yo‘li hujjatlashtiriladi. Hardlink uchun replace inode aloqasini uzishi mumkin: sukut bo‘yicha read-only yoki aniq tushuntirilgan alohida siyosat. ACL/xattr/owner saqlanishi uch OS fixturelarida tekshiriladi; jim metadata yo‘qotishga yo‘l qo‘yilmaydi.

## 3. Hujjat va Office adapterlari

| Metod | Asosiy maydonlar | Semantika |
|---|---|---|
| `document.inspect` | entryId | MIME/format/capabilities; extension va signature mosligi |
| `preview.open` | documentId, revision, options | previewSessionId, cached/jobId, sahifa soni mavjud bo‘lsa |
| `preview.getPage` | sessionId, page, scale | Image/token yoki PDF range; bounds va quota |
| `preview.close` | sessionId | Refcount kamayadi, faol bo‘lmagan resurslar bo‘shaydi |
| `edit.begin` | documentId, baseRevision, adapter | Working copy/session; boshlang‘ich view → editor handoff |
| `edit.status` | editSessionId | loading/clean/dirty/saving/conflict/recovery |
| `edit.commit` | sessionId, savedWorkingRevision, baseRevision | Originalga commit yoki conflict |
| `edit.saveAs` | sessionId, target grant | Boshqa nusxaga commit |
| `edit.close` | sessionId, save/discard/cancel | Dirty sessionni jim yopmaydi |

`DocumentCapabilities`: preview, textExtraction, editMode (`none|native|embedded`), saveFormats, annotations, preservesMacros (`false` bo‘lishi mumkin), limitations. UI shunga ko‘ra tugmalarni yoqadi. Adapter ko‘rsatmagan imkoniyat UI tomonidan va’da qilinmaydi.

`OfficeRenderer` trait: `probe_runtime`, `render(snapshot, options, cancel)`, `extract(snapshot, cancel)`, `shutdown`. `OfficeEditor` trait: `open_working_copy`, `subscribe_changes`, `request_save`, `close`, `recover`. Implementatsiyalar: `NativeLibreOffice` (R1) va `lok` feature ortidagi `LibreOfficeKit` (R3 darvozasi). `PdfBackend` trait: `open`, `page_count`, `render_page(page, scale)`, `text(page)`, `search`; tahrir uchun alohida `PdfEditor` (`rotate`, `delete`, `reorder`, `annotate`, `save_as`). Har biri engine version va OS/arch metadata qaytaradi. Windows/macOS/Linux launch code bir adapter family ichida farqlanadi.

Preview va editor **ikkita mantiqiy servis**. Ular bitta LibreOffice distributivini ishlatishi mumkin, lekin bir xil user profil yoki o‘zaro boshqarilmagan jarayonni bo‘lishmasligi kerak. Original faylga to‘g‘ridan-to‘g‘ri yozuvchi external editor ishlatilsa boshqariladigan commit/recovery kafolatlari cheklanishi alohida ko‘rsatiladi; asosiy yo‘l working copydir.

## 4. Qidiruv va summary API

| Metod | Kirish | Chiqish |
|---|---|---|
| `index.addRoot` | directory grant, excludes, autoSummary=false | rootId, scanJobId |
| `index.updateRoot` | rootId, excludes, policy | Reconciliation job; o‘zgargan policy bo‘yicha purge |
| `index.removeRoot` | rootId, purgeDerivedData | Tranzaksion unregister/purge job |
| `index.scan` | rootId, incremental/full | jobId va avlod IDsi |
| `index.pause/resume/cancel` | jobId/rootId | Holat va checkpoint |
| `search.query` | query, root IDs, type/date/size filters, cursor | Ranked hits, source locator, stale/offline flags |
| `summary.generate` | documentId, revision, localProviderId, policy | jobId; model/version/coverage |
| `summary.get` | documentId, revision? | Ready/stale/partial/failed + provenance |

Qidiruv foydalanuvchi matnini SQL yoki xom FTS query sifatida to‘g‘ridan-to‘g‘ri bajarmaydi. Oddiy matn tokenlari escape qilinadi; advanced syntax alohida rejim. O‘zbekcha `o‘/oʻ/o'/ў` kabi variantlar uchun normalize maydoni va test korpusi belgilanadi; original matn o‘zgartirilmaydi. Stemming/semantik moslik FTSning mavjud bo‘lmagan imkoniyati sifatida da’vo qilinmaydi.

`SearchHit`: documentId, path, title, score, snippet, locator, matchSource (`name|content|summary`), currentRevision, indexedRevision, availability. Qidiruv natijasidan ochishda real fayl va ruxsat qayta tekshiriladi. Eskirgan summary asosida topilgan natija shunday belgilanadi.

`Summary`: text, language, documentRevision, modelRepo/modelId/revision yoki Ollama digest, quantization, runtimeVersion, promptVersion, extractorVersion, chunkPolicyVersion, generatedAt, coveredChunks/totalChunks, status, sourceLocators. Provenance noma’lum bo‘lsa “noma’lum” qiymati saqlanadi; aniq model commit uydirilmaydi.

## 5. AI provayder kontrakti

Provider adapter (`commandor-ai`):

```rust
pub enum Locality { ManagedLocal, UserLocal, Remote }

pub struct ProviderCapabilities {
    pub locality: Locality,
    pub chat: bool, pub streaming: bool, pub structured_output: bool,
    pub tool_calls: bool, pub embeddings: bool, pub vision: bool,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn probe(&self) -> Result<ProviderCapabilities, AiError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiError>;
    fn stream_chat(&self, req: ChatRequest, cancel: CancellationToken)
        -> BoxStream<'static, Result<ChatEvent, AiError>>;
    async fn generate_structured(&self, req: StructuredRequest, cancel: CancellationToken)
        -> Result<serde_json::Value, AiError>;
    // Qo‘llanmaydigan funksiya AiError::UnsupportedCapability qaytaradi.
}
```

Konfiguratsiya: providerId, type, baseURL, modelId, keyRef, timeouts, context/output budget, capabilities, lastProbe. UIga haqiqiy key qaytarilmaydi. Ollama yo‘llari `/api/chat` va model ro‘yxat API; compatible adapter base URLni `/v1` bilan normalizatsiya qiladi va chat yo‘lini bir marta qo‘shadi. APIga o‘xshash nom ulanishni tasdiqlamaydi: real probe kerak.

So‘rovlar system/user/assistant/tool rollariga alohida ajratiladi; hujjat matni system roliga qo‘shilmaydi. Modelga faylning faqat ruxsat etilgan kerakli fragmenti beriladi. Remote chatga attachment qo‘shilganda qaysi matn/yo‘l yuborilishi ko‘rinadi; avtomatik local summarydan remote chatga yashirin context ko‘chirilmaydi.

Streaming eventlar (`ChatEvent`): `Started`, `Token`, `ToolProposal`, `Usage`, `Completed`, `Cancelled`, `Failed`. Tool-call yoki JSON output ikki bosqichda tekshiriladi: parser/schema, keyin broker policy/path/revision. Model javobi `serde` bilan qat’iy turga (`deny_unknown_fields`) parse qilinadi va hech qachon SQL, shell yoki boshqa interpretatorga o‘tmaydi.

## 6. Model va runtime API

| Metod | Kirish | Natija |
|---|---|---|
| `models.recommend` | hardware profile, useCase, memory budget | Sababli nomzodlar, confidence/unknown flags |
| `models.resolveArtifact` | allowlisted repo, quantization | Pinned revision + file manifest + size/hash/license |
| `models.download` | resolvedManifestId, target store | Download job; user initiated |
| `models.cancel/resume` | downloadId | Checkpoint/validator holati |
| `models.import` | picker orqali GGUF | Metadata, validation va original/copy ownership |
| `models.remove` | installedModelId | Stop talab qilinishi yoki delete result |
| `runtime.start` | modelId, runtimeId, context, backend | loading → ready; local endpoint brokerda qoladi |
| `runtime.stop` | instanceId | stopping → stopped; faqat owned PID tree |
| `runtime.status` | instanceId | version, backend, model, memory, lastError |

Model holatlari: `discovered → resolving → downloadable → downloading → verifying → installed`; xatoda `failed`, cancelda `partial`. Runtime holatlari: `stopped → starting → loading → ready → stopping → stopped`; crash/OOMda `failed`. `installed` inference ishlashini anglatmaydi, `ready` alohida dalil talab qiladi.

Qwen katalog namunasi (placeholderlar yuklash oldidan API/manifestdan to‘ldiriladi):

```json
{
  "displayName": "Qwen3.5 2B · Q4_K_M",
  "sourceModel": "Qwen/Qwen3.5-2B",
  "ggufRepo": "unsloth/Qwen3.5-2B-GGUF",
  "filename": "Qwen3.5-2B-Q4_K_M.gguf",
  "pinnedCommit": "RESOLVE_BEFORE_DOWNLOAD",
  "sha256": "RESOLVE_BEFORE_DOWNLOAD",
  "sizeBytes": null,
  "quantization": "Q4_K_M",
  "licenseSource": "model-repository",
  "recommendedContext": 4096,
  "testedRuntimeBuild": null,
  "benchmarkStatus": "not-yet-tested"
}
```

Bu namunani to‘g‘ridan-to‘g‘ri install qilish rad etilishi kerak: commit/hash/size va runtime moslik tekshiruvi hal qilinmagan. Tavsiya algoritmi model sifati va xotira taxminini ajratadi; mahalliy o‘zbekcha benchmark bo‘lmasa sifat bahosi “tekshirilmagan”.

## 7. Saqlash modeli

| Entity | Muhim maydonlar va kalitlar |
|---|---|
| `schema_migrations` | version PK, appliedAt, checksum |
| `roots` | id PK, canonicalPath, volumeId, caseMode, excludes, policy, lastCompleteGeneration |
| `documents` | id PK, rootId FK, relativePath, fileIdentity, size, mtimeNs, currentHash, status; UNIQUE(rootId, relativePath) |
| `document_versions` | id PK, documentId FK, hash, size, extractorVersion, extractionStatus, createdAt |
| `chunks` | id PK, versionId FK, ordinal, locatorJSON, normalizedText, tokenCount; UNIQUE(versionId, ordinal) |
| `summaries` | id PK, versionId FK, modelFingerprint, promptVersion, chunkPolicyVersion, text, coverage, status, createdAt |
| `search_entries` | id PK, versionId FK, locator, path, content, summary; FTS uchun manba |
| `search_fts` | FTS5 path/content/summary; rowid orqali search_entriesga bog‘langan |
| `preview_cache` | cacheKey PK, versionId FK, engineVersion, fontFingerprint, artifactPath, bytes, accessedAt |
| `jobs` | id PK, kind, state, progress, checkpoint, retryCount, ownerId, errorCode |
| `operations` | id PK, planHash, idempotencyKey UNIQUE, state, createdAt, completedAt |
| `operation_items` | operationId FK, ordinal, source, target, revisionBefore/After, status, rollbackInfo |
| `providers` | id PK, adapter, baseURL, modelId, capabilitiesJSON, secretRef (`keyring` yozuvi); maxfiy key yo‘q |
| `models` | id PK, repo, commit, quantization, runtimeConstraints, installedAt |
| `model_files` | modelId FK, filename, sha256, bytes, localPath, ownership, verificationStatus |
| `settings` | key PK, valueJSON, schemaVersion; faqat non-secret qiymatlar |

`document_versions` o‘zgarmas snapshot/provenance yozuvi. UI uchun current version pointer `documents`da. Qidiruvning current holati faqat committed generation/versionga bog‘lanadi; yarim extractiondan FTS natija chiqmaydi. SQLite `rusqlite` (`bundled`, FTS5) orqali; `foreign_keys=ON`; tegishli ON DELETE CASCADE va FTS update/delete/rebuild sinovlari majburiy.

Overlap rootlar ruxsat etiladi, lekin bitta fayl ikki root orqali topilsa UI canonical file identity/path orqali deduplikatsiya qiladi. Rootni olib tashlash faqat uning yozuvlarini o‘chiradi; umumiy content artifact boshqa reference bo‘lsa qoladi. Oxirgi reference yo‘qolganda GC ishlaydi. Reconciliationda offline rootning fayllari deleted emas, offline bo‘ladi.

Local profile joylashuvi OS app-data konvensiyasi orqali olinadi (`directories` crate). Diskdagi kataloglar: `db/`, `previews/`, `models/`, `runtime/`, `sessions/`, `recovery/`, `logs/`. Recovery nusxalarida ham shaxsiy mazmun bor; retention va purge sozlamalari foydalanuvchiga ko‘rsatiladi. Oddiy SQLite mazmunni o‘z-o‘zidan shifrlamaydi; boshlang‘ich himoya OS hisob/ruxsatlari va foydalanuvchining disk shifrlashidir. Kuchaytirilgan encrypted cache alohida ADR talab qiladi.

## 8. Xatolar va kuzatuv

| Kod | UI harakati |
|---|---|
| `PERMISSION_DENIED` / `ROOT_UNAVAILABLE` | Yo‘l/ruxsat yoki diskni tekshirish; cheksiz retry yo‘q |
| `NOT_FOUND` / `STALE_REFERENCE` | Panel/natijani yangilash |
| `CONFLICT` / `DESTINATION_EXISTS` | Diff, skip/keep-both/Save As/explicit replace |
| `UNSUPPORTED_FORMAT` / `UNSUPPORTED_ENCODING` | Mos adapter/encoding tanlash yoki read-only |
| `PASSWORD_REQUIRED` | Sessiyaga parol; log yoki indeksga yozmaslik |
| `ENGINE_NOT_INSTALLED` / `ENGINE_CRASHED` | Runtime sozlash yoki worker retry; original saqlanadi |
| `MODEL_UNSUPPORTED` / `OUT_OF_MEMORY` | Runtime/model/context tanlash; remote fallback yo‘q |
| `CHECKSUM_MISMATCH` / `DOWNLOAD_INTERRUPTED` | Quarantine/retry/resume; installed emas |
| `PROVIDER_UNAVAILABLE` / `RATE_LIMITED` | Timeout/backoff/cancel; foydalanuvchiga sabab |
| `POLICY_BLOCKED` / `PLAN_EXPIRED` | Yangi reja va haqiqiy ruxsat doirasi |
| `INVALID_NAME` | Nom maydonida xato; oyna ochiq qoladi, foydalanuvchi tuzatadi |
| `TRASH_UNAVAILABLE` | Fayl joyida qoladi; doimiy o‘chirish avtomatik taklif qilinmaydi (FS-05) |
| `NO_APPLICATION` | Fayl turi uchun dastur yo‘q: "Dastur bilan ochish…" taklif qilinadi |
| `DISK_FULL` / `CANCELLED` | Recovery/partial holati, tozalash va retry |

Loglar (`tracing`): requestId/jobId, davomiylik, engine/version, status, sanalgan bytes; maxfiy kalitlar, raw promptlar va to‘liq fayl matni yo‘q. Xato stacki foydalanuvchi UIga xom uzatilmaydi. Diagnostic export oldidan yo‘llar va shaxsiy ma’lumotlar redaction qilinadi. Kuzatuv default lokal; telemetry alohida opt-in.

## 9. Manbalar

- [SQLite FTS5](https://sqlite.org/fts5.html) — full-text va content table integratsiyasi.
- [Ollama chat API](https://docs.ollama.com/api/chat) — native chat protokoli.
- [llama.cpp server](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) — runtime HTTP interfeysi.
- [Hugging Face Hub API](https://huggingface.co/docs/hub/api) — model metadata manbasi.

Kontrakt nomlari va job/revision sxemalari Commandor uchun taklif etilgan dizayndir, yuqoridagi mahsulotlarning tayyor APIlari deb talqin qilinmaydi.
