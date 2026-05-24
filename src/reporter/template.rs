//! 报告模板 — 拟人化排版输出
//!
//! 生成带 emoji、表格、趋势图（mermaid）的可读报告

use crate::analyzer::relevance::RelevanceLevel;
use crate::collector::github::RepoEntry;
use crate::analyzer::relevance;

/// 生成完整的日报 Markdown（含 mermaid 趋势图）
pub fn render_daily_report(
    date: &str,
    repos: &[RepoEntry],
    hq_stories: &[crate::collector::hackernews::HNStory],
    papers: &[crate::collector::arxiv::ArxivPaper],
) -> String {
    let mut md = String::new();

    // 标题
    md.push_str(&format!("# 📡 AI 趋势日报 · {}\n\n", date));

    // GitHub 热榜
    md.push_str("## 🔥 GitHub 趋势\n\n");

    let mut has_ai = false;
    for repo in repos {
        let r = relevance::analyze(&repo.full_name, &repo.description, &repo.topics, &repo.language);
        let tag = if r.is_ai_related { "🤖" } else { "  " };
        let forge_tag = match r.forge_relevance {
            RelevanceLevel::High => " 🔥 forge 直接相关",
            RelevanceLevel::Medium => " ⚡ forge 参考",
            RelevanceLevel::Low => "",
        };
        md.push_str(&format!("{} **{}** ⭐{}{}\n", tag, repo.full_name, repo.stars, forge_tag));
        if !repo.description.is_empty() {
            md.push_str(&format!("   {}\n", &repo.description[..repo.description.len().min(80)]));
        }
        md.push_str("\n");
        has_ai = true;
    }

    if !has_ai {
        md.push_str("   本期未发现 AI 相关新项目。\n\n");
    }

    // HN 热帖
    md.push_str("## 🌐 Hacker News 热议\n\n");
    for story in hq_stories.iter().take(5) {
        let url = story.url.as_deref().unwrap_or("https://news.ycombinator.com");
        md.push_str(&format!("- [{}]({}) ({} 👍)\n", story.title, url, story.points));
    }
    md.push_str("\n");

    // ArXiv 论文
    md.push_str("## 📄 ArXiv 新论文\n\n");
    for paper in papers.iter().take(3) {
        md.push_str(&format!("- **{}** — {}...\n", paper.title, paper.summary));
    }
    md.push_str("\n");

    // forge 行动建议
    md.push_str("## 🎯 forge 行动项\n\n");
    md.push_str("_由 aion-news 自动生成_\n");

    md
}

/// 生成趋势线 mermaid 图表
pub fn render_trend_chart(repo: &str, history: &[(String, u64)]) -> String {
    let mut md = format!("### 📈 {} 趋势线\n\n```mermaid\n", repo);
    md.push_str("  linechart\n");
    for (date, stars) in history {
        md.push_str(&format!("    {} {}\n", date, stars));
    }
    md.push_str("  ```\n");
    md
}
