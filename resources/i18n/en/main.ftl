# Commandor — English

app-title = Commandor
panel-left = Panel A
panel-right = Panel B
panel-active = active
panel-placeholder = File listing arrives in the next stage (P3-01).
hint-tab = Tab — switch active panel
status-broker-connecting = Connecting to broker…
status-broker-ready = Ready · v{ $version } · { $os }/{ $arch }
status-broker-error = Error: { $error }
status-broker-stopped = Broker stopped
fkey-view = F3 View
fkey-edit = F4 Edit
fkey-copy = F5 Copy
fkey-move = F6 Move
fkey-mkdir = F7 Folder
fkey-trash = F8 Trash
fkey-unavailable = This action is not available yet

error-permission-denied = Permission denied. Check the path and access rights.
error-root-unavailable = The drive or folder is unavailable or disconnected.
error-not-found = File or folder not found.
error-stale-reference = Information is out of date. Refresh the panel.
error-conflict = The file changed elsewhere. Saving was stopped.
error-destination-exists = An item with this name already exists at the destination.
error-unsupported-format = This format is not supported.
error-unsupported-encoding = Text encoding could not be detected or is not supported.
error-password-required = The file is password protected.
error-engine-not-installed = The required engine is not installed.
error-engine-crashed = The engine stopped unexpectedly. The original file is intact.
error-model-unsupported = The model is not compatible with this runtime.
error-out-of-memory = Out of memory. Choose a smaller model or context.
error-checksum-mismatch = Checksum mismatch; not installed.
error-download-interrupted = Download interrupted.
error-provider-unavailable = The AI provider is not responding.
error-rate-limited = Rate limit reached. Try again later.
error-policy-blocked = The action was blocked by security policy.
error-plan-expired = The action plan has expired. Create a new plan.
error-disk-full = The disk is full.
error-cancelled = Cancelled.
error-invalid-argument = Invalid request.
error-unsupported-api-version = Application component versions do not match.
error-internal = An internal error occurred.

# P3-01 — listing va navigatsiya
nav-home = Home
nav-up = Up
nav-refresh = Refresh
nav-path = Absolute folder path
nav-open = Open
list-filter = Filter by name
list-apply-filter = Filter
list-hidden = Hidden files
list-loading = Reading folder…
list-empty = Empty folder or no matching entries.
list-retry = Retry
list-count = { $count } entries
list-truncated = Listing is limited. Use a name filter or a smaller folder.
list-skipped = Unreadable entries: { $count }
list-symlink = Symbolic link; following links is currently disabled.
list-directory = Folder


# P3-01b — metadata va picker
nav-pick-folder = Pick folder…
list-col-name = Name
list-col-size = Size
list-col-modified = Modified

# P3-02 — tarix, saralash va multi-select
nav-back = Back
nav-forward = Forward
list-selected = Selected: { $count } ({ $size })

# P3-02b — tablar, xatcho'plar va buyruq paneli
nav-bookmarks = Bookmarks
bookmark-add-current = Bookmark current folder
tab-new = New tab
tab-close = Close tab
palette-title = Command Palette
palette-hint = Type a command…
palette-empty = No matching commands
cmd-go-home = Go to Home folder
cmd-go-up = Go to parent folder
cmd-go-back = Go back
cmd-go-forward = Go forward
cmd-refresh = Refresh
cmd-new-tab = Open new tab
cmd-close-tab = Close active tab
cmd-toggle-panel = Switch active panel (Tab)
cmd-toggle-hidden = Toggle hidden files
cmd-sort-name = Sort by Name
cmd-sort-size = Sort by Size
cmd-sort-modified = Sort by Date
cmd-select-all = Select all files
cmd-deselect-all = Deselect all
cmd-pick-folder = Open folder via system dialog
cmd-add-bookmark = Add current folder to bookmarks

