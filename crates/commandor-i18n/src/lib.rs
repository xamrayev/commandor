//! Tarjimalar. Resurslar binaryga kompilyatsiya vaqtida joylanadi (`resources/i18n`).
//!
//! Ru/en tarjimasida kalit bo‘lmasa o‘zbekcha qiymat, u ham bo‘lmasa kalitning o‘zi qaytadi —
//! UI hech qachon bo‘sh matn ko‘rsatmaydi.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Uz,
    Ru,
    En,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Uz, Lang::Ru, Lang::En];

    pub const fn tag(self) -> &'static str {
        match self {
            Self::Uz => "uz-Latn",
            Self::Ru => "ru",
            Self::En => "en",
        }
    }

    /// Kompilyatsiyaga kiritilgan FTL manbasi.
    pub const fn source(self) -> &'static str {
        match self {
            Self::Uz => include_str!("../../../resources/i18n/uz/main.ftl"),
            Self::Ru => include_str!("../../../resources/i18n/ru/main.ftl"),
            Self::En => include_str!("../../../resources/i18n/en/main.ftl"),
        }
    }
}

/// FTL resursi yuklanmasa qaytadigan xato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub lang: Lang,
    pub details: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} tarjimasi yuklanmadi: {}",
            self.lang.tag(),
            self.details
        )
    }
}

impl std::error::Error for LoadError {}

pub struct I18n {
    lang: Lang,
    primary: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n")
            .field("lang", &self.lang)
            .finish_non_exhaustive()
    }
}

fn bundle(lang: Lang) -> Result<FluentBundle<FluentResource>, LoadError> {
    let err = |details: String| LoadError { lang, details };
    let id: LanguageIdentifier = lang.tag().parse().map_err(|e| err(format!("{e:?}")))?;
    let resource = FluentResource::try_new(lang.source().to_owned())
        .map_err(|(_, errors)| err(format!("{errors:?}")))?;
    let mut bundle = FluentBundle::new(vec![id]);
    // egui Unicode isolation belgilarini ko‘rinadigan qilib chizishi mumkin.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .map_err(|errors| err(format!("{errors:?}")))?;
    Ok(bundle)
}

impl I18n {
    pub fn new(lang: Lang) -> Result<Self, LoadError> {
        let primary = bundle(lang)?;
        let fallback = if lang == Lang::Uz {
            None
        } else {
            Some(bundle(Lang::Uz)?)
        };
        Ok(Self {
            lang,
            primary,
            fallback,
        })
    }

    pub const fn lang(&self) -> Lang {
        self.lang
    }

    pub fn t(&self, key: &str) -> String {
        self.format(key, None)
    }

    /// Argumentli xabar: `t_args("status-broker-ready", &[("version", "0.1.0")])`.
    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, FluentValue::from(*value));
        }
        self.format(key, Some(&fluent_args))
    }

    pub fn has(&self, key: &str) -> bool {
        self.primary.has_message(key)
    }

    fn format(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        for bundle in std::iter::once(&self.primary).chain(self.fallback.as_ref()) {
            if let Some(pattern) = bundle.get_message(key).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                return bundle
                    .format_pattern(pattern, args, &mut errors)
                    .into_owned();
            }
        }
        key.to_owned()
    }
}

/// FTL manbasidagi xabar kalitlari (testlar va xtask uchun).
pub fn message_keys(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter(|line| line.starts_with(|c: char| c.is_ascii_lowercase()))
        .filter_map(|line| line.split_once(" =").map(|(key, _)| key.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use commandor_core::ErrorCode;
    use std::collections::BTreeSet;

    #[test]
    fn all_languages_load() {
        for lang in Lang::ALL {
            I18n::new(lang).expect("load");
        }
    }

    #[test]
    fn all_languages_share_the_same_keys() {
        let uz: BTreeSet<_> = message_keys(Lang::Uz.source()).into_iter().collect();
        assert!(uz.len() > 10);
        for lang in [Lang::Ru, Lang::En] {
            let other: BTreeSet<_> = message_keys(lang.source()).into_iter().collect();
            assert_eq!(uz, other, "{} kalitlari uz bilan mos emas", lang.tag());
        }
    }

    #[test]
    fn every_error_code_has_a_translation_in_every_language() {
        for lang in Lang::ALL {
            let i18n = I18n::new(lang).expect("load");
            for code in ErrorCode::ALL {
                assert!(i18n.has(code.message_key()), "{} {:?}", lang.tag(), code);
            }
        }
    }

    #[test]
    fn arguments_and_fallback() {
        let uz = I18n::new(Lang::Uz).expect("load");
        let text = uz.t_args(
            "status-broker-ready",
            &[("version", "0.1.0"), ("os", "macos"), ("arch", "aarch64")],
        );
        assert_eq!(text, "Tayyor · v0.1.0 · macos/aarch64");
        assert_eq!(uz.t("panel-right"), "B panel");
        assert_eq!(uz.t("no-such-key"), "no-such-key");
    }
}
