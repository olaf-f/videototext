use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::models::AppSettings;
use crate::services::ai_service::{has_deepseek_api_key, save_deepseek_api_key};

const SETTINGS_FILE_NAME: &str = "settings.json";
const APP_DIR_NAME: &str = "smartocr-pro";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct StoredAppSettings {
    default_prompt: String,
    web_portal_url: String,
    #[serde(default)]
    deepseek_api_key: Option<String>,
}

impl Default for StoredAppSettings {
    fn default() -> Self {
        Self {
            default_prompt: AppSettings::default().default_prompt,
            web_portal_url: AppSettings::default().web_portal_url,
            deepseek_api_key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsService {
    settings_path: PathBuf,
}

impl SettingsService {
    pub fn new() -> AppResult<Self> {
        let base_dir = default_base_dir()?;
        Ok(Self::from_base_dir(base_dir))
    }

    pub fn new_for_tests(base_dir: PathBuf) -> Self {
        Self::from_base_dir(base_dir)
    }

    pub fn load(&self) -> AppResult<AppSettings> {
        let stored = self.load_stored_settings()?;
        let mut deepseek_api_key_saved = has_deepseek_api_key();

        if !deepseek_api_key_saved {
            if let Some(cached_key) = stored.deepseek_api_key.as_deref() {
                let cached_key = cached_key.trim();
                if !cached_key.is_empty() {
                    save_deepseek_api_key(cached_key)?;
                    deepseek_api_key_saved = true;
                }
            }
        }

        if !self.settings_path.exists() {
            return Ok(AppSettings {
                deepseek_api_key_saved,
                ..AppSettings::default()
            });
        }

        Ok(AppSettings {
            deepseek_api_key_saved,
            default_prompt: stored.default_prompt,
            web_portal_url: stored.web_portal_url,
        })
    }

    pub fn save(&self, settings: &AppSettings) -> AppResult<()> {
        self.ensure_parent_dir()?;
        let mut stored = self.load_stored_settings()?;
        stored.default_prompt = settings.default_prompt.clone();
        stored.web_portal_url = settings.web_portal_url.clone();
        let raw = serde_json::to_string_pretty(&stored)?;
        std::fs::write(&self.settings_path, raw)?;
        Ok(())
    }

    pub fn save_deepseek_api_key(&self, api_key: &str) -> AppResult<()> {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("DeepSeek API Key 不能为空".into()));
        }

        save_deepseek_api_key(trimmed)?;

        self.ensure_parent_dir()?;
        let mut stored = self.load_stored_settings()?;
        stored.deepseek_api_key = Some(trimmed.to_string());
        let raw = serde_json::to_string_pretty(&stored)?;
        std::fs::write(&self.settings_path, raw)?;
        Ok(())
    }

    fn from_base_dir(base_dir: PathBuf) -> Self {
        Self {
            settings_path: base_dir.join(SETTINGS_FILE_NAME),
        }
    }

    fn ensure_parent_dir(&self) -> AppResult<()> {
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(())
    }

    fn load_stored_settings(&self) -> AppResult<StoredAppSettings> {
        if !self.settings_path.exists() {
            return Ok(StoredAppSettings::default());
        }

        let raw = std::fs::read_to_string(&self.settings_path)?;
        let stored: StoredAppSettings = serde_json::from_str(&raw)?;
        Ok(stored)
    }

    #[allow(dead_code)]
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }
}

pub fn save_settings_for_test(base_dir: PathBuf, settings: &AppSettings) -> AppResult<()> {
    SettingsService::new_for_tests(base_dir).save(settings)
}

pub fn load_settings_for_test(base_dir: PathBuf) -> AppResult<AppSettings> {
    SettingsService::new_for_tests(base_dir).load()
}

fn default_base_dir() -> AppResult<PathBuf> {
    if let Some(path) = std::env::var_os("SMARTOCR_PRO_SETTINGS_DIR") {
        return Ok(PathBuf::from(path));
    }

    if let Some(path) = std::env::var_os("LOCALAPPDATA") {
        return Ok(PathBuf::from(path).join(APP_DIR_NAME));
    }

    if let Some(path) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(path).join(APP_DIR_NAME));
    }

    Err(AppError::SettingsPathUnavailable)
}
