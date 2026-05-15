use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::models::{ModelProviderStatus, ModelProviderStatusKind};

pub const DEFAULT_QWEN_ENDPOINT: &str =
    "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";
pub const DEFAULT_QWEN_MODEL: &str = "qwen-plus";

const DEFAULT_PROVIDER: &str = "qwen";
const CONFIG_FILE_NAME: &str = "model-provider.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ModelProviderConfig {
    #[serde(alias = "provider")]
    pub provider: String,
    #[serde(alias = "api_url")]
    pub api_url: String,
    #[serde(alias = "model_name")]
    pub model_name: String,
    #[serde(alias = "api_key")]
    pub api_key: String,
}

pub struct ModelProviderConfigState {
    pub config: Option<ModelProviderConfig>,
    pub config_path: PathBuf,
    pub created_template: bool,
}

impl ModelProviderConfigState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let config_path = resolve_config_path(app)?;
        let created_template = ensure_template_file(&config_path)?;
        let raw = fs::read_to_string(&config_path).map_err(|error| {
            format!(
                "读取模型配置文件失败：{} ({error})",
                config_path.display()
            )
        })?;

        let config: ModelProviderConfig = serde_json::from_str(&raw).map_err(|error| {
            format!(
                "模型配置文件解析失败：{} ({error})",
                config_path.display()
            )
        })?;

        if !config.has_api_key() {
            return Ok(Self {
                config: None,
                config_path,
                created_template,
            });
        }

        config.validate().map_err(|message| {
            format!("{message}。配置文件：{}", config_path.display())
        })?;

        Ok(Self {
            config: Some(config),
            config_path,
            created_template,
        })
    }

    pub fn fallback_notice(&self) -> String {
        if self.created_template {
            return format!(
                "未检测到可用模型配置，已在 {} 生成 model-provider.json 模板。填写 PROVIDER、API_URL、MODEL_NAME、API_KEY 后，Tauri 桌面端会自动切换到实时分析；当前已回退到本地 mock 会话。",
                self.config_path.display()
            );
        }

        format!(
            "未检测到可用的 API_KEY，已回退到本地 mock 会话。请在 {} 中通过配置文件填写 PROVIDER、API_URL、MODEL_NAME、API_KEY 后重试。",
            self.config_path.display()
        )
    }
}

impl ModelProviderConfig {
    pub fn has_api_key(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    pub fn provider_slug(&self) -> String {
        self.provider.trim().to_ascii_lowercase()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.provider.trim().is_empty() {
            return Err("模型配置缺少 PROVIDER".to_string());
        }

        if self.api_url.trim().is_empty() {
            return Err("模型配置缺少 API_URL".to_string());
        }

        if self.model_name.trim().is_empty() {
            return Err("模型配置缺少 MODEL_NAME".to_string());
        }

        Ok(())
    }
}

pub fn get_status(app: &AppHandle) -> Result<ModelProviderStatus, String> {
    let config_path = resolve_config_path(app)?;
    let created_template = ensure_template_file(&config_path)?;
    let raw = fs::read_to_string(&config_path).map_err(|error| {
        format!(
            "读取模型配置文件失败：{} ({error})",
            config_path.display()
        )
    })?;

    let config_path_string = config_path.display().to_string();

    match serde_json::from_str::<ModelProviderConfig>(&raw) {
        Ok(config) => {
            let provider = trimmed_option(&config.provider);
            let api_url = trimmed_option(&config.api_url);
            let model_name = trimmed_option(&config.model_name);
            let has_api_key = config.has_api_key();
            let api_key_preview = mask_api_key(&config.api_key);
            let provider_slug = config.provider_slug();

            let (status, message) = match config.validate() {
                Err(error) => (ModelProviderStatusKind::InvalidConfig, error),
                Ok(()) if !is_supported_provider(&provider_slug) => (
                    ModelProviderStatusKind::UnsupportedProvider,
                    format!(
                        "PROVIDER={} 暂不支持。当前仅支持 qwen / dashscope 兼容链路。",
                        provider.clone().unwrap_or_else(|| "unknown".to_string())
                    ),
                ),
                Ok(()) if !has_api_key => (
                    ModelProviderStatusKind::NeedsConfiguration,
                    if created_template {
                        "模板已生成，填写 API_KEY 后即可切换到实时分析。".to_string()
                    } else {
                        "配置文件已读取，但 API_KEY 仍为空，当前会回退到本地 mock。".to_string()
                    },
                ),
                Ok(()) => (
                    ModelProviderStatusKind::Ready,
                    "模型配置已通过校验，新的桌面会话会直接走实时分析。".to_string(),
                ),
            };

            Ok(ModelProviderStatus {
                status,
                message,
                config_path: config_path_string,
                created_template,
                provider,
                api_url,
                model_name,
                has_api_key,
                api_key_preview,
            })
        }
        Err(error) => Ok(ModelProviderStatus {
            status: ModelProviderStatusKind::InvalidConfig,
            message: format!("模型配置文件不是合法 JSON：{error}"),
            config_path: config_path_string,
            created_template,
            provider: None,
            api_url: None,
            model_name: None,
            has_api_key: false,
            api_key_preview: None,
        }),
    }
}

pub fn is_supported_provider(provider_slug: &str) -> bool {
    matches!(
        provider_slug,
        "qwen" | "dashscope" | "aliyun-dashscope" | "aliyun_dashscope"
    )
}

fn resolve_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("无法定位应用配置目录：{error}"))?;

    Ok(config_dir.join(CONFIG_FILE_NAME))
}

fn ensure_template_file(config_path: &Path) -> Result<bool, String> {
    if config_path.exists() {
        return Ok(false);
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "创建模型配置目录失败：{} ({error})",
                parent.display()
            )
        })?;
    }

    let template = ModelProviderConfig {
        provider: DEFAULT_PROVIDER.to_string(),
        api_url: DEFAULT_QWEN_ENDPOINT.to_string(),
        model_name: DEFAULT_QWEN_MODEL.to_string(),
        api_key: String::new(),
    };

    let content = serde_json::to_string_pretty(&template)
        .map_err(|error| format!("生成模型配置模板失败：{error}"))?;

    fs::write(config_path, format!("{content}\n")).map_err(|error| {
        format!(
            "写入模型配置模板失败：{} ({error})",
            config_path.display()
        )
    })?;

    Ok(true)
}

fn trimmed_option(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn mask_api_key(api_key: &str) -> Option<String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return None;
    }

    let suffix = trimmed
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    Some(format!("已配置（后 4 位：{}）", suffix))
}