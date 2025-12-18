use fluent::{FluentBundle, FluentResource};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use rust_embed::RustEmbed;
use unic_langid::LanguageIdentifier;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Locales;

pub struct LocaleManager {
    bundle: FluentBundle<FluentResource>,
    pub current_lang: LanguageIdentifier,
}

impl Default for LocaleManager {
    fn default() -> Self {
        Self::new("en-US")
    }
}

impl LocaleManager {
    pub fn new(lang_id: &str) -> Self {
        let lang: LanguageIdentifier = lang_id.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
        let bundle = Self::load_bundle(&lang);
        Self {
            bundle,
            current_lang: lang,
        }
    }

    fn load_bundle(lang_id: &LanguageIdentifier) -> FluentBundle<FluentResource> {
        let mut bundle = FluentBundle::new(vec![lang_id.clone()]);

        // Determine available locales
        // Note: This must match folders in locales/
        let available_locales: Vec<LanguageIdentifier> =
            vec!["en".parse().unwrap(), "de".parse().unwrap()];

        // Negotiate
        let requested = vec![lang_id.clone()];
        let default = "en".parse().unwrap();
        let supported = negotiate_languages(
            &requested,
            &available_locales,
            Some(&default),
            NegotiationStrategy::Filtering,
        );

        // Load resources
        let active_lang = supported.first().unwrap();
        // Use just the language code ("en", "de") for folder names
        let lang_key = active_lang.language.as_str();

        let path = format!("{}/main.ftl", lang_key);

        // Function to load and add resource
        let load_res = |b: &mut FluentBundle<FluentResource>, p: &str| {
            if let Some(file) = Locales::get(p) {
                if let Ok(source) = String::from_utf8(file.data.into_owned()) {
                    if let Ok(resource) = FluentResource::try_new(source) {
                        let _ = b.add_resource(resource);
                        return true;
                    }
                }
            }
            false
        };

        if !load_res(&mut bundle, &path) {
            eprintln!("Locale file not found or invalid: {}", path);
            // Fallback to English if failed
            if lang_key != "en" {
                load_res(&mut bundle, "en/main.ftl");
            }
        }

        bundle
    }

    pub fn set_locale(&mut self, lang_id: &str) {
        let lang: LanguageIdentifier = lang_id.parse().unwrap_or_else(|_| "en-US".parse().unwrap());
        self.bundle = Self::load_bundle(&lang);
        self.current_lang = lang;
    }

    pub fn t(&self, key: &str) -> String {
        self.format_pattern(key, None)
    }

    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut f_args = fluent::FluentArgs::new();
        for (k, v) in args {
            f_args.set(*k, *v);
        }
        self.format_pattern(key, Some(&f_args))
    }

    fn format_pattern(&self, key: &str, args: Option<&fluent::FluentArgs>) -> String {
        let pattern = match self.bundle.get_message(key) {
            Some(msg) => match msg.value() {
                Some(pattern) => pattern,
                None => return key.to_string(),
            },
            None => return key.to_string(),
        };

        let mut errors = vec![];
        let value = self.bundle.format_pattern(pattern, args, &mut errors);
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_manager_creation() {
        let lm = LocaleManager::new("en");
        assert_eq!(lm.current_lang.language.as_str(), "en");
    }

    #[test]
    fn test_translation_en() {
        let lm = LocaleManager::new("en");
        // Test a known key from en/main.ftl
        let t = lm.t("menu-file");
        assert_ne!(t, "menu-file");
        // Assuming "File" is the translation
        assert!(t.contains("File") || t.contains("Datei") == false);
    }

    #[test]
    fn test_translation_de() {
        let lm = LocaleManager::new("de");
        // Test a known key from de/main.ftl
        let t = lm.t("menu-file");
        assert_ne!(t, "menu-file");
        assert!(t.contains("Datei"));
    }

    #[test]
    fn test_set_locale() {
        let mut lm = LocaleManager::new("en");
        assert!(lm.t("menu-file").contains("File"));

        lm.set_locale("de");
        assert!(lm.t("menu-file").contains("Datei"));
    }

    #[test]
    fn test_missing_key() {
        let lm = LocaleManager::new("en");
        let t = lm.t("non-existent-key");
        assert_eq!(t, "non-existent-key");
    }

    #[test]
    fn test_translation_performance() {
        let lm = LocaleManager::new("en");
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = lm.t("menu-file");
        }
        let duration = start.elapsed();
        println!("1000 translations took: {:?}", duration);
        // Usually should be < 10ms on modern CPUs
        assert!(duration.as_millis() < 50);
    }
}
