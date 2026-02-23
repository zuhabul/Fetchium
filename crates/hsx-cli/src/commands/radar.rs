//! `hsx radar` — personalized research radar (PRD §33).

use colored::Colorize;
use hsx_core::intelligence::intelligence_data_dir;
use hsx_core::intelligence::pie::qpm::QueryPredictionModel;

pub fn run(limit: usize) -> anyhow::Result<()> {
    let db_path = intelligence_data_dir().join("query_patterns.db");
    if !db_path.exists() {
        println!(
            "{} No search history yet. Run some searches first.",
            "i".blue()
        );
        return Ok(());
    }

    let qpm = QueryPredictionModel::new(&db_path)?;
    let topics = qpm.top_topics(10)?;

    if topics.is_empty() {
        println!(
            "{} No radar suggestions yet — run more searches to build history.",
            "i".blue()
        );
        return Ok(());
    }

    println!("{}\n", "Research Radar".bold().cyan());

    for (i, (topic, freq)) in topics.iter().enumerate().take(limit) {
        let relevance = 1.0_f64 / (1.0 + i as f64 * 0.1);
        let query = format!("{topic} latest developments 2026");
        println!(
            "  {}. {} {}",
            i + 1,
            query.bold(),
            format!("({:.0}%)", relevance * 100.0).dimmed()
        );
        println!(
            "     Topic: {} \u{2014} You've researched this {} time(s)",
            topic,
            freq.to_string().dimmed()
        );
        println!();
    }

    Ok(())
}
