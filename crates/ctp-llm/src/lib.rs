//! # CTP LLM Integration
//!
//! LLM integration layer for CodeTruth Protocol.
//! Supports Anthropic Claude, OpenAI GPT, and local models via Ollama.
//!
//! ## Context-Aware Analysis
//!
//! The `context_aware` module provides hierarchical context compression
//! for analyzing code within large codebases without losing essential information.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

pub mod context_aware;

pub use context_aware::{ContextAwareLLM, ContextAwareRequest, ContextAwareInference};

#[derive(Debug, Clone)]
pub enum LLMProvider {
    Anthropic,
    OpenAI,
    Ollama,
}

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: usize,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            model: "claude-sonnet-4-20250514".into(),
            api_key: None,
            base_url: None,
            max_tokens: 1000,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentInference {
    pub inferred_intent: String,
    pub confidence: f64,
    pub business_context: String,
    pub technical_rationale: String,
}

pub struct LLMClient {
    config: LLMConfig,
    client: reqwest::Client,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            config,
            client,
        }
    }

    async fn retry_with_backoff<F, Fut, T>(
        &self,
        mut operation: F,
        max_retries: u32,
    ) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt >= max_retries {
                        return Err(e);
                    }
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    warn!("LLM request failed (attempt {}/{}), retrying in {:?}: {}", 
                          attempt, max_retries, delay, e);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    pub async fn infer_intent(&self, code: &str, context: &str) -> Result<IntentInference> {
        debug!("Inferring intent via LLM");
        
        let prompt = format!(
            r#"Analyze this code and infer its intent.

Code:
```
{}
```

Context (comments/docs):
{}

Respond in JSON:
{{"inferred_intent": "...", "confidence": 0.0-1.0, "business_context": "...", "technical_rationale": "..."}}"#,
            code, context
        );

        match self.config.provider {
            LLMProvider::Anthropic => self.call_anthropic(&prompt).await,
            LLMProvider::OpenAI => self.call_openai(&prompt).await,
            LLMProvider::Ollama => self.call_ollama(&prompt).await,
        }
    }

    async fn call_anthropic(&self, prompt: &str) -> Result<IntentInference> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key required"))?;

        self.retry_with_backoff(
            || async {
                let response = self.client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("Content-Type", "application/json")
                    .json(&serde_json::json!({
                        "model": &self.config.model,
                        "max_tokens": self.config.max_tokens,
                        "messages": [{"role": "user", "content": prompt}]
                    }))
                    .send()
                    .await
                    .context("Failed to send request to Anthropic API")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    anyhow::bail!("Anthropic API error {}: {}", status, error_text);
                }

                let json: serde_json::Value = response.json().await
                    .context("Failed to parse Anthropic response as JSON")?;
                
                let content = json["content"][0]["text"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing 'content[0].text' in Anthropic response"))?;
                
                let inference: IntentInference = serde_json::from_str(content)
                    .context("Failed to parse LLM response as IntentInference JSON")?;
                
                Ok(inference)
            },
            3,
        ).await
    }

    async fn call_openai(&self, prompt: &str) -> Result<IntentInference> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key required"))?;

        self.retry_with_backoff(
            || async {
                let response = self.client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&serde_json::json!({
                        "model": &self.config.model,
                        "max_tokens": self.config.max_tokens,
                        "messages": [{"role": "user", "content": prompt}]
                    }))
                    .send()
                    .await
                    .context("Failed to send request to OpenAI API")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    anyhow::bail!("OpenAI API error {}: {}", status, error_text);
                }

                let json: serde_json::Value = response.json().await
                    .context("Failed to parse OpenAI response as JSON")?;
                
                let content = json["choices"][0]["message"]["content"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing 'choices[0].message.content' in OpenAI response"))?;
                
                let inference: IntentInference = serde_json::from_str(content)
                    .context("Failed to parse LLM response as IntentInference JSON")?;
                
                Ok(inference)
            },
            3,
        ).await
    }

    async fn call_ollama(&self, prompt: &str) -> Result<IntentInference> {
        let base_url = self.config.base_url.as_deref().unwrap_or("http://localhost:11434");
        
        self.retry_with_backoff(
            || async {
                let response = self.client
                    .post(format!("{}/api/generate", base_url))
                    .json(&serde_json::json!({
                        "model": &self.config.model,
                        "prompt": prompt,
                        "stream": false
                    }))
                    .send()
                    .await
                    .context("Failed to send request to Ollama API")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    anyhow::bail!("Ollama API error {}: {}", status, error_text);
                }

                let json: serde_json::Value = response.json().await
                    .context("Failed to parse Ollama response as JSON")?;
                
                let content = json["response"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing 'response' in Ollama response"))?;
                
                let inference: IntentInference = serde_json::from_str(content)
                    .context("Failed to parse LLM response as IntentInference JSON")?;
                
                Ok(inference)
            },
            3,
        ).await
    }
}
