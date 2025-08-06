use serde::Deserialize;
use std::collections::HashMap;

/// Defines the structure for translation files.
#[derive(Deserialize)]
pub struct Translations {
    #[serde(flatten)]
    map: HashMap<String, String>,
}

impl Translations {
    pub fn get(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| key.to_string())
    }
}

/// Loads translations that match the current system locale.
/// Defaults to English if the locale detection fails or is not Chinese.
pub fn load() -> Translations {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    println!("[DEBUG] Detected system locale: {}", locale);

    let content = if locale.starts_with("zh") {
        println!("[DEBUG] Loading embedded 'zh_CN' translations.");
        include_str!("../../res/locales/zh_CN.json")
    } else {
        println!("[DEBUG] Loading embedded 'en' translations.");
        include_str!("../../res/locales/en.json")
    };

    serde_json::from_str(content)
        .expect("Failed to parse embedded translations JSON")
}