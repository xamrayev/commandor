# Commandor — o‘zbekcha (asosiy til)

app-title = Commandor
panel-left = A panel
panel-right = B panel
panel-active = faol
panel-placeholder = Fayl ro‘yxati keyingi bosqichda (P3-01) qo‘shiladi.
hint-tab = Tab — faol panelni almashtirish
status-broker-connecting = Broker bilan ulanmoqda…
status-broker-ready = Tayyor · v{ $version } · { $os }/{ $arch }
status-broker-error = Xato: { $error }
status-broker-stopped = Broker to‘xtagan
fkey-view = F3 Ko‘rish
fkey-edit = F4 Tahrirlash
fkey-copy = F5 Nusxa
fkey-move = F6 Ko‘chirish
fkey-mkdir = F7 Papka
fkey-trash = F8 Savat
fkey-unavailable = Bu amal hali mavjud emas

error-permission-denied = Ruxsat yo‘q. Yo‘l va kirish huquqini tekshiring.
error-root-unavailable = Disk yoki papka mavjud emas yoki uzilgan.
error-not-found = Fayl yoki papka topilmadi.
error-stale-reference = Ma’lumot eskirgan. Panelni yangilang.
error-conflict = Fayl boshqa joyda o‘zgargan. Saqlash to‘xtatildi.
error-destination-exists = Maqsad joyda bunday nom mavjud.
error-unsupported-format = Bu format qo‘llab-quvvatlanmaydi.
error-unsupported-encoding = Matn kodirovkasi aniqlanmadi yoki qo‘llab-quvvatlanmaydi.
error-password-required = Fayl parol bilan himoyalangan.
error-engine-not-installed = Kerakli dvigatel o‘rnatilmagan.
error-engine-crashed = Dvigatel kutilmaganda to‘xtadi. Asl fayl saqlandi.
error-model-unsupported = Model ushbu runtime bilan mos emas.
error-out-of-memory = Xotira yetmadi. Kichikroq model yoki context tanlang.
error-checksum-mismatch = Fayl nazorat summasi mos kelmadi; o‘rnatilmadi.
error-download-interrupted = Yuklash uzildi.
error-provider-unavailable = AI provayderi javob bermayapti.
error-rate-limited = So‘rovlar chegarasi oshdi. Keyinroq urinib ko‘ring.
error-policy-blocked = Amal xavfsizlik siyosati bilan bloklandi.
error-plan-expired = Amal rejasi eskirdi. Yangi reja tuzing.
error-disk-full = Diskda joy qolmadi.
error-cancelled = Bekor qilindi.
error-invalid-argument = Noto‘g‘ri so‘rov.
error-unsupported-api-version = Ilova qismlari versiyasi mos emas.
error-internal = Ichki xato yuz berdi.

# P3-01 — listing va navigatsiya
nav-home = Uy papkasi
nav-up = Yuqoriga
nav-refresh = Yangilash
nav-path = To‘liq papka yo‘li
nav-open = Ochish
list-filter = Nom bo‘yicha filtr
list-apply-filter = Filtrlash
list-hidden = Yashirin fayllar
list-loading = Papka o‘qilmoqda…
list-empty = Papka bo‘sh yoki filtrga mos yozuv yo‘q.
list-retry = Qayta urinish
list-count = { $count } ta yozuv
list-truncated = Ro‘yxat cheklangan. Nom filtri yoki kichikroq papkadan foydalaning.
list-skipped = O‘qilmagan yozuvlar: { $count }
list-symlink = Ramziy havola; uni kuzatish hozircha o‘chiq.
list-directory = Papka


# P3-01b — metadata va picker
nav-pick-folder = Papka tanlash…
list-col-name = Nom
list-col-size = Hajm
list-col-modified = O'zgartirilgan

# P3-02 — tarix, saralash va multi-select
nav-back = Orqaga
nav-forward = Oldinga
list-selected = Tanlangan: { $count } ta ({ $size })

# P3-02b — tablar, xatcho'plar va buyruq paneli
nav-bookmarks = Xatcho‘plar
bookmark-add-current = Joriy papkani saqlash
tab-new = Yangi tab
tab-close = Tabni yopish
palette-title = Buyruqlar paneli
palette-hint = Buyruq nomini kiriting…
palette-empty = Mos buyruq topilmadi
cmd-go-home = Uy papkasiga o‘tish
cmd-go-up = Yuqoridagi papkaga chiqish
cmd-go-back = Orqaga qaytish
cmd-go-forward = Oldinga o‘tish
cmd-refresh = Yangilash
cmd-new-tab = Yangi tab ochish
cmd-close-tab = Faol tabni yopish
cmd-toggle-panel = Faol panelni almashtirish (Tab)
cmd-toggle-hidden = Yashirin fayllarni ko‘rsatish/yashirish
cmd-sort-name = Nom bo‘yicha saralash
cmd-sort-size = Hajm bo‘yicha saralash
cmd-sort-modified = Sana bo‘yicha saralash
cmd-select-all = Barcha fayllarni tanlash
cmd-deselect-all = Tanlovni bekor qilish
cmd-pick-folder = Tizim dialogi orqali papka ochish
cmd-add-bookmark = Joriy papkani xatcho‘plarga qo‘shish

# P3-08 — tomlar (disklar) va F1/F2
volumes-title = Tomlar
volumes-open = Tomlar ro‘yxati
volumes-loading = Tomlar o‘qilmoqda…
volumes-empty = Tom topilmadi
volumes-free = { $free } bo‘sh / { $total }
volumes-read-only = faqat o‘qish
volumes-hint-select = tanlash
volumes-hint-open = ochish
volumes-hint-close = yopish
volume-kind-system = Tizim
volume-kind-internal = Ichki
volume-kind-removable = Olinadigan
volume-kind-network = Tarmoq
volume-kind-other = Boshqa
fkey-volumes-left = F1 A tomlari
fkey-volumes-right = F2 B tomlari

