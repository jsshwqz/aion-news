//! ArXiv 采集器
//!
//! 通过 ArXiv API 获取 AI 相关新论文

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArxivPaper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub categories: Vec<String>,
    pub published: String,
}

/// 获取 ArXiv 最新 AI/Agent 相关论文
pub async fn fetch_recent_papers(max_results: u32) -> Result<Vec<ArxivPaper>> {
    let query = "cat:cs.AI+OR+cat:cs.CL+OR+cat:cs.LG+OR+cat:cs.MA";
    let url = format!(
        "http://export.arxiv.org/api/query?search_query={}&sortBy=submittedDate&sortOrder=descending&max_results={}",
        query, max_results
    );

    // ArXiv API 返回 XML，这里用简单方式解析
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    let text = resp.text().await?;

    // 简易 XML 解析（非完整 XML parser，只提取关键字段）
    let mut papers = Vec::new();
    for entry in text.split("<entry>").skip(1) {
        let id = extract_tag(entry, "id").unwrap_or_default();
        let title = extract_tag(entry, "title").unwrap_or_default();
        let summary = extract_tag(entry, "summary").unwrap_or_default();
        let published = extract_tag(entry, "published").unwrap_or_default();
        let categories: Vec<String> = entry.split("<category")
            .skip(1)
            .filter_map(|s| {
                let term = s.split("term=\"").nth(1)?;
                let rest = term.split('"').next()?;
                Some(rest.to_string())
            })
            .collect();

        let authors: Vec<String> = entry.split("<author>")
            .skip(1)
            .filter_map(|s| extract_tag(s, "name"))
            .collect();

        papers.push(ArxivPaper {
            id: id.trim().to_string(),
            title: clean_arxiv_text(&title),
            authors,
            summary: clean_arxiv_text(&summary)[..200.min(clean_arxiv_text(&summary).len())].to_string(),
            categories,
            published: published.trim().to_string(),
        });

        if papers.len() >= max_results as usize {
            break;
        }
    }

    Ok(papers)
}

fn extract_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = text.find(&open)?;
    let end = text[start..].find(&close)?;
    Some(text[start + open.len()..start + end].to_string())
}

fn clean_arxiv_text(s: &str) -> String {
    s.replace('\n', " ").replace("  ", " ").trim().to_string()
}
