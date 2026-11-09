# Commandor — русский

app-title = Commandor
panel-left = Панель A
panel-right = Панель B
panel-active = активна
panel-placeholder = Список файлов появится на следующем этапе (P3-01).
hint-tab = Tab — переключить активную панель
status-broker-connecting = Подключение к брокеру…
status-broker-ready = Готово · v{ $version } · { $os }/{ $arch }
status-broker-error = Ошибка: { $error }
status-broker-stopped = Брокер остановлен
fkey-view = F3 Просмотр
fkey-edit = F4 Правка
fkey-copy = F5 Копия
fkey-move = F6 Перенос
fkey-mkdir = F7 Папка
fkey-trash = F8 Корзина
fkey-unavailable = Это действие пока недоступно

error-permission-denied = Нет доступа. Проверьте путь и права.
error-root-unavailable = Диск или папка недоступны или отключены.
error-not-found = Файл или папка не найдены.
error-stale-reference = Данные устарели. Обновите панель.
error-conflict = Файл изменён в другом месте. Сохранение остановлено.
error-destination-exists = В месте назначения уже есть элемент с таким именем.
error-unsupported-format = Формат не поддерживается.
error-unsupported-encoding = Кодировка текста не определена или не поддерживается.
error-password-required = Файл защищён паролем.
error-engine-not-installed = Необходимый движок не установлен.
error-engine-crashed = Движок неожиданно остановился. Исходный файл сохранён.
error-model-unsupported = Модель несовместима с этой средой выполнения.
error-out-of-memory = Недостаточно памяти. Выберите модель или контекст меньше.
error-checksum-mismatch = Контрольная сумма не совпала; не установлено.
error-download-interrupted = Загрузка прервана.
error-provider-unavailable = AI-провайдер не отвечает.
error-rate-limited = Превышен лимит запросов. Повторите позже.
error-policy-blocked = Действие заблокировано политикой безопасности.
error-plan-expired = План действий устарел. Создайте новый план.
error-disk-full = На диске нет места.
error-cancelled = Отменено.
error-invalid-argument = Неверный запрос.
error-unsupported-api-version = Версии компонентов приложения не совпадают.
error-internal = Произошла внутренняя ошибка.

# P3-01 — listing va navigatsiya
nav-home = Домой
nav-up = Вверх
nav-refresh = Обновить
nav-path = Полный путь к папке
nav-open = Открыть
list-filter = Фильтр по имени
list-apply-filter = Фильтр
list-hidden = Скрытые файлы
list-loading = Чтение папки…
list-empty = Папка пуста или нет подходящих записей.
list-retry = Повторить
list-count = Записей: { $count }
list-truncated = Список ограничен. Используйте фильтр или откройте меньшую папку.
list-skipped = Не удалось прочитать: { $count }
list-symlink = Символическая ссылка; переход по ссылкам пока отключён.
list-directory = Папка


# P3-01b — metadata va picker
nav-pick-folder = Выбрать папку…
list-col-name = Имя
list-col-size = Размер
list-col-modified = Изменён

# P3-02 — tarix, saralash va multi-select
nav-back = Назад
nav-forward = Вперёд
list-selected = Выбрано: { $count } ({ $size })

# P3-02b — tablar, xatcho'plar va buyruq paneli
nav-bookmarks = Закладки
bookmark-add-current = Сохранить текущую папку
tab-new = Новая вкладка
tab-close = Закрыть вкладку
palette-title = Палитра команд
palette-hint = Введите команду…
palette-empty = Команды не найдены
cmd-go-home = Перейти в домашнюю папку
cmd-go-up = Вверх к родительской папке
cmd-go-back = Назад
cmd-go-forward = Вперёд
cmd-refresh = Обновить
cmd-new-tab = Открыть новую вкладку
cmd-close-tab = Закрыть активную вкладку
cmd-toggle-panel = Переключить активную панель (Tab)
cmd-toggle-hidden = Показать/скрыть скрытые файлы
cmd-sort-name = Сортировать по имени
cmd-sort-size = Сортировать по размеру
cmd-sort-modified = Сортировать по дате
cmd-select-all = Выбрать все файлы
cmd-deselect-all = Снять выделение
cmd-pick-folder = Открыть папку через системный диалог
cmd-add-bookmark = Добавить текущую папку в закладки

# P3-08 — tomlar (disklar) va F1/F2
volumes-title = Тома
volumes-open = Список томов
volumes-loading = Чтение томов…
volumes-empty = Тома не найдены
volumes-free = свободно { $free } из { $total }
volumes-read-only = только чтение
volumes-hint-select = выбор
volumes-hint-open = открыть
volumes-hint-close = закрыть
volume-kind-system = Системный
volume-kind-internal = Внутренний
volume-kind-removable = Съёмный
volume-kind-network = Сетевой
volume-kind-other = Другой
fkey-volumes-left = F1 Тома A
fkey-volumes-right = F2 Тома B

# P3-02d — вкладки, закладки и палитра команд (UI)
bookmarks-empty = Закладок нет. Сохраните текущую папку кнопкой ниже.
bookmark-remove = Удалить закладку
bookmark-already = Текущая папка сохранена
bookmarks-session-note = Пока закладки хранятся только в этом сеансе.
palette-hint-run = выполнить
