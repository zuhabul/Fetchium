//! Result Clustering Engine (RCE) — semantic topic grouping.
//!
//! Novel algorithm: After ranking, cluster results into semantic topic groups
//! using term-overlap similarity. Instead of a flat list, users see organized
//! facets of their query.
//!
//! Uses single-pass agglomerative clustering with term-overlap Jaccard
//! similarity. O(N²) but N is small (typically 10-30 results).

use crate::types::ResultItem;
use std::collections::HashSet;

/// A cluster of related results.
#[derive(Debug, Clone)]
pub struct ResultCluster {
    /// Human-readable label derived from common terms.
    pub label: String,
    /// Results in this cluster, in rank order.
    pub results: Vec<ResultItem>,
    /// Terms that are common across most results in this cluster.
    pub common_terms: Vec<String>,
}

/// Configuration for clustering.
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// Minimum Jaccard similarity to consider results as related.
    pub similarity_threshold: f64,
    /// Maximum number of clusters.
    pub max_clusters: usize,
    /// Minimum results per cluster (smaller clusters get merged into "Other").
    pub min_cluster_size: usize,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.15,
            max_clusters: 5,
            min_cluster_size: 2,
        }
    }
}

/// Cluster a list of ranked results into topic groups.
///
/// Results within each cluster maintain their relative ranking order.
pub fn cluster_results(results: &[ResultItem], config: &ClusterConfig) -> Vec<ResultCluster> {
    if results.is_empty() {
        return vec![];
    }

    if results.len() == 1 {
        return vec![ResultCluster {
            label: extract_label(&results[0]),
            results: results.to_vec(),
            common_terms: vec![],
        }];
    }

    // Extract term sets for each result
    let term_sets: Vec<HashSet<String>> = results.iter().map(extract_terms).collect();

    // Compute pairwise similarity matrix
    let n = results.len();
    let mut similarity = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let sim = jaccard_similarity(&term_sets[i], &term_sets[j]);
            similarity[i][j] = sim;
            similarity[j][i] = sim;
        }
    }

    // Greedy clustering: assign each result to the most similar cluster
    let mut assignments: Vec<Option<usize>> = vec![None; n];
    let mut clusters: Vec<Vec<usize>> = vec![];

    for i in 0..n {
        // Find the best existing cluster for this result
        let mut best_cluster = None;
        let mut best_sim = config.similarity_threshold;

        for (c_idx, cluster) in clusters.iter().enumerate() {
            // Average similarity to cluster members
            let avg_sim: f64 =
                cluster.iter().map(|&j| similarity[i][j]).sum::<f64>() / cluster.len() as f64;

            if avg_sim > best_sim {
                best_sim = avg_sim;
                best_cluster = Some(c_idx);
            }
        }

        match best_cluster {
            Some(c_idx) => {
                clusters[c_idx].push(i);
                assignments[i] = Some(c_idx);
            }
            None => {
                // Start a new cluster
                assignments[i] = Some(clusters.len());
                clusters.push(vec![i]);
            }
        }
    }

    // Build output clusters
    let mut output: Vec<ResultCluster> = clusters
        .iter()
        .map(|indices| {
            let cluster_results: Vec<ResultItem> =
                indices.iter().map(|&i| results[i].clone()).collect();

            let cluster_terms: Vec<&HashSet<String>> =
                indices.iter().map(|&i| &term_sets[i]).collect();

            let common = find_common_terms(&cluster_terms);
            let label = if common.is_empty() {
                extract_label(&cluster_results[0])
            } else {
                common
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            ResultCluster {
                label,
                results: cluster_results,
                common_terms: common,
            }
        })
        .collect();

    // Merge small clusters into "Other"
    let (large, small): (Vec<_>, Vec<_>) = output
        .into_iter()
        .partition(|c| c.results.len() >= config.min_cluster_size);

    output = large;

    if !small.is_empty() {
        let other_results: Vec<ResultItem> = small.into_iter().flat_map(|c| c.results).collect();

        if !other_results.is_empty() {
            output.push(ResultCluster {
                label: "Other".into(),
                results: other_results,
                common_terms: vec![],
            });
        }
    }

    // Limit to max clusters
    output.truncate(config.max_clusters);

    output
}

/// Extract significant terms from a result.
fn extract_terms(item: &ResultItem) -> HashSet<String> {
    let text = format!("{} {}", item.title, item.snippet).to_lowercase();
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .filter(|t| !is_common_word(t))
        .map(|t| t.to_string())
        .collect()
}

