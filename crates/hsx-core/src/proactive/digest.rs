//! Intelligent digest builder — weekly/daily topic summaries (PRD §33.3).

use crate::error::HsxResult;
use serde::{Deserialize, Serialize};

/// A compiled digest of recent research findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Digest {
    pub period: DigestPeriod,
    pub topics: Vec<String>,
    pub sections: Vec<DigestSection>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl DigestPeriod {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "daily" => DigestPeriod::Daily,
            "monthly" => DigestPeriod::Monthly,
            _ => DigestPeriod::Weekly,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            DigestPeriod::Daily => "Daily",
            DigestPeriod::Weekly => "Weekly",
            DigestPeriod::Monthly => "Monthly",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSection {
    pub topic: String,
    pub headline: String,
    pub items: Vec<DigestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published: Option<String>,
}

/// Builds digest from a map of topic → result items.
pub struct DigestBuilder {
    pub period: DigestPeriod,
    pub topics: Vec<String>,
    sections: Vec<DigestSection>,
}

impl DigestBuilder {
    pub fn new(period: DigestPeriod, topics: Vec<String>) -> Self {
        Self {
            period,
            topics,
            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, topic: &str, items: Vec<crate::types::ResultItem>) {
        if items.is_empty() {
            return;
        }
        let headline = format!("{} updates in \"{}\"", items.len(), topic);
        let digest_items: Vec<DigestItem> = items
            .into_iter()
            .map(|r| DigestItem {
                title: r.title,
                url: r.url,
                snippet: r.snippet,
                published: r.published_date,
            })
            .collect();
        self.sections.push(DigestSection {
            topic: topic.to_string(),
            headline,
            items: digest_items,
        });
    }

    pub fn build(self) -> Digest {
        Digest {
            period: self.period,
            topics: self.topics,
            sections: self.sections,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Digest {
    pub fn to_markdown(&self) -> String {
        let mut md = format!(
            "# {} HyperSearchX Digest\n_Generated: {}_\n\n",
            self.period.label(),
            self.generated_at
        );
        if self.sections.is_empty() {
            md.push_str("_No new findings for your topics this period._\n");
            return md;
        }
        for section in &self.sections {
            md.push_str(&format!("## {}\n_{}_\n\n", section.topic, section.headline));
            for item in &section.items {
                md.push_str(&format!(
                    "- **[{}]({})** — {}\n",
                    item.title, item.url, item.snippet
                ));
            }
            md.push('\n');
        }
        md
    }

    pub fn to_json(&self) -> HsxResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_markdown_empty() {
        let d = DigestBuilder::new(DigestPeriod::Weekly, vec!["rust".into()]).build();
        let md = d.to_markdown();
        assert!(md.contains("No new findings"));
    }
}
