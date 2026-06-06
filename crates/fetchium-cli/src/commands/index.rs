//! `fetchium index` — manage the local document index.

use anyhow::{Context, Result};
use fetchium_core::config::FetchiumConfig;
use fetchium_core::extract::pipeline;
use fetchium_core::http::client::HttpClient;
use fetchium_core::index::document::IndexedDocument;
use fetchium_core::index::store::DocumentStore;
use fetchium_core::token::qatbe::extract_with_budget;

use crate::cli::{Format, IndexAction, IndexArgs};

pub async fn run(args: IndexArgs, config: &FetchiumConfig, format: Format) -> Result<()> {
    let data_dir = config.data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("index.db");
    let store = DocumentStore::new(&db_path)?;

    match args.action {
        IndexAction::Add { url, query } => {
            let http = HttpClient::new(config).context("Failed to build HTTP client")?;
            let query_str = query.as_deref().unwrap_or("");

            let fetch_result = http
                .fetch(&url)
                .await
                .with_context(|| format!("Failed to fetch {url}"))?;
            let extracted = pipeline::extract(&fetch_result.body, &fetch_result.url);
            let qatbe = extract_with_budget(&extracted, query_str, 4000);

            // Combine all segment text for storage
            let content: String = qatbe
                .segments
                .iter()
                .filter_map(|s| s.content.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");

            let domain = url::Url::parse(&url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();

            let content_hash = DocumentStore::content_hash(&content);

            let doc = IndexedDocument {
                id: 0,
                url: url.clone(),
                title: extracted.title.clone(),
                content,
                domain,
                fetched_at: chrono::Utc::now(),
                content_hash,
                embedding: None,
            };

            let (id, _changed) = store.upsert(&doc)?;
            println!("Indexed {url} (id={id}, {} tokens)", qatbe.tokens_used);
        }

        IndexAction::Search { query, max_results } => {
            let results = store.search(&query, max_results)?;

            if results.is_empty() {
                println!("No results found for {:?}", query);
            } else {
                match format {
                    Format::Json => {
                        let out: Vec<_> = results
                            .iter()
                            .map(|d| {
                                serde_json::json!({
                                    "id": d.id,
                                    "url": d.url,
                                    "title": d.title,
                                    "domain": d.domain,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::to_string_pretty(&out)?);
                    }
                    _ => {
                        for (i, doc) in results.iter().enumerate() {
                            println!(
                                "{}. [{}] {}\n   {}\n",
                                i + 1,
                                doc.domain,
                                doc.title,
                                doc.url
                            );
                        }
                    }
                }
            }
        }

        IndexAction::Stats => {
            let stats = store.stats()?;
            println!("Documents:  {}", stats.document_count);
            println!("Embedded:   {}", stats.embedded_count);
        }

        IndexAction::Clear => {
            let n = store.clear()?;
            println!("Index cleared ({n} documents removed).");
        }
    }

    Ok(())
}
