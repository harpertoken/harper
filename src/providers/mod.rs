use crate::core::{ApiConfig, ApiProvider, Message};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

pub async fn call_llm(
    client: &reqwest::Client,
    config: &ApiConfig,
    history: &[Message],
) -> Result<String, Box<dyn std::error::Error>> {
    let res = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => {
            let messages_json: Vec<_> = history
                .iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect();
            let body = json!({
                "model": config.model_name,
                "messages": messages_json,
                "temperature": 0.1,
                "top_p": 0.1
            });
            client
                .post(&config.base_url)
                .header(AUTHORIZATION, format!("Bearer {}", config.api_key))
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
        ApiProvider::Gemini => {
            let mut gemini_contents = Vec::new();
            if let Some(first_message) = history.first() {
                if first_message.role == "system" {
                    gemini_contents.push(json!({
                        "role": "user",
                        "parts": [{"text": first_message.content}]
                    }));
                    gemini_contents.push(json!({
                        "role": "model",
                        "parts": [{"text": "Understood."}]
                    }));
                }
            }

            for msg in history.iter().skip(1) {
                let role = if msg.role == "assistant" {
                    "model"
                } else {
                    "user"
                };
                gemini_contents.push(json!({
                    "role": role,
                    "parts": [{"text": msg.content}]
                }));
            }

            let body = json!({
                "contents": gemini_contents
            });
            let url = format!("{}?key={}", config.base_url, config.api_key);
            client
                .post(&url)
                .header(CONTENT_TYPE, "application/json")
                .json(&body)
                .send()
                .await?
        }
    };

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(format!("API Error: {} ({})", error_text, status).into());
    }

    let resp_json: serde_json::Value = res.json().await.unwrap_or_else(|_| json!({}));

    let assistant_reply = match config.provider {
        ApiProvider::OpenAI | ApiProvider::Sambanova => resp_json["choices"][0]["message"]
            ["content"]
            .as_str()
            .unwrap_or("[No response]")
            .to_string(),
        ApiProvider::Gemini => resp_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("[No response]")
            .to_string(),
    };

    Ok(assistant_reply)
}
