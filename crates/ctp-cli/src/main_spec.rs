//! Spec command handler (to be added to main.rs)

use anyhow::Result;
use crate::spec_commands::*;

async fn cmd_spec(
    action: SpecAction,
    llm_key: Option<&str>,
    llm_provider: Option<&str>,
) -> Result<()> {
    match action {
        SpecAction::Generate { path, use_llm, output } => {
            cmd_spec_generate(path, use_llm, output, llm_key, llm_provider).await
        }
        SpecAction::Validate { spec_file, path } => {
            cmd_spec_validate(spec_file, path).await
        }
        SpecAction::Enrich { spec_file, output } => {
            cmd_spec_enrich(spec_file, output, llm_key, llm_provider).await
        }
        SpecAction::Show { spec_file } => {
            cmd_spec_show(spec_file).await
        }
    }
}
