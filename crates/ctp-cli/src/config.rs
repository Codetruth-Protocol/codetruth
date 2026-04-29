use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

const DEFAULT_CONFIG_FILES: [&str; 2] = ["config.yaml", "config.yml"];

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    pub analysis: AnalysisConfig,
    pub llm: LlmConfig,
    pub drift: DriftConfig,
    pub policies: PoliciesConfig,
    pub output: OutputConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            analysis: AnalysisConfig::default(),
            llm: LlmConfig::default(),
            drift: DriftConfig::default(),
            policies: PoliciesConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AnalysisConfig {
    pub languages: Vec<String>,
    pub exclude: Vec<String>,
    pub max_file_size: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                "python".into(),
                "javascript".into(),
                "typescript".into(),
                "rust".into(),
                "go".into(),
                "java".into(),
            ],
            exclude: vec![
                "**/node_modules/**".into(),
                "**/target/**".into(),
                "**/.git/**".into(),
                "**/dist/**".into(),
                "**/build/**".into(),
                "**/__pycache__/**".into(),
                "**/venv/**".into(),
            ],
            max_file_size: 10 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    pub enabled: bool,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: None,
            model: None,
            api_key: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DriftConfig {
    pub min_severity: String,
    pub similarity_threshold: f64,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            min_severity: "low".into(),
            similarity_threshold: 0.7,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PoliciesConfig {
    pub path: String,
    pub fail_on_violation: bool,
}

impl Default for PoliciesConfig {
    fn default() -> Self {
        Self {
            path: ".ctp/policies".into(),
            fail_on_violation: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub format: String,
    pub store_results: bool,
    pub results_path: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "simple".into(),
            store_results: true,
            results_path: ".ctp/analyses".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub config_path: Option<PathBuf>,
    pub root_dir: PathBuf,
}

impl ConfigPaths {
    pub fn resolve(explicit: Option<PathBuf>, root: &Path) -> Result<Self> {
        if let Some(path) = explicit {
            return Ok(Self {
                config_path: Some(path),
                root_dir: root.to_path_buf(),
            });
        }

        if let Ok(env_path) = env::var("CTP_CONFIG") {
            return Ok(Self {
                config_path: Some(PathBuf::from(env_path)),
                root_dir: root.to_path_buf(),
            });
        }

        let ctp_dir = root.join(".ctp");
        for candidate in DEFAULT_CONFIG_FILES {
            let path = ctp_dir.join(candidate);
            if path.exists() {
                return Ok(Self {
                    config_path: Some(path),
                    root_dir: root.to_path_buf(),
                });
            }
        }

        Ok(Self {
            config_path: None,
            root_dir: root.to_path_buf(),
        })
    }
}

impl CliConfig {
    pub fn load(paths: &ConfigPaths) -> Result<Self> {
        let mut config = CliConfig::default();

        if let Some(path) = &paths.config_path {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config at {}", path.display()))?;
            config = serde_yaml::from_str(&contents)
                .with_context(|| format!("Invalid YAML in {}", path.display()))?;
        }

        if config.llm.api_key.is_none() {
            if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
                config.llm.api_key = Some(key);
                config.llm.provider.get_or_insert("anthropic".into());
            } else if let Ok(key) = env::var("OPENAI_API_KEY") {
                config.llm.api_key = Some(key);
                config.llm.provider.get_or_insert("openai".into());
            }
        }

        Ok(config)
    }

    pub fn exclude_set(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.analysis.exclude {
            builder.add(Glob::new(pattern)?);
        }
        Ok(builder.build()?)
    }

    pub fn allowed_languages(&self) -> HashSet<String> {
        self.analysis
            .languages
            .iter()
            .map(|lang| lang.to_lowercase())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct FileFilter {
    exclude: GlobSet,
    allowed_languages: HashSet<String>,
}

impl FileFilter {
    pub fn new(config: &CliConfig) -> Result<Self> {
        Ok(Self {
            exclude: config.exclude_set()?,
            allowed_languages: config.allowed_languages(),
        })
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        self.exclude.is_match(path)
    }

    pub fn language_for_path(&self, path: &Path) -> Option<String> {
        let language = language_from_path(path)?;
        if self.allowed_languages.contains(&language) {
            Some(language)
        } else {
            None
        }
    }

    pub fn is_allowed_file(&self, path: &Path) -> bool {
        if self.is_excluded(path) {
            return false;
        }
        self.language_for_path(path).is_some()
    }
}

pub fn language_from_path(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    language_from_extension(&ext).map(|lang| lang.to_string())
}

pub fn language_from_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "py" => Some("python"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "ts" | "tsx" => Some("typescript"),
        "rs" => Some("rust"),
        "go" => Some("go"),
        "java" => Some("java"),
        "rb" => Some("ruby"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("cpp"),
        "cs" => Some("csharp"),
        _ => None,
    }
}
