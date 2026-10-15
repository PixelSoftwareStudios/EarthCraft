use fluent_bundle::FluentValue;

// ───────────────────────────── Languages ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    PtBr,
}
impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::PtBr];
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::PtBr => "pt-BR",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::PtBr => "Português (Brasil)",
        }
    }
    /// An embedded, cached label with English fallback.
    pub fn text(self, id: &str) -> &str {
        super::localization::text(self, id)
    }
    /// Format a whole translated message; use string arguments for exact legacy numbers.
    pub fn format(self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        super::localization::format(self, id, args)
    }
}

/// Keep the user's choice separate from the language resolved for this session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LanguagePreference {
    #[default]
    System,
    Explicit(Lang),
}

impl LanguagePreference {
    pub const ALL: [Self; 3] = [
        Self::System,
        Self::Explicit(Lang::En),
        Self::Explicit(Lang::PtBr),
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Explicit(lang) => lang.code(),
        }
    }

    /// Existing lang.txt values remain explicit overrides.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|choice| choice.code() == code)
    }

    pub fn resolve(self, system_locale: &str) -> Lang {
        match self {
            Self::Explicit(lang) => lang,
            Self::System => Self::system_language(system_locale).unwrap_or(Lang::En),
        }
    }

    fn system_language(system_locale: &str) -> Option<Lang> {
        let locale = system_locale
            .split(['.', '@'])
            .next()
            .unwrap_or(system_locale)
            .replace('_', "-")
            .parse::<unic_langid::LanguageIdentifier>()
            .ok()?;
        match locale.language.as_str() {
            "en" => Some(Lang::En),
            "pt" => Some(Lang::PtBr),
            _ => None,
        }
    }

    pub fn label(self, lang: Lang, system_locale: &str) -> String {
        match self {
            Self::System => {
                let system_language = Self::system_language(system_locale);
                let message = if system_language.is_some() {
                    "settings-language-system"
                } else {
                    "settings-language-system-fallback"
                };
                lang.format(
                    message,
                    &[(
                        "language",
                        system_language.unwrap_or(Lang::En).label().into(),
                    )],
                )
            }
            Self::Explicit(language) => language.label().into(),
        }
    }
}

/// Canonically decomposes and removes combining accents for name matching.
/// This is a comparison key, not a transliteration or a displayed place name.
pub fn fold_name(s: &str) -> String {
    use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
    s.nfd()
        .flat_map(char::to_lowercase)
        .filter(|c| !is_combining_mark(*c) && c.is_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_language_resolves_supported_locales_with_english_fallback() {
        let preference = LanguagePreference::default();
        assert_eq!(preference, LanguagePreference::System);
        for locale in [
            "pt",
            "pt-BR",
            "pt-PT",
            "pt_BR.UTF-8",
            "pt_BR@modifier",
            "PT-br",
        ] {
            assert_eq!(preference.resolve(locale), Lang::PtBr, "{locale}");
        }
        for locale in [
            "en-US",
            "en_GB.UTF-8",
            "fr-FR",
            "ja-JP",
            "",
            "C",
            "bad_locale",
        ] {
            assert_eq!(preference.resolve(locale), Lang::En, "{locale}");
        }
        for language in Lang::ALL {
            let explicit = LanguagePreference::Explicit(language);
            for locale in ["en-US", "pt-BR", "fr-FR", ""] {
                assert_eq!(explicit.resolve(locale), language);
            }
        }
        assert_eq!(preference.code(), "system");
        assert_eq!(preference.resolve("pt-BR"), Lang::PtBr);
        assert_eq!(preference.resolve("en-US"), Lang::En);
    }
    #[test]
    fn system_labels_show_the_resolved_language_and_identify_fallbacks() {
        let system = LanguagePreference::System;
        for locale in ["en-US", "en_GB.UTF-8"] {
            assert_eq!(system.label(Lang::En, locale), "System locale (English)");
        }
        for locale in ["pt-BR", "pt-PT", "pt_BR.UTF-8"] {
            assert_eq!(
                system.label(Lang::En, locale),
                "System locale (Português (Brasil))"
            );
        }
        for locale in ["fr-FR", "ja-JP", "", "C", "bad_locale"] {
            assert_eq!(
                system.label(Lang::En, locale),
                "System locale (English — fallback)"
            );
            assert_eq!(
                system.label(Lang::PtBr, locale),
                "Idioma do sistema (English — idioma alternativo)"
            );
        }
        assert_eq!(
            system.label(Lang::PtBr, "pt-BR"),
            "Idioma do sistema (Português (Brasil))"
        );
        assert_eq!(
            system.label(Lang::PtBr, "en-US"),
            "Idioma do sistema (English)"
        );
        for language in Lang::ALL {
            for ui_language in Lang::ALL {
                assert_eq!(
                    LanguagePreference::Explicit(language).label(ui_language, "fr-FR"),
                    language.label()
                );
            }
        }
    }
    #[test]
    fn names_match_across_canonical_accents_and_case_without_transliterating_scripts() {
        for (name, expected) in [
            ("São Paulo", "saopaulo"),
            ("SA\u{303}O PAULO", "saopaulo"),
            ("ÁÀÂÃÄ ÉÈÊË ÍÌÎÏ ÓÒÔÕÖ ÚÙÛÜ Ç", "aaaaaeeeeiiiiooooouuuuc"),
            ("İstanbul", "istanbul"),
            ("Thành phố Huế", "thanhphohue"),
            ("Αθήνα", "αθηνα"),
            ("東京", "東京"),
            ("Москва", "москва"),
        ] {
            assert_eq!(fold_name(name), expected, "{name}");
        }
        assert_eq!(fold_name("Huế"), fold_name("Hue\u{302}\u{301}"));
    }
}
