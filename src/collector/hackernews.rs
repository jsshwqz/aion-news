//! Hacker News 采集器
//!
//! 通过 Algolia HN API 获取当日 AI 相关热帖

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNStory {
    pub title: String,
    pub url: Option<String>,
    pub points: i64,
    pub author: String,
    pub created_at: String,
    pub num_comments: i64,
}

/// 获取 Hacker News 当日 AI 相关热帖
pub async fn fetch_ai_stories(days: u32) -> Result<Vec<HNStory>> {
    let cutoff = (chrono::Local::now() - chrono::Duration::days(days as i64)).to_rfc3339();
    let url = format!(
        "https://hn.algolia.com/api/v1/search?query=AI+OR+LLM+OR+agent+OR+Claude+OR+GPT+OR+coding&tags=story&numericFilters=created_at_i>{}&hitsPerPage=10",
        chrono::Local::now().timestamp() - days as i64 * 86400
    );

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    let body: serde_json::Value = resp.json().await?;

    let stories = body["hits"].as_array()
        .map(|arr| arr.iter().map(|h| HNStory {
            title: h["title"].as_str().unwrap_or("").to_string(),
            url: h["url"].as_str().map(String::from),
            points: h["points"].as_i64().unwrap_or(0),
            author: h["author"].as_str().unwrap_or("").to_string(),
            created_at: h["created_at"].as_str().unwrap_or("").to_string(),
            num_comments: h["num_comments"].as_i64().unwrap_or(0),
        }).collect())
        .unwrap_or_default();

    Ok(stories)
}
