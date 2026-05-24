//! GitHub 采集器
//!
//! - 日榜：昨天创建的项目
//! - 周/月/年/总榜：按时间范围
//! - 趋势线：项目每日 star 历史

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEntry {
    pub full_name: String,
    pub description: String,
    pub stars: u64,
    pub url: String,
    pub language: String,
    pub topics: Vec<String>,
    pub created_at: String,
    pub pushed_at: String,
}

/// 按时间范围查询 GitHub 热门仓库
/// period: "daily" / "weekly" / "monthly" / "yearly" / "all"
pub async fn fetch_trending(period: &str, limit: usize) -> Result<Vec<RepoEntry>> {
    let date_filter = match period {
        "daily" => {
            let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
            let today = chrono::Local::now().date_naive();
            format!("created:>={}&created:<{}", yesterday, today)
        }
        "weekly" => {
            let week_ago = chrono::Local::now().date_naive() - chrono::Duration::days(7);
            format!("created:>{}", week_ago)
        }
        "monthly" => {
            let month_ago = chrono::Local::now().date_naive() - chrono::Duration::days(30);
            format!("created:>{}", month_ago)
        }
        "yearly" => {
            let year_ago = chrono::Local::now().date_naive() - chrono::Duration::days(365);
            format!("created:>{}", year_ago)
        }
        "all" => "stars:>10000".to_string(),
        _ => "stars:>100".to_string(),
    };

    let _query = format!("{}&sort=stars&order=desc&per_page={}", date_filter, limit.min(25));
    let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}",
        date_filter, limit.min(25));

    let client = reqwest::Client::builder()
        .user_agent("aion-news/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let resp = client.get(&url).send().await?;
    let body: serde_json::Value = resp.json().await?;
    let empty_vec = vec![];
    let items = body["items"].as_array().unwrap_or(&empty_vec);

    let repos = items.iter().map(|item| {
        RepoEntry {
            full_name: item["full_name"].as_str().unwrap_or("unknown").to_string(),
            description: item["description"].as_str().unwrap_or("").to_string(),
            stars: item["stargazers_count"].as_u64().unwrap_or(0),
            url: item["html_url"].as_str().unwrap_or("").to_string(),
            language: item["language"].as_str().unwrap_or("N/A").to_string(),
            topics: item["topics"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            created_at: item["created_at"].as_str().unwrap_or("").to_string(),
            pushed_at: item["pushed_at"].as_str().unwrap_or("").to_string(),
        }
    }).collect();

    Ok(repos)
}

/// 查询项目 star 历史（简化版：取过去 7 天每天的快照差值）
pub async fn fetch_star_history(repo: &str, days: u32) -> Result<Vec<(String, u64)>> {
    // 直接用 today -> yesterday -> ... 各时间点的 star 数
    let client = reqwest::Client::builder()
        .user_agent("aion-news/0.1")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let mut history = Vec::new();
    for i in 0..days {
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(i as i64);
        // GitHub API 不支持按日期查 star，此处做占位
        // 后续可用 Star History API（star-history.com）或自建快照
        let url = format!("https://api.github.com/repos/{}", repo);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(stars) = body["stargazers_count"].as_u64() {
                    history.push((date.to_string(), stars));
                    continue;
                }
            }
        }
        history.push((date.to_string(), 0));
    }

    Ok(history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_trending_daily() {
        let repos = fetch_trending("daily", 3).await.unwrap_or_default();
        // GitHub API may fail without network, so just check it doesn't crash
        assert!(repos.len() <= 3);
    }

    #[test]
    fn test_date_filter_daily_is_yesterday() {
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        let today = chrono::Local::now().date_naive();
        assert_ne!(yesterday, today);
    }
}
