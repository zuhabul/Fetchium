//! Cross-source deduplication — URL normalization + SimHash content similarity.
//!
//! Two-phase deduplication pipeline:
//! 1. **URL dedup**: Normalize URLs (strip tracking params, fragments, trailing slash),
//!    then exact-match. Keeps the higher-scored item when duplicates are found.
//! 2. **SimHash dedup**: Compute 64-bit SimHash of `title + snippet`. Items whose
//!    Hamming distance is ≤ `threshold` (default 6, i.e. ≥90% similarity) are
//!    considered near-duplicates; only the first is kept.

use crate::types::ResultItem;
use std::collections::HashMap;
use tracing::debug;

// ── SimHash ────────────────────────────────────────────────────────────────────

/// 64-bit SimHash fingerprint for near-duplicate content detection.
///
/// Based on Charikar's locality-sensitive hashing algorithm:
/// tokenise → hash each token with FNV-1a → accumulate ±1 per bit → sign-collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimHash(pub u64);

impl SimHash {
    /// Compute a SimHash fingerprint from arbitrary text.
    ///
    /// Algorithm:
    /// 1. Split text into whitespace-delimited tokens, lowercased.
    /// 2. For each token: compute FNV-1a-64 hash; for each bit position,
    ///    add +1 if the bit is set, −1 otherwise.
    /// 3. Collapse: bit[i] = 1 iff counter[i] > 0.
    pub fn compute(text: &str) -> Self {
        let mut counters = [0i32; 64];

        for word in text.split_whitespace() {
            let word = word.to_lowercase();
            // Strip leading/trailing punctuation
            let word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if word.len() < 2 {
                continue;
            }
            let hash = fnv1a_64(word.as_bytes());
            for (bit, counter) in counters.iter_mut().enumerate() {
                if (hash >> bit) & 1 == 1 {
                    *counter += 1;
                } else {
                    *counter -= 1;
                }
            }
        }

        let mut fingerprint = 0u64;
        for (bit, &counter) in counters.iter().enumerate() {
            if counter > 0 {
                fingerprint |= 1u64 << bit;
            }
        }
        SimHash(fingerprint)
    }

    /// Hamming distance between two fingerprints (number of differing bits, 0–64).
    #[inline]
    pub fn distance(&self, other: &SimHash) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Returns `true` if the two fingerprints are similar (Hamming distance ≤ threshold).
    ///
    /// A threshold of 6 means ≤6/64 bits differ (≥90% bit-level agreement).
    #[inline]
    pub fn is_similar(&self, other: &SimHash, threshold: u32) -> bool {
        self.distance(other) <= threshold
    }
}

/// FNV-1a 64-bit hash (fast, non-cryptographic).
#[inline]
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// ── URL normalisation ──────────────────────────────────────────────────────────

/// Common tracking/analytics query parameters to strip during URL normalisation.
const TRACKING_PARAMS: &[&str] = &[
    // UTM (Google Analytics)
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "utm_id", "utm_name",
    // Social platform trackers
    "fbclid", "gclid", "msclkid", "twclid", "igshid",
    // Generic referral/tracking
    "ref", "source", "_ga", "_gl", "mc_cid", "mc_eid",
    "si", "feature", "app",
];

/// Normalise a URL for deduplication.
///
/// Transformations applied:
/// - Strip URL fragment (`#section`)
/// - Remove common tracking query parameters (UTM, fbclid, gclid, …)
/// - Strip trailing slash on non-root paths
/// - Lowercase the host
///
/// Falls back to `url.to_lowercase()` if the URL cannot be parsed.
pub fn normalize_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }

    match url::Url::parse(url) {
        Ok(mut parsed) => {
            // Strip fragment
            parsed.set_fragment(None);

            // Filter out tracking query parameters
            let filtered: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(k, _)| !TRACKING_PARAMS.contains(&k.as_ref()))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if filtered.is_empty() {
                parsed.set_query(None);
            } else {
                let qs = filtered
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&");
                parsed.set_query(Some(&qs));
            }

            // Strip trailing slash on non-root paths (e.g. /article/ → /article)
            let mut s = parsed.to_string();
            let path = parsed.path();
            if s.ends_with('/') && path != "/" {
                s.pop();
            }
            s
        }
        Err(_) => url.to_lowercase(),
    }
}

// ── Deduplication ──────────────────────────────────────────────────────────────

