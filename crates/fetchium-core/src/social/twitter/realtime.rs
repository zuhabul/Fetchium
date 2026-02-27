//! Realtime Twitter/X monitoring via periodic polling.

use crate::http::client::HttpClient;
use crate::social::twitter::search::search_tweets;
use crate::social::twitter::sentiment::{analyze_sentiment, SentimentResult};
use crate::social::twitter::types::{Tweet, TwitterPipelineConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc;

/// Configuration for realtime monitoring.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    pub query: String,
    pub interval_secs: u64,
    pub sentiment: bool,
    pub max_per_poll: usize,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            interval_secs: 120,
            sentiment: true,
            max_per_poll: 20,
        }
    }
}

/// A new tweet detected during monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweetUpdate {
    pub tweet: Tweet,
    pub sentiment: Option<SentimentResult>,
    pub is_new: bool,
}

/// Start monitoring for tweets matching query, sending new results to the channel.
pub async fn monitor_stream(
    config: RealtimeConfig,
    http: HttpClient,
) -> mpsc::Receiver<TweetUpdate> {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut seen_ids: HashSet<String> = HashSet::new();
        let pipeline_config = TwitterPipelineConfig {
            query: config.query.clone(),
            max_tweets: config.max_per_poll,
            ..Default::default()
        };

        loop {
            match search_tweets(&config.query, config.max_per_poll, &pipeline_config, &http).await {
                Ok(tweets) => {
                    for tweet in tweets {
                        if seen_ids.contains(&tweet.id) {
                            continue;
                        }
                        seen_ids.insert(tweet.id.clone());

                        let sentiment = if config.sentiment {
                            Some(analyze_sentiment(&tweet.text))
                        } else {
                            None
                        };

                        let update = TweetUpdate {
                            tweet,
                            sentiment,
                            is_new: true,
                        };

                        if tx.send(update).await.is_err() {
                            return; // receiver dropped
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Monitor poll failed: {e}");
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(config.interval_secs)).await;
        }
    });

    rx
}
