//! aion-news: AI 趋势日报 — 多平台信息聚合与智能分析
//!
//! 数据源: GitHub 热榜 / Hacker News / ArXiv
//! 输出: 拟人化中文日报 (Markdown/JSON)

mod collector;
mod analyzer;
mod reporter;

use anyhow::{Context, Result};
use clap::Parser;
use crate::collector::github::fetch_star_history;

#[derive(Parser)]
#[command(
    name = "aion-news",
    version,
    about = "AI 趋势日报 — 多平台信息聚合与智能分析"
)]
struct Args {
    /// 分析周期: daily, weekly, monthly, yearly, all
    #[arg(short, long, default_value = "daily")]
    period: String,

    /// 查看指定仓库的 Star 趋势 (格式: owner/repo)
    #[arg(long)]
    trend: Option<String>,

    /// 输出格式: markdown, json
    #[arg(short, long, default_value = "markdown")]
    format: String,

    /// 最大返回数量
    #[arg(short, long, default_value_t = 10)]
    limit: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(ref repo) = args.trend {
        return trend_mode(repo, &args.format).await;
    }

    report_mode(&args).await
}

/// 趋势模式: 查看某仓库的 Star 历史
async fn trend_mode(repo: &str, format: &str) -> Result<()> {
    let history = fetch_star_history(repo, 30)
        .await
        .with_context(|| format!("获取 {repo} Star 历史失败"))?;

    match format {
        "json" => {
            let output = serde_json::json!({
                "repo": repo,
                "star_history": history.iter().map(|(date, stars)| {
                    serde_json::json!({ "date": date, "stars": stars })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            let chart = reporter::template::render_trend_chart(repo, &history);
            println!("{chart}");
        }
    }

    Ok(())
}

/// 日报模式: 采集 → 过滤 → 分析 → 报告
async fn report_mode(args: &Args) -> Result<()> {
    let period = &args.period;
    let hn_days = hn_days_for_period(period);

    // 并行采集三个数据源
    let (repos, hn_stories, arxiv_papers) = tokio::join!(
        collector::github::fetch_trending(period, args.limit),
        collector::hackernews::fetch_ai_stories(hn_days),
        collector::arxiv::fetch_recent_papers((args.limit * 3) as u32),
    );

    let mut repos = repos.context("GitHub 采集失败")?;
    let hn_stories = hn_stories.context("HN 采集失败")?;
    let arxiv_papers = arxiv_papers.context("ArXiv 采集失败")?;

    // AI 相关度过滤 + 排序
    filter_and_sort_by_relevance(&mut repos);

    // 生成日期字符串
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    match args.format.as_str() {
        "json" => print_json_report(&date, period, &repos, &hn_stories, &arxiv_papers)?,
        _ => {
            let report = reporter::template::render_daily_report(
                &date, &repos, &hn_stories, &arxiv_papers,
            );
            println!("{report}");
        }
    }

    Ok(())
}

/// 按 AI 相关度过滤并排序
fn filter_and_sort_by_relevance(repos: &mut Vec<collector::github::RepoEntry>) {
    // 过滤: 只保留 AI 相关
    repos.retain(|r| {
        analyzer::relevance::analyze(&r.full_name, &r.description, &r.topics, &r.language).is_ai_related
    });

    // 排序: forge 相关度高的在前
    repos.sort_by(|a, b| {
        let ra = analyzer::relevance::analyze(&a.full_name, &a.description, &a.topics, &a.language);
        let rb = analyzer::relevance::analyze(&b.full_name, &b.description, &b.topics, &b.language);
        let score = |r: &analyzer::relevance::RelevanceResult| match r.forge_relevance {
            analyzer::relevance::RelevanceLevel::High => 3,
            analyzer::relevance::RelevanceLevel::Medium => 2,
            analyzer::relevance::RelevanceLevel::Low => 1,
        };
        score(&rb).cmp(&score(&ra))
    });
}

fn hn_days_for_period(period: &str) -> u32 {
    match period {
        "daily" => 1,
        "weekly" => 7,
        "monthly" => 30,
        "yearly" | "all" => 90,
        _ => 7,
    }
}

/// JSON 格式输出报告
fn print_json_report(
    date: &str,
    period: &str,
    repos: &[collector::github::RepoEntry],
    hn_stories: &[collector::hackernews::HNStory],
    arxiv_papers: &[collector::arxiv::ArxivPaper],
) -> Result<()> {
    let repo_list: Vec<_> = repos
        .iter()
        .map(|r| {
            let rel = analyzer::relevance::analyze(&r.full_name, &r.description, &r.topics, &r.language);
            serde_json::json!({
                "name": r.full_name,
                "description": r.description,
                "stars": r.stars,
                "url": r.url,
                "language": r.language,
                "topics": r.topics,
                "relevance": {
                    "is_ai_related": rel.is_ai_related,
                    "tag": rel.tag,
                    "forge_relevance": format!("{:?}", rel.forge_relevance),
                    "matched_keywords": rel.matched_keywords,
                    "reason": rel.reason,
                }
            })
        })
        .collect();

    let hn_list: Vec<_> = hn_stories
        .iter()
        .map(|s| {
            serde_json::json!({
                "title": s.title,
                "url": s.url,
                "points": s.points,
                "author": s.author,
            })
        })
        .collect();

    let paper_list: Vec<_> = arxiv_papers
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "authors": p.authors,
                "published": p.published,
                "summary": truncate(&p.summary, 200),
            })
        })
        .collect();

    let output = serde_json::json!({
        "date": date,
        "period": period,
        "repos": repo_list,
        "hacker_news_stories": hn_list,
        "arxiv_papers": paper_list,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
