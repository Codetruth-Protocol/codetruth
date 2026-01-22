use ctp_core::{CodeTruthEngine, EngineConfig};

use crate::config::CliConfig;

pub fn build_engine(config: &CliConfig, enable_llm: bool) -> CodeTruthEngine {
    let engine_config = EngineConfig {
        enable_llm,
        llm_provider: config.llm.provider.clone(),
        llm_model: config.llm.model.clone(),
        llm_api_key: config.llm.api_key.clone(),
        max_file_size: config.analysis.max_file_size,
        languages: config.analysis.languages.clone(),
    };

    CodeTruthEngine::new(engine_config)
}
