//! AI 相关度分析
//!
//! 对采集到的项目进行关联度评分，标注 forge 相关度

use serde::{Deserialize, Serialize};

const AI_KEYWORDS: &[(&str, &str, &str)] = &[
    ("ai", "AI", "一般 AI"),
    ("agent", "AI Agent", "智能体"),
    ("claude", "Claude/Anthropic", "Anthropic 生态"),
    ("anthropic", "Anthropic", "Anthropic 生态"),
    ("openai", "OpenAI", "OpenAI 生态"),
    ("gpt", "GPT", "大语言模型"),
    ("llm", "LLM", "大语言模型"),
    ("mcp", "MCP", "工具协议 — 与 forge 直接相关"),
    ("skill", "Skill", "技能系统 — 与 forge 直接相关"),
    ("coding", "Coding Agent", "编码智能体 — 与 forge 直接相关"),
    ("automation", "Automation", "自动化"),
    ("rag", "RAG", "RAG"),
    ("embedding", "Embedding", "嵌入"),
    ("prompt", "Prompt", "提示工程"),
    ("function-call", "Function Calling", "工具调用 — 与 forge 直接相关"),
    ("tool-use", "Tool Use", "工具使用 — 与 forge 直接相关"),
    ("codex", "Codex", "OpenAI Codex"),
    ("copilot", "Copilot", "GitHub Copilot"),
    ("rust", "Rust", "同语言生态"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceResult {
    pub is_ai_related: bool,
    pub tag: String,
    pub forge_relevance: RelevanceLevel,
    pub matched_keywords: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelevanceLevel {
    High,
    Medium,
    Low,
}

/// 分析一个项目的 AI 相关度和 forge 关联度
pub fn analyze(name: &str, description: &str, topics: &[String], language: &str) -> RelevanceResult {
    let combined = format!("{} {} {} {} {:?}", name, description, language, topics.join(" "), topics).to_lowercase();

    let mut matched_keywords: Vec<String> = Vec::new();
    let mut tag = String::new();
    let mut forge_score = 0i32;

    for (kw, label, _forge_tag) in AI_KEYWORDS {
        if combined.contains(kw) {
            matched_keywords.push(kw.to_string());
            if tag.is_empty() {
                tag = label.to_string();
            }
            // forge 直接相关的关键词加分
            match *kw {
                "mcp" | "skill" | "coding" | "function-call" | "tool-use" | "rust" => forge_score += 2,
                "agent" | "llm" | "claude" => forge_score += 1,
                _ => forge_score += 0,
            }
        }
    }

    let is_ai_related = !matched_keywords.is_empty();

    let forge_relevance = if forge_score >= 4 {
        RelevanceLevel::High
    } else if forge_score >= 2 {
        RelevanceLevel::Medium
    } else if is_ai_related {
        RelevanceLevel::Low
    } else {
        RelevanceLevel::Low
    };

    let reason = match forge_relevance {
        RelevanceLevel::High => "与 forge 直接相关，建议关注".to_string(),
        RelevanceLevel::Medium => "与 forge 业务相关，可参考".to_string(),
        RelevanceLevel::Low => if is_ai_related { "AI 领域项目".to_string() } else { "非 AI 项目".to_string() },
    };

    RelevanceResult {
        is_ai_related,
        tag,
        forge_relevance,
        matched_keywords,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_keyword_high_relevance() {
        let r = analyze("forge-mcp-server", "MCP tool server for AI agents", &[], "Rust");
        assert_eq!(r.forge_relevance, RelevanceLevel::High);
    }

    #[test]
    fn test_coding_agent_medium_relevance() {
        let r = analyze("devin", "AI coding agent", &[], "Python");
        assert_eq!(r.forge_relevance, RelevanceLevel::Medium);
    }

    #[test]
    fn test_non_ai_low_relevance() {
        let r = analyze("linux", "OS kernel", &["kernel".to_string(), "c".to_string()], "C");
        assert!(!r.is_ai_related);
    }
}
