use serde::Deserialize;
use std::collections::HashMap;

// 定义翻译文件的结构
#[derive(Deserialize)]
pub struct Translations {
    #[serde(flatten)]
    map: HashMap<String, String>,
}

impl Translations {
    pub fn get(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| {
            eprintln!("翻译键未找到: {}", key);
            key.to_string()
        })
    }
}

// 加载适合当前系统语言环境的翻译
pub fn load() -> Translations {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());

    let content = if locale.starts_with("zh") {
        include_str!("../../res/locales/zh_CN.json")
    } else {
        include_str!("../../res/locales/en.json")
    };

    serde_json::from_str(content)
        .unwrap_or_else(|e| {
            eprintln!("解析翻译文件失败: {}", e);
            Translations {
                map: std::collections::HashMap::new(),
            }
        })
}
