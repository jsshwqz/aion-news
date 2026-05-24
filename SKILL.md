---
name: aion-news
description: "AI 趋势日报 — 每天 9:09 自动采集 GitHub/HN/ArXiv 趋势，生成拟人化分析报告。用 aion-news 查看当日 AI 新项目、热门讨论、研究论文。"
---

# aion-news

AI 趋势日报：多平台信息聚合与智能分析。

## 用法

```bash
# 昨天日报
aion-news

# 周报
aion-news --period weekly

# 查看项目趋势线
aion-news --trend smallcode

# JSON 输出
aion-news --format json
```

## 数据源

- **GitHub** — 日/周/月/年/总榜 + Star 趋势
- **Hacker News** — AI 相关热帖
- **ArXiv** — 最新 AI/Agent 论文

## 定时触发

每天 9:09 通过 Claude Code schedule 远程触发器自动执行。