/// Jaccard similarity between two term sets.
fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Find terms common to most results in a cluster.
fn find_common_terms(term_sets: &[&HashSet<String>]) -> Vec<String> {
    if term_sets.is_empty() {
        return vec![];
    }

    let threshold = (term_sets.len() as f64 * 0.6).ceil() as usize;

    // Count term frequency across results
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for ts in term_sets {
        for term in ts.iter() {
            *counts.entry(term.as_str()).or_insert(0) += 1;
        }
    }

    let mut common: Vec<String> = counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(term, _)| term.to_string())
        .collect();

    common.sort();
    common
}

/// Extract a label from a single result.
fn extract_label(item: &ResultItem) -> String {
    let title = &item.title;
    if title.len() <= 40 {
        title.clone()
    } else {
        format!("{}...", &title[..37])
    }
}

/// Common words to exclude from clustering terms.
fn is_common_word(word: &str) -> bool {
    const COMMON: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "her", "was", "one", "our",
        "out", "has", "had", "how", "its", "may", "new", "now", "old", "see", "way", "who", "did",
        "get", "let", "say", "she", "too", "use", "this", "that", "with", "have", "from", "they",
        "been", "some", "when", "what", "your", "each", "make", "like", "just", "than", "them",
        "very", "will", "more", "also",
    ];
    COMMON.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, snippet: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: format!("https://{}.com", title.to_lowercase().replace(' ', "-")),
            snippet: snippet.into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: Some(0.5),
            published_date: None,
        }
    }

    #[test]
    fn empty_input_empty_output() {
        let clusters = cluster_results(&[], &ClusterConfig::default());
        assert!(clusters.is_empty());
    }

    #[test]
    fn single_result_single_cluster() {
        let results = vec![make_item("Rust Guide", "Learn Rust programming")];
        let clusters = cluster_results(&results, &ClusterConfig::default());
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].results.len(), 1);
    }

    #[test]
    fn similar_results_clustered_together() {
        let results = vec![
            make_item(
                "Rust Memory Safety",
                "Rust borrow checker prevents memory bugs in systems code",
            ),
            make_item(
                "Rust Ownership Model",
                "Ownership and borrowing in Rust for safe concurrency",
            ),
            make_item(
                "Rust Lifetimes Explained",
                "Understanding lifetimes and references in Rust compiler",
            ),
            make_item(
                "Python Data Science",
                "Pandas, NumPy, and matplotlib for data analysis",
            ),
            make_item(
                "Python Machine Learning",
                "Scikit-learn and TensorFlow for training ML models",
            ),
        ];

        let config = ClusterConfig {
            similarity_threshold: 0.08, // low threshold to detect topical grouping
            min_cluster_size: 2,
            ..Default::default()
        };

        let clusters = cluster_results(&results, &config);

        // Should have at least 2 clusters (Rust safety vs Python data)
        assert!(
            clusters.len() >= 2,
            "Expected at least 2 clusters, got {} with labels: {:?}",
            clusters.len(),
            clusters.iter().map(|c| &c.label).collect::<Vec<_>>()
        );
    }

    #[test]
    fn respects_max_clusters() {
        let results: Vec<ResultItem> = (0..20)
            .map(|i| {
                make_item(
                    &format!("Topic {i}"),
                    &format!("Unique content about subject {i}"),
                )
            })
            .collect();

        let config = ClusterConfig {
            max_clusters: 3,
            min_cluster_size: 1,
            ..Default::default()
        };

        let clusters = cluster_results(&results, &config);
        assert!(clusters.len() <= 3, "Should respect max_clusters");
    }

    #[test]
    fn jaccard_identical_sets() {
        let a: HashSet<String> = ["rust", "web"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["rust", "web"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 0.01);
    }

    #[test]
    fn jaccard_disjoint_sets() {
        let a: HashSet<String> = ["rust", "web"].iter().map(|s| s.to_string()).collect();
        let b: HashSet<String> = ["python", "django"].iter().map(|s| s.to_string()).collect();
        assert!((jaccard_similarity(&a, &b)).abs() < 0.01);
    }

    #[test]
    fn cluster_labels_not_empty() {
        let results = vec![
            make_item("Rust Programming", "Systems programming with Rust"),
            make_item("Rust Memory Safety", "Memory safety in Rust language"),
        ];
        let clusters = cluster_results(&results, &ClusterConfig::default());
        for cluster in &clusters {
            assert!(
                !cluster.label.is_empty(),
                "Cluster label should not be empty"
            );
        }
    }
}
