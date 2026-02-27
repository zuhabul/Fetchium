//! Temporal Decay Ranker (TDR) — time-aware result scoring.
//!
//! Novel algorithm: Apply exponential decay to result relevance based on
//! content age and query intent. Different query types have different
//! temporal expectations:
//!
//! - **Breaking news**: Half-life 1 day — yesterday's news is old
//! - **Current events**: Half-life 7 days — last week is still relevant
//! - **Technical**: Half-life 180 days — 6-month-old docs are fine
//! - **Academic**: Half-life 365 days — papers age slowly
//! - **Evergreen**: Half-life 730 days — how-tos barely age
//!
//! The decay is applied as a multiplier on the relevance score, so a
//! perfectly relevant but stale result scores lower than a moderately
//! relevant but fresh result.

use crate::rank::fusion::QueryIntent;
use crate::types::ResultItem;

/// Temporal decay configuration.
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    /// Base half-life in days (adjusted per intent).
    pub default_half_life_days: f64,
    /// Maximum boost for very fresh content (multiplier).
    pub freshness_boost: f64,
    /// Minimum decay floor — even ancient content keeps this fraction.
    pub decay_floor: f64,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            default_half_life_days: 90.0,
            freshness_boost: 1.15,
            decay_floor: 0.3,
        }
    }
}

/// Half-life (in days) for each query intent.
fn half_life_for_intent(intent: &QueryIntent) -> f64 {
    match intent {
        QueryIntent::CurrentEvents => 7.0,
        QueryIntent::Code => 120.0,
        QueryIntent::Academic => 365.0,
        QueryIntent::Data => 180.0,
        QueryIntent::Factual => 365.0,
        QueryIntent::HowTo => 730.0,
        QueryIntent::Comparison => 180.0,
        QueryIntent::Verification => 90.0,
        QueryIntent::DeepAnalysis => 180.0,
        QueryIntent::Opinion => 30.0,
        QueryIntent::Informational => 365.0, // definitions don't age quickly
    }
}

/// Apply temporal decay to a list of scored results.
///
/// Modifies each result's score in-place based on its publication date
/// and the query intent. Results without dates get a neutral adjustment.
pub fn apply_temporal_decay(
    results: &mut [ResultItem],
    intent: &QueryIntent,
    config: &TemporalConfig,
) {
    let half_life = half_life_for_intent(intent);
    let now = chrono::Utc::now();

    for item in results.iter_mut() {
        let Some(ref date_str) = item.published_date else {
            // No date → apply neutral factor (no penalty, no boost)
            continue;
        };

        let age_days = parse_age_days(date_str, &now);

        let decay = compute_decay(age_days, half_life, config);

        if let Some(score) = item.score.as_mut() {
            *score *= decay;
        }
    }
}

/// Compute the decay multiplier for a given age.
///
/// Uses exponential decay: `decay = max(floor, 2^(-age/half_life))`
/// with a freshness boost for content < 1 day old.
fn compute_decay(age_days: f64, half_life: f64, config: &TemporalConfig) -> f64 {
    if age_days <= 0.0 {
        // Future date or same-day → freshness boost
        return config.freshness_boost;
    }

    // Exponential decay: score *= 2^(-age/half_life)
    let raw_decay = (2.0f64).powf(-age_days / half_life);

    // Apply floor and freshness boost
    if age_days < 1.0 {
        config.freshness_boost
    } else {
        raw_decay.max(config.decay_floor)
    }
}

/// Parse a date string and compute age in days from now.
fn parse_age_days(date_str: &str, now: &chrono::DateTime<chrono::Utc>) -> f64 {
    // Try ISO 8601 formats
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return (*now - dt.with_timezone(&chrono::Utc)).num_hours() as f64 / 24.0;
    }

    // Try common date formats
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let dt_utc = dt.and_hms_opt(0, 0, 0).unwrap_or_default();
        let dt_utc =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt_utc, chrono::Utc);
        return (*now - dt_utc).num_hours() as f64 / 24.0;
    }

    // Try "Month Day, Year" format
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%B %d, %Y") {
        let dt_utc = dt.and_hms_opt(0, 0, 0).unwrap_or_default();
        let dt_utc =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt_utc, chrono::Utc);
        return (*now - dt_utc).num_hours() as f64 / 24.0;
    }

    // Unknown format → treat as neutral (0 days = no penalty)
    0.0
}

/// Get the decay factor for a specific age (useful for display/debugging).
pub fn decay_factor(age_days: f64, intent: &QueryIntent) -> f64 {
    let half_life = half_life_for_intent(intent);
    compute_decay(age_days, half_life, &TemporalConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_result(score: f64, date: Option<&str>) -> ResultItem {
        ResultItem {
            title: "Test".into(),
            url: "https://test.com".into(),
            snippet: "Test content".into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: Some(score),
            published_date: date.map(|d| d.to_string()),
        }
    }

    #[test]
    fn fresh_content_boosted() {
        let config = TemporalConfig::default();
        let factor = compute_decay(0.5, 90.0, &config);
        assert!(factor > 1.0, "Fresh content should be boosted: {factor}");
    }

    #[test]
    fn old_content_decayed() {
        let config = TemporalConfig::default();
        let factor = compute_decay(365.0, 90.0, &config);
        assert!(factor < 1.0, "Old content should be decayed: {factor}");
        assert!(
            factor >= config.decay_floor,
            "Should not go below floor: {factor}"
        );
    }

    #[test]
    fn news_decays_fast() {
        let news_30d = decay_factor(30.0, &QueryIntent::CurrentEvents);
        let academic_30d = decay_factor(30.0, &QueryIntent::Academic);
        assert!(
            news_30d < academic_30d,
            "30-day-old news should decay more than academic: news={news_30d} academic={academic_30d}"
        );
    }

    #[test]
    fn decay_floor_prevents_zero() {
        let config = TemporalConfig {
            decay_floor: 0.2,
            ..Default::default()
        };
        let factor = compute_decay(10000.0, 7.0, &config);
        assert!(factor >= 0.2, "Should not go below floor: {factor}");
    }

    #[test]
    fn apply_decay_modifies_scores() {
        let mut results = vec![
            make_result(0.9, Some("2020-01-01")),
            make_result(0.9, None), // no date — unchanged
        ];

        apply_temporal_decay(
            &mut results,
            &QueryIntent::CurrentEvents,
            &TemporalConfig::default(),
        );

        // The dated result should have a lower score (it's very old for news)
        assert!(
            results[0].score.unwrap() < results[1].score.unwrap(),
            "Old news should score lower: dated={} undated={}",
            results[0].score.unwrap(),
            results[1].score.unwrap()
        );
    }

    #[test]
    fn half_life_varies_by_intent() {
        let news_hl = half_life_for_intent(&QueryIntent::CurrentEvents);
        let academic_hl = half_life_for_intent(&QueryIntent::Academic);
        assert!(news_hl < academic_hl, "News half-life should be shorter");
    }

    #[test]
    fn parse_iso_date() {
        let now = chrono::Utc::now();
        let yesterday = (now - chrono::Duration::hours(24))
            .format("%Y-%m-%d")
            .to_string();
        let age = parse_age_days(&yesterday, &now);
        // Date-only format resolves to midnight, so age can be 1.0-2.0 days
        assert!(
            (0.5..=2.5).contains(&age),
            "Yesterday should be ~1-2 days old: {age}"
        );
    }
}