# P3-02d — tablar, xatcho‘plar va buyruq paneli (UI)
bookmarks-empty = Xatcho‘p yo‘q. Joriy papkani quyidagi tugma bilan saqlang.
bookmark-remove = Xatcho‘pni o‘chirish
bookmark-already = Joriy papka saqlangan
bookmarks-session-note = Xatcho‘plar sozlamalar faylida saqlanadi va keyingi ishga tushishda tiklanadi.
palette-hint-run = bajarish

# P3-09 — tizim dasturida ochish va "Open with"
error-no-application = Bu fayl turini ochadigan dastur topilmadi.
cmd-open = Ochish (papka yoki tizim dasturi)
cmd-open-with = Dastur bilan ochish…
menu-open = Ochish
menu-open-with = Dastur bilan ochish…
open-with-title = «{ $name }» — dastur bilan ochish
open-with-search = Dastur nomi…
open-with-loading = Dasturlar ro‘yxati o‘qilmoqda…
open-with-empty = Mos dastur topilmadi
open-with-recommended = Tavsiya etilgan
open-with-others = Boshqa dasturlar
open-with-show-all = Barcha dasturlar ({ $count })
open-with-show-less = Faqat tavsiya etilganlar
open-with-system = Tizim dialogi…
open-with-default = standart
open-hint-open = ochish
confirm-exec-title = Bajariladigan fayl
confirm-exec-body = «{ $name }» dastur yoki skript bo‘lishi mumkin. Uni ochish undagi kodni ishga tushiradi. Davom etasizmi?
confirm-exec-run = Ishga tushirish
confirm-exec-cancel = Bekor qilish
notice-open-failed = «{ $name }» ochilmadi: { $error }
notice-dismiss = Yopish

# P3-04a — yangi papka, rename, savat
error-invalid-name = Bu nom ushbu tizimda ruxsat etilmagan.
error-trash-unavailable = Savat mavjud emas yoki ishlamadi; fayl o‘chirilmadi.
cmd-new-folder = Yangi papka yaratish
cmd-rename = Nomini o‘zgartirish
cmd-trash = Savatga tashlash
op-new-folder-title = Yangi papka
op-new-folder-hint = Papka nomi
op-rename-title = «{ $name }» nomini o‘zgartirish
op-rename-hint = Yangi nom
op-create = Yaratish
op-rename = O‘zgartirish
op-cancel = Bekor qilish
op-in = Papka: { $dir }
op-working = Bajarilmoqda…
op-trash-title = Savatga tashlash
op-trash-body = { $count } ta yozuv ({ $size }) tizim savatiga tashlanadi. Ularni savatdan qaytarish mumkin.
op-trash-more = … va yana { $count } ta
op-trash-confirm = Savatga tashlash
notice-op-failed = Amal bajarilmadi ({ $failed }/{ $total }){ $name }: { $error }

# P3-05a — nusxalash, ko‘chirish, konfliktlar, fon vazifalari
cmd-copy = Boshqa panelga nusxalash
cmd-move = Boshqa panelga ko‘chirish
op-copy-title = Nusxalash
op-move-title = Ko‘chirish
op-transfer-body = { $count } ta yozuv, jami { $size }.
op-to = Qayerga: { $dir }
op-copy-confirm = Nusxalash
op-move-confirm = Ko‘chirish
op-conflicts = Maqsadda shu nomli { $count } ta yozuv bor. Nima qilinsin?
op-choose-policy = Davom etish uchun variantni tanlang.
policy-skip = O‘tkazib yuborish (mavjudlari tegilmaydi)
policy-keep-both = Ikkalasini saqlash (yangisiga “(2)” qo‘shiladi)
policy-replace = Almashtirish (fayllar almashadi, papkalar birlashtiriladi)
job-copying = Nusxalanmoqda: «{ $name }»{ $more }
job-moving = Ko‘chirilmoqda: «{ $name }»{ $more }
job-cancelling = Bekor qilinmoqda: «{ $name }»{ $more }
job-cancel = Vazifani bekor qilish
job-queued = {" "}· navbatda yana { $count } ta
notice-op-cancelled = Amal bekor qilindi: { $done } ta yozuv bajarildi, qolganlariga tegilmadi.
error-reason-empty = Nom bo‘sh bo‘lishi mumkin emas.
error-reason-dot = “.” va “..” nom sifatida ishlatilmaydi.
error-reason-too-long = Nom juda uzun (255 baytdan oshmasin).
error-reason-character = Nomda ruxsat etilmagan belgi bor.
error-reason-trailing = Nom probel yoki nuqta bilan tugamasin.
error-reason-reserved = Bu nom tizim uchun band (CON, NUL, COM1 …).
error-reason-unchanged = Yangi nom eskisi bilan bir xil.
error-reason-same-directory = Yozuv allaqachon shu papkada.
error-reason-into-itself = Papkani o‘zining ichiga nusxalab yoki ko‘chirib bo‘lmaydi.

# P3-07a — sozlamalar va sessiya
notice-settings = Sozlamalar: { $error }
error-reason-settings-invalid = sozlamalar fayli o‘qilmadi, sukut qiymatlar ishlatildi. Asl fayl settings.toml.broken nomi bilan saqlandi.
error-reason-settings-too-large = sozlamalar fayli juda katta (1 MiB dan ortiq), sukut qiymatlar ishlatildi.
