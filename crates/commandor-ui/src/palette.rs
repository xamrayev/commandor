//! Buyruqlar paneli modeli (GUIsiz test qilinadi): buyruqlar ro‘yxati va qidiruv.

/// Panel orqali bajariladigan buyruq. Har birining i18n nomi bor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaletteCommand {
    GoHome,
    GoUp,
    GoBack,
    GoForward,
    Refresh,
    NewTab,
    CloseTab,
    TogglePanel,
    ToggleHidden,
    SortName,
    SortSize,
    SortModified,
    SelectAll,
    DeselectAll,
    PickFolder,
    AddBookmark,
}

impl PaletteCommand {
    pub const ALL: [Self; 16] = [
        Self::GoHome,
        Self::GoUp,
        Self::GoBack,
        Self::GoForward,
        Self::Refresh,
        Self::NewTab,
        Self::CloseTab,
        Self::TogglePanel,
        Self::ToggleHidden,
        Self::SortName,
        Self::SortSize,
        Self::SortModified,
        Self::SelectAll,
        Self::DeselectAll,
        Self::PickFolder,
        Self::AddBookmark,
    ];

    pub const fn label_key(self) -> &'static str {
        match self {
            Self::GoHome => "cmd-go-home",
            Self::GoUp => "cmd-go-up",
            Self::GoBack => "cmd-go-back",
            Self::GoForward => "cmd-go-forward",
            Self::Refresh => "cmd-refresh",
            Self::NewTab => "cmd-new-tab",
            Self::CloseTab => "cmd-close-tab",
            Self::TogglePanel => "cmd-toggle-panel",
            Self::ToggleHidden => "cmd-toggle-hidden",
            Self::SortName => "cmd-sort-name",
            Self::SortSize => "cmd-sort-size",
            Self::SortModified => "cmd-sort-modified",
            Self::SelectAll => "cmd-select-all",
            Self::DeselectAll => "cmd-deselect-all",
            Self::PickFolder => "cmd-pick-folder",
            Self::AddBookmark => "cmd-add-bookmark",
        }
    }
}

/// Paneldagi qator: buyruq yoki xatcho‘p (indeks `AppState::bookmarks` ichida).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteItem {
    Command(PaletteCommand),
    Bookmark(usize),
}

/// Ochiq panel holati.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaletteState {
    pub query: String,
    pub highlighted: usize,
}

/// So‘rovdagi har bir so‘z matnda (registrsiz) uchrasa mos keladi.
pub(crate) fn matches(query: &str, text: &str) -> bool {
    let text = text.to_lowercase();
    query
        .split_whitespace()
        .all(|term| text.contains(&term.to_lowercase()))
}

/// So‘rovga mos qatorlar: avval buyruqlar (tarjima qilingan nom bo‘yicha), keyin xatcho‘plar (yo‘l bo‘yicha).
pub(crate) fn filter(
    query: &str,
    bookmarks: &[String],
    label: impl Fn(PaletteCommand) -> String,
) -> Vec<PaletteItem> {
    let commands = PaletteCommand::ALL
        .into_iter()
        .filter(|&command| matches(query, &label(command)))
        .map(PaletteItem::Command);
    let marks = bookmarks
        .iter()
        .enumerate()
        .filter(|(_, path)| matches(query, path))
        .map(|(index, _)| PaletteItem::Bookmark(index));
    commands.chain(marks).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_term_must_match_case_insensitively() {
        assert!(matches("", "Har qanday"));
        assert!(matches("tab yangi", "Yangi tab ochish"));
        assert!(matches("ЯРЛЫК", "ярлык папки"));
        assert!(!matches("tab yopish saralash", "Faol tabni yopish"));
    }

    #[test]
    fn filter_lists_commands_then_bookmarks() {
        let bookmarks = vec!["/Users/me/docs".to_owned(), "/Volumes/USB".to_owned()];
        let all = filter("", &bookmarks, |c| c.label_key().to_owned());
        assert_eq!(all.len(), PaletteCommand::ALL.len() + 2);
        assert_eq!(all.last(), Some(&PaletteItem::Bookmark(1)));

        let found = filter("usb", &bookmarks, |c| c.label_key().to_owned());
        assert_eq!(found, vec![PaletteItem::Bookmark(1)]);

        let sort = filter("sort", &bookmarks, |c| c.label_key().to_owned());
        assert_eq!(
            sort,
            vec![
                PaletteItem::Command(PaletteCommand::SortName),
                PaletteItem::Command(PaletteCommand::SortSize),
                PaletteItem::Command(PaletteCommand::SortModified),
            ]
        );
    }

    #[test]
    fn label_keys_are_unique() {
        let keys: std::collections::BTreeSet<_> =
            PaletteCommand::ALL.iter().map(|c| c.label_key()).collect();
        assert_eq!(keys.len(), PaletteCommand::ALL.len());
    }
}
