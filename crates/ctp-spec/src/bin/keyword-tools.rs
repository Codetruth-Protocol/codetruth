#!/usr/bin/env cargo script

//! Keyword Dictionary CLI Tools
//! 
//! Provides utilities for managing the keyword dictionary:
//! - Validation
//! - Statistics
//! - Search
//! - Export/Import

use std::env;
use std::path::Path;
use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin keyword-tools <command> [args]");
        eprintln!("Commands:");
        eprintln!("  validate [path]    - Validate keyword dictionary");
        eprintln!("  stats [path]       - Show dictionary statistics");
        eprintln!("  search <term>      - Search for keywords");
        eprintln!("  export <format>    - Export dictionary (json, csv)");
        return Ok(());
    }
    
    let command = &args[1];
    let dict_path = if args.len() > 2 {
        Path::new(&args[2])
    } else {
        Path::new("spec/keywords.csv")
    };
    
    match command.as_str() {
        "validate" => {
            let dict = ctp_spec::KeywordDictionary::load_from_csv(dict_path)?;
            let validation = dict.validate()?;
            
            if validation.errors.is_empty() {
                println!("✅ Dictionary validation passed");
                if !validation.warnings.is_empty() {
                    println!("⚠️  Warnings:");
                    for warning in validation.warnings {
                        println!("  - {}", warning);
                    }
                }
            } else {
                println!("❌ Dictionary validation failed:");
                for error in validation.errors {
                    println!("  - {}", error);
                }
                std::process::exit(1);
            }
        }
        "stats" => {
            let dict = ctp_spec::KeywordDictionary::load_from_csv(dict_path)?;
            let stats = dict.stats();
            
            println!("📊 Keyword Dictionary Statistics");
            println!("Version: {}", stats.version);
            println!("Last Updated: {}", stats.last_updated);
            println!("Domain Keywords: {}", stats.domain_keywords_count);
            println!("Critical Domains: {}", stats.critical_domains_count);
            println!("Stopwords: {}", stats.stopwords_count);
            println!("Project Types: {}", stats.project_types_count);
        }
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: cargo run --bin keyword-tools search <term>");
                return Ok(());
            }
            
            let dict = ctp_spec::KeywordDictionary::load_from_csv(dict_path)?;
            let search_term = &args[2];
            
            println!("🔍 Searching for: {}", search_term);
            
            // Search domain keywords
            if let Some(keyword) = dict.find_domain_keyword(search_term) {
                println!("Domain Keyword Found:");
                println!("  Term: {}", keyword.term);
                println!("  Weight: {}", keyword.weight);
                println!("  Context: {}", keyword.context);
                println!("  Synonyms: {}", keyword.synonyms.join(", "));
                println!("  Notes: {}", keyword.notes);
            }
            
            // Check criticality
            let score = dict.get_criticality_score(search_term);
            if score > 0 {
                println!("Criticality Score: {}/10", score);
            }
            
            // Check if stopword
            if dict.is_stopword(search_term) {
                println!("Stopword: Yes");
            }
        }
        "export" => {
            if args.len() < 3 {
                eprintln!("Usage: cargo run --bin keyword-tools export <format>");
                eprintln!("Formats: json, csv");
                return Ok(());
            }
            
            let dict = ctp_spec::KeywordDictionary::load_from_csv(dict_path)?;
            let format = &args[2];
            
            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&dict)?;
                    println!("{}", json);
                }
                "csv" => {
                    // Re-export as CSV (basically just cat the file)
                    let content = std::fs::read_to_string(dict_path)?;
                    println!("{}", content);
                }
                _ => {
                    eprintln!("Unsupported format: {}", format);
                    return Ok(());
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
    
    Ok(())
}
