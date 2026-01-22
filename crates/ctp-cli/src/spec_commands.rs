//! Product specification CLI commands

use std::path::PathBuf;
use anyhow::{Result, Context};
use console::style;
use walkdir::WalkDir;
use ctp_spec::{ProductSpecManager, ProductMetadata};

use crate::llm_utils;

pub async fn cmd_spec_generate(
    path: PathBuf,
    use_llm: bool,
    output: PathBuf,
    llm_key: Option<&str>,
    llm_provider: Option<&str>,
) -> Result<()> {
    println!("{} Generating product specification from codebase...", style("→").cyan());
    
    // Validate LLM config if needed
    if use_llm {
        let (key, provider) = llm_utils::validate_llm_config(llm_key, llm_provider)?;
        println!("{} Using LLM: {} (auto-detected from key)", style("→").cyan(), style(&provider).green());
        
        // TODO: Pass LLM config to generator
        // For now, just inform user
        println!("{} LLM enrichment will be implemented in next phase", style("!").yellow());
    }
    
    // Generate spec
    let manager = ProductSpecManager::load_or_generate(&path, use_llm).await?;
    let spec = manager.get_spec().await;
    
    // Save to output
    let json = serde_json::to_string_pretty(&spec)?;
    tokio::fs::write(&output, json).await?;
    
    println!("{} Product spec generated: {}", style("✓").green(), output.display());
    println!("  Confidence: {:.0}%", spec.confidence * 100.0);
    println!("  Core functionalities: {}", spec.core_functionalities.len());
    println!("  Generation method: {:?}", spec.generation_method);
    
    Ok(())
}

pub async fn cmd_spec_validate(
    spec_file: PathBuf,
    path: PathBuf,
) -> Result<()> {
    println!("{} Validating product spec against codebase...", style("→").cyan());
    
    // Load spec
    let content = tokio::fs::read_to_string(&spec_file).await
        .context("Failed to read spec file")?;
    let spec: ProductMetadata = serde_json::from_str(&content)?;
    
    // Analyze codebase
    use ctp_core::{CodeTruthEngine, EngineConfig};
    let engine = CodeTruthEngine::new(EngineConfig::default());
    
    // Collect files
    let mut analyses = vec![];
    for entry in walkdir::WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        if let Some(ext) = file_path.extension() {
            if ["rs", "py", "js", "ts", "go", "java"].contains(&ext.to_str().unwrap_or("")) {
                if let Ok(analysis) = engine.analyze_file(file_path).await {
                    analyses.push(analysis);
                }
            }
        }
    }
    
    // Validate
    let manager = ProductSpecManager::new(spec, spec_file.clone());
    let report = manager.validate(&analyses).await?;
    
    println!();
    if report.is_valid {
        println!("{} Spec is valid", style("✓").green());
    } else {
        println!("{} Spec validation issues found", style("!").yellow());
    }
    
    println!("  Confidence: {:.0}%", report.confidence * 100.0);
    
    if !report.missing_functionalities.is_empty() {
        println!("\n{} Missing functionalities (in spec but not in code):", style("⚠").yellow());
        for func in &report.missing_functionalities {
            println!("  - {}", func);
        }
    }
    
    if !report.undocumented_code.is_empty() {
        println!("\n{} Undocumented code (in code but not in spec):", style("→").cyan());
        for file in report.undocumented_code.iter().take(10) {
            println!("  - {}", file);
        }
        if report.undocumented_code.len() > 10 {
            println!("  ... and {} more", report.undocumented_code.len() - 10);
        }
    }
    
    Ok(())
}

pub async fn cmd_spec_enrich(
    spec_file: PathBuf,
    output: Option<PathBuf>,
    llm_key: Option<&str>,
    llm_provider: Option<&str>,
) -> Result<()> {
    println!("{} Enriching product spec with LLM...", style("→").cyan());
    
    // Validate LLM config
    let (key, provider) = llm_utils::validate_llm_config(llm_key, llm_provider)?;
    println!("{} Using LLM: {}", style("→").cyan(), style(&provider).green());
    
    // Load spec
    let content = tokio::fs::read_to_string(&spec_file).await?;
    let mut spec: ProductMetadata = serde_json::from_str(&content)?;
    
    println!("{} LLM enrichment will be implemented in next phase", style("!").yellow());
    println!("{} Current spec confidence: {:.0}%", style("→").cyan(), spec.confidence * 100.0);
    
    // Save
    let output_path = output.unwrap_or(spec_file);
    let json = serde_json::to_string_pretty(&spec)?;
    tokio::fs::write(&output_path, json).await?;
    
    println!("{} Spec saved to {}", style("✓").green(), output_path.display());
    
    Ok(())
}

pub async fn cmd_spec_show(spec_file: PathBuf) -> Result<()> {
    let content = tokio::fs::read_to_string(&spec_file).await?;
    let spec: ProductMetadata = serde_json::from_str(&content)?;
    
    println!("\n{}", style("Product Specification").bold());
    println!("  Product: {}", style(&spec.product.name).cyan());
    println!("  Type: {}", spec.product.product_type);
    println!("  Language: {}", spec.product.primary_language);
    println!("  Confidence: {:.0}%", spec.confidence * 100.0);
    println!("  Generation: {:?}", spec.generation_method);
    println!();
    
    println!("{}", style("Core Functionalities").bold());
    for func in &spec.core_functionalities {
        println!("  {} - {} ({})", 
            style(&func.name).cyan(),
            func.description,
            format!("{:?}", func.criticality)
        );
        println!("    Entry points: {}", func.entry_points.len());
        println!("    Confidence: {:.0}%", func.confidence * 100.0);
    }
    
    if let Some(constraints) = &spec.technical_constraints {
        println!();
        println!("{}", style("Technical Constraints").bold());
        if let Some(rt) = constraints.max_response_time_ms {
            println!("  Max response time: {}ms", rt);
        }
        if !constraints.supported_platforms.is_empty() {
            println!("  Platforms: {}", constraints.supported_platforms.join(", "));
        }
    }
    
    Ok(())
}