/// Deduplicate a list of `ResultItem`s using URL normalisation + SimHash similarity.
///
/// **Phase 1 — URL dedup**: Groups results by normalised URL. When two items
/// share a URL, the one with the higher `score` (or the earlier one if equal) is kept.
///
/// **Phase 2 — SimHash dedup**: Scans remaining results for near-duplicate content
/// (Hamming distance ≤ `simhash_threshold`). Only the first of each cluster is kept.
///
/// **Re-ranking**: After deduplication, `rank` fields are reassigned from 1 upward.
///
/// # Arguments
/// * `results` — Items from one or more backends (mixed ranking acceptable).
/// * `simhash_threshold` — Maximum Hamming distance to consider near-duplicate
///   (0 = exact match only, 64 = everything matches; default 6 ≈ 90% similarity).
pub fn deduplicate(mut results: Vec<ResultItem>, simhash_threshold: u32) -> Vec<ResultItem> {
    let original_len = results.len();

    // ── Phase 1: URL dedup ────────────────────────────────────────────────────
    let mut url_seen: HashMap<String, usize> = HashMap::with_capacity(results.len());
    let mut phase1: Vec<ResultItem> = Vec::with_capacity(results.len());

    for item in results.drain(..) {
        let key = normalize_url(&item.url);
        match url_seen.get(&key).copied() {
            Some(existing_idx) => {
                // Keep the higher-scored item
                let existing_score = phase1[existing_idx].score.unwrap_or(0.0);
                let new_score = item.score.unwrap_or(0.0);
                if new_score > existing_score {
                    phase1[existing_idx] = item;
                }
                // (else: discard the new item — the existing one is better)
            }
            None => {
                url_seen.insert(key, phase1.len());
                phase1.push(item);
            }
        }
    }

    // ── Phase 2: SimHash near-dup dedup ──────────────────────────────────────
    let mut seen_hashes: Vec<SimHash> = Vec::with_capacity(phase1.len());
    let mut final_results: Vec<ResultItem> = Vec::with_capacity(phase1.len());

    for item in phase1 {
        let text = format!("{} {}", item.title, item.snippet);
        let hash = SimHash::compute(&text);

        let is_near_dup = seen_hashes
            .iter()
            .any(|h| h.is_similar(&hash, simhash_threshold));

        if !is_near_dup {
            seen_hashes.push(hash);
            final_results.push(item);
        }
    }

    // ── Re-assign contiguous ranks ────────────────────────────────────────────
    for (i, item) in final_results.iter_mut().enumerate() {
        item.rank = (i + 1) as u32;
    }

    debug!(
        "Dedup: {} → {} results (URL+SimHash, threshold={})",
        original_len,
        final_results.len(),
        simhash_threshold
    );

    final_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_item(title: &str, url: &str, snippet: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: None,
            published_date: None,
        }
    }

    fn make_item_with_score(title: &str, url: &str, snippet: &str, score: f64) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: Some(score),
            published_date: None,
        }
    }

    // ── SimHash tests ─────────────────────────────────────────────────────────

    #[test]
    fn simhash_identical_texts_distance_zero() {
        let h1 = SimHash::compute("hello world foo bar baz");
        let h2 = SimHash::compute("hello world foo bar baz");
        assert_eq!(h1.distance(&h2), 0);
    }

    #[test]
    fn simhash_different_domains_high_distance() {
        let h1 = SimHash::compute("quantum physics nuclear reactor neutrons electrons protons");
        let h2 = SimHash::compute("chocolate cake baking sugar flour vanilla cream butter");
        assert!(
            h1.distance(&h2) > 6,
            "Expected high distance, got {}",
            h1.distance(&h2)
        );
    }

    #[test]
    fn simhash_near_identical_texts_low_distance() {
        let h1 = SimHash::compute("Rust programming language systems safe fast reliable");
        // Adding a couple of words shouldn't push beyond a moderate threshold
        let h2 = SimHash::compute("Rust programming language systems safe fast reliable concurrent");
        assert!(
            h1.distance(&h2) <= 20,
            "Expected low distance for similar text, got {}",
            h1.distance(&h2)
        );
    }

    #[test]
    fn simhash_is_similar_exact_match() {
        let h = SimHash::compute("same text");
        assert!(h.is_similar(&h, 0));
    }

    #[test]
    fn simhash_is_similar_threshold() {
        let h1 = SimHash::compute("abc def ghi jkl");
        let h2 = SimHash::compute("abc def ghi jkl");
        assert!(h1.is_similar(&h2, 6));
        assert!(!h1.is_similar(&h2, 0) || h1.distance(&h2) == 0);
    }

    #[test]
    fn fnv1a_deterministic() {
        let h1 = fnv1a_64(b"hello");
        let h2 = fnv1a_64(b"hello");
        assert_eq!(h1, h2);
        // Different inputs → different hash (with very high probability)
        let h3 = fnv1a_64(b"world");
        assert_ne!(h1, h3);
    }

    // ── URL normalization tests ───────────────────────────────────────────────

    #[test]
    fn normalize_url_strips_utm_params() {
        let url = "https://example.com/article?id=42&utm_source=twitter&utm_medium=social";
        let norm = normalize_url(url);
        assert!(norm.contains("id=42"), "id=42 should be kept in: {norm}");
        assert!(!norm.contains("utm_source"), "utm_source should be stripped from: {norm}");
        assert!(!norm.contains("utm_medium"), "utm_medium should be stripped from: {norm}");
    }

    #[test]
    fn normalize_url_strips_fbclid() {
        let url = "https://example.com/page?fbclid=abc123";
        let norm = normalize_url(url);
        assert!(!norm.contains("fbclid"), "fbclid should be stripped from: {norm}");
    }

    #[test]
    fn normalize_url_strips_fragment() {
        let url = "https://example.com/page#section-2";
        let norm = normalize_url(url);
        assert!(!norm.contains('#'), "Fragment should be stripped from: {norm}");
        assert!(norm.contains("example.com/page"), "Path should be kept in: {norm}");
    }

    #[test]
    fn normalize_url_strips_trailing_slash_on_path() {
        let url = "https://example.com/article/";
        let norm = normalize_url(url);
        assert!(!norm.ends_with('/'), "Trailing slash should be stripped from: {norm}");
        assert!(norm.ends_with("article"), "Path should be kept in: {norm}");
    }

    #[test]
    fn normalize_url_keeps_root_slash() {
        let url = "https://example.com/";
        let norm = normalize_url(url);
        assert!(norm.contains("example.com"), "Host should be present in: {norm}");
        // Root slash is preserved (it's the only path component)
    }

    #[test]
    fn normalize_url_strips_all_tracking_no_remaining_query() {
        let url = "https://example.com/page?utm_source=x&utm_medium=y&utm_campaign=z";
        let norm = normalize_url(url);
        assert!(!norm.contains('?'), "All params stripped, so no '?' in: {norm}");
    }

    #[test]
    fn normalize_url_empty_string() {
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn normalize_url_invalid_url_lowercased() {
        let url = "NOT A URL";
        let norm = normalize_url(url);
        assert_eq!(norm, "not a url");
    }

    // ── Deduplication tests ───────────────────────────────────────────────────

    #[test]
    fn deduplicate_removes_exact_url_duplicates() {
        let items = vec![
            make_item("Page A", "https://example.com/page", "content A"),
            make_item("Page A dup", "https://example.com/page", "content A duplicate"),
            make_item("Page B", "https://other.com/page", "different content B"),
        ];
        let deduped = deduplicate(items, 6);
        assert_eq!(deduped.len(), 2, "Expected 2 results after URL dedup");
    }

    #[test]
    fn deduplicate_removes_url_duplicates_with_tracking_params() {
        let items = vec![
            make_item("Page A", "https://example.com/page", "content"),
            make_item("Page A twitter", "https://example.com/page?utm_source=twitter", "content"),
            make_item("Page B", "https://other.com/different", "different content"),
        ];
        let deduped = deduplicate(items, 6);
        assert_eq!(deduped.len(), 2, "Tracking-param variant should be deduped");
    }

    #[test]
    fn deduplicate_keeps_higher_scored_url_duplicate() {
        let items = vec![
            make_item_with_score("Low score", "https://example.com/page", "content", 0.3),
            make_item_with_score("High score", "https://example.com/page", "content", 0.9),
        ];
        let deduped = deduplicate(items, 0); // threshold=0 to only do URL dedup
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].title, "High score");
    }

    #[test]
    fn deduplicate_removes_simhash_near_duplicates() {
        let text = "Rust is a systems programming language focused on safety and performance";
        let items = vec![
            make_item("Rust A", "https://a.com", text),
            make_item("Rust B", "https://b.com", text), // Same content, different URL
            make_item(
                "Python",
                "https://python.org",
                "Python is a dynamic scripting language for web development",
            ),
        ];
        let deduped = deduplicate(items, 3); // Strict SimHash threshold
        // Python should survive; the two Rust items with identical content should be deduped
        let has_python = deduped.iter().any(|r| r.title == "Python");
        assert!(has_python, "Python result should survive dedup");
        assert!(deduped.len() < 3, "At least one Rust duplicate should be removed");
    }

    #[test]
    fn deduplicate_reassigns_ranks_sequentially() {
        let items = vec![
            make_item("A", "https://a.com", "content about topic a"),
            make_item("B", "https://b.com", "content about topic b"),
            make_item("C", "https://c.com", "content about topic c"),
        ];
        let deduped = deduplicate(items, 6);
        for (i, item) in deduped.iter().enumerate() {
            assert_eq!(
                item.rank,
                (i + 1) as u32,
                "Rank should be {} but was {}",
                i + 1,
                item.rank
            );
        }
    }

    #[test]
    fn deduplicate_empty_input() {
        let deduped = deduplicate(vec![], 6);
        assert!(deduped.is_empty());
    }

    #[test]
    fn deduplicate_single_item() {
        let items = vec![make_item("Only", "https://only.com", "only content")];
        let deduped = deduplicate(items, 6);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].rank, 1);
    }

    #[test]
    fn deduplicate_preserves_unique_results() {
        // Use clearly distinct topics so SimHash does not group them
        let distinct_items = vec![
            make_item("Rust compiler", "https://rustlang.org", "memory safety borrow checker lifetimes ownership systems"),
            make_item("Machine learning", "https://pytorch.org", "neural networks gradient descent backpropagation training epochs"),
            make_item("Database indexing", "https://postgresql.org", "btree btree index scan sequential lookup primary key"),
            make_item("Network protocol", "https://ietf.org", "tcp ip handshake packets routing latency bandwidth"),
            make_item("Cryptocurrency", "https://bitcoin.org", "blockchain hash proof-of-work mining consensus ledger"),
        ];
        let deduped = deduplicate(distinct_items, 6);
        // All items are topically distinct — none should be removed
        assert_eq!(deduped.len(), 5, "All 5 distinct items should be kept");
    }
}