# P3-08 — tomlar (disklar) va F1/F2
volumes-title = Volumes
volumes-open = Volume list
volumes-loading = Reading volumes…
volumes-empty = No volumes found
volumes-free = { $free } free of { $total }
volumes-read-only = read-only
volumes-hint-select = select
volumes-hint-open = open
volumes-hint-close = close
volume-kind-system = System
volume-kind-internal = Internal
volume-kind-removable = Removable
volume-kind-network = Network
volume-kind-other = Other
fkey-volumes-left = F1 Volumes A
fkey-volumes-right = F2 Volumes B

# P3-02d — tabs, bookmarks and command palette (UI)
bookmarks-empty = No bookmarks. Save the current folder with the button below.
bookmark-remove = Remove bookmark
bookmark-already = Current folder is saved
bookmarks-session-note = Bookmarks are kept for this session only for now.
palette-hint-run = run

# P3-09 — open with the system default app and "Open with"
error-no-application = No application can open this file type.
cmd-open = Open (folder or system app)
cmd-open-with = Open with…
menu-open = Open
menu-open-with = Open with…
open-with-title = Open “{ $name }” with
open-with-search = Application name…
open-with-loading = Reading applications…
open-with-empty = No matching application
open-with-recommended = Recommended
open-with-others = Other applications
open-with-show-all = All applications ({ $count })
open-with-show-less = Recommended only
open-with-system = System dialog…
open-with-default = default
open-hint-open = open
confirm-exec-title = Executable file
confirm-exec-body = “{ $name }” may be a program or script. Opening it runs its code. Continue?
confirm-exec-run = Run
confirm-exec-cancel = Cancel
notice-open-failed = Could not open “{ $name }”: { $error }
notice-dismiss = Dismiss

# P3-04a — new folder, rename, trash
error-invalid-name = This name is not allowed on this system.
error-trash-unavailable = The trash is unavailable or failed; nothing was deleted.
cmd-new-folder = Create new folder
cmd-rename = Rename
cmd-trash = Move to trash
op-new-folder-title = New folder
op-new-folder-hint = Folder name
op-rename-title = Rename “{ $name }”
op-rename-hint = New name
op-create = Create
op-rename = Rename
op-cancel = Cancel
op-in = Folder: { $dir }
op-working = Working…
op-trash-title = Move to trash
op-trash-body = { $count } item(s) ({ $size }) will be moved to the system trash. They can be restored from there.
op-trash-more = … and { $count } more
op-trash-confirm = Move to trash
notice-op-failed = Operation failed ({ $failed }/{ $total }){ $name }: { $error }

# P3-05a — copy, move, conflicts, background jobs
cmd-copy = Copy to other panel
cmd-move = Move to other panel
op-copy-title = Copy
op-move-title = Move
op-transfer-body = { $count } item(s), { $size } in total.
op-to = To: { $dir }
op-copy-confirm = Copy
op-move-confirm = Move
op-conflicts = { $count } item(s) with the same name already exist at the target. What should happen?
op-choose-policy = Choose an option to continue.
policy-skip = Skip (existing items are not touched)
policy-keep-both = Keep both (“(2)” is added to the new one)
policy-replace = Replace (files are replaced, folders are merged)
job-copying = Copying “{ $name }”{ $more }
job-moving = Moving “{ $name }”{ $more }
job-cancelling = Cancelling “{ $name }”{ $more }
job-cancel = Cancel job
job-queued = {" "}· { $count } more queued
notice-op-cancelled = Operation cancelled: { $done } item(s) done, the rest were not touched.
error-reason-empty = The name cannot be empty.
error-reason-dot = “.” and “..” cannot be used as names.
error-reason-too-long = The name is too long (max 255 bytes).
error-reason-character = The name contains a character that is not allowed.
error-reason-trailing = The name must not end with a space or a dot.
error-reason-reserved = This name is reserved by the system (CON, NUL, COM1 …).
error-reason-unchanged = The new name is the same as the old one.
error-reason-same-directory = The item is already in this folder.
error-reason-into-itself = A folder cannot be copied or moved into itself.
