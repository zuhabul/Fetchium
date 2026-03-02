//! `fetchium digest` — intelligent research digest (PRD §33).

use colored::Colorize;
use fetchium_core::proactive::digest::{DigestBuilder, DigestPeriod};
use fetchium_core::types::ResultItem;

pub fn run(period: &str, topics: &[String], output: Option<&str>) -> anyhow::Result<()> {
    if topics.is_empty() {
        eprintln!("Specify at least one topic with --topics \"topic1,topic2\"");
        return Ok(());
    }

    let period_enum = DigestPeriod::parse(period);
    let mut builder = DigestBuilder::new(period_enum, topics.to_vec());

    // Placeholder: in production this would run live searches per topic.
    // Build digest with stub entries per topic to show the pipeline working.
    for topic in topics {
        let stub = ResultItem {
            title: format!("Latest on {topic}"),
            url: format!("https://example.com/{topic}"),
            snippet: format!("Recent developments in {topic} — search live for real results."),
            rank: 1,
            backend: fetchium_core::types::BackendId::DuckDuckGo,
            score: None,
            published_date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        };
        builder.add_section(topic, vec![stub]);
    }

    let digest = builder.build();
    let md = digest.to_markdown();

    match output {
        Some(path) => {
            std::fs::write(path, &md)?;
            println!("{} Digest saved to {path}", "OK".green());
        }
        None => println!("{}", md),
    }
    Ok(())
}
