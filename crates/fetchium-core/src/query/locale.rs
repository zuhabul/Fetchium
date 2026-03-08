//! Query locale detection — infers the best proxy country for a search query.
//!
//! Two-stage pipeline (fast, zero external deps):
//! 1. Country/city mention extraction ("in Paris" → "fr", "Tokyo" → "jp")
//! 2. Script & language-word detection (CJK→jp/cn, Cyrillic→ru, French words→fr)
//!
//! Returns a 2-letter lowercase ISO 3166-1 alpha-2 code, or `None` for default (US).
//! Used to route residential proxy requests through the appropriate country,
//! giving location-aware results from Google/Bing/DDG.

/// Detect the best proxy country for a query.
///
/// Returns `Some("fr")`, `Some("jp")` etc., or `None` for US/default routing.
pub fn detect_query_locale(query: &str) -> Option<&'static str> {
    // Script detection is O(n chars) — fast and highest confidence
    if let Some(cc) = detect_by_script(query) {
        return Some(cc);
    }
    let q = query.to_lowercase();
    // Country/city mentions — explicit signals
    if let Some(cc) = detect_country_mention(&q) {
        return Some(cc);
    }
    // Language word patterns — useful for non-English queries without explicit location
    detect_language_words(&q)
}

/// Detect locale from Unicode script ranges — O(n) char scan.
fn detect_by_script(query: &str) -> Option<&'static str> {
    let mut has_cjk = false;
    let mut has_hiragana = false;
    let mut has_katakana = false;

    for c in query.chars() {
        match c {
            // Japanese kana (unambiguous)
            '\u{3040}'..='\u{309F}' => has_hiragana = true,
            '\u{30A0}'..='\u{30FF}' => has_katakana = true,
            // Hangul — Korean
            '\u{AC00}'..='\u{D7AF}' | '\u{1100}'..='\u{11FF}' => return Some("kr"),
            // Arabic
            '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' => return Some("ae"),
            // Cyrillic — Russian
            '\u{0400}'..='\u{04FF}' => return Some("ru"),
            // Devanagari — Hindi/Indian
            '\u{0900}'..='\u{097F}' => return Some("in"),
            // Thai
            '\u{0E00}'..='\u{0E7F}' => return Some("th"),
            // Greek
            '\u{0370}'..='\u{03FF}' => return Some("gr"),
            // CJK Unified — could be Chinese or Japanese kanji
            '\u{4E00}'..='\u{9FFF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{20000}'..='\u{2A6DF}' => has_cjk = true,
            _ => {}
        }
    }

    if has_hiragana || has_katakana {
        return Some("jp"); // kana present → Japanese
    }
    if has_cjk {
        return Some("cn"); // CJK only (no kana) → Chinese
    }
    None
}

/// Detect country from explicit mentions in the (lowercased) query.
fn detect_country_mention(q: &str) -> Option<&'static str> {
    // ── United States ────────────────────────────────────────────────────────
    if q.contains("near me")
        || q.contains("new york")
        || q.contains("los angeles")
        || q.contains("chicago")
        || q.contains("san francisco")
        || q.contains("houston")
        || q.contains("las vegas")
        || q.contains("miami")
        || q.contains(" usa")
        || q.contains("united states")
        || q.contains("american ")
    {
        return Some("us");
    }

    // ── United Kingdom ───────────────────────────────────────────────────────
    if q.contains("london")
        || q.contains("manchester")
        || q.contains("edinburgh")
        || q.contains("birmingham")
        || q.contains("united kingdom")
        || q.contains(" uk ")
        || q.ends_with(" uk")
        || q.starts_with("uk ")
        || q.contains("england")
        || q.contains("britain")
        || q.contains("british ")
        || q.contains("scotland")
        || q.contains("wales")
    {
        return Some("gb");
    }

    // ── France ───────────────────────────────────────────────────────────────
    if q.contains("france")
        || q.contains("paris")
        || q.contains("lyon")
        || q.contains("marseille")
        || q.contains("bordeaux")
        || q.contains("toulouse")
        || q.contains("french ")
        || q.contains(" france")
    {
        return Some("fr");
    }

    // ── Germany ──────────────────────────────────────────────────────────────
    if q.contains("germany")
        || q.contains("berlin")
        || q.contains("munich")
        || q.contains("münchen")
        || q.contains("hamburg")
        || q.contains("frankfurt")
        || q.contains("cologne")
        || q.contains("düsseldorf")
        || q.contains("deutsch")
        || q.contains("german ")
    {
        return Some("de");
    }

    // ── Japan ────────────────────────────────────────────────────────────────
    if q.contains("japan")
        || q.contains("tokyo")
        || q.contains("osaka")
        || q.contains("kyoto")
        || q.contains("hiroshima")
        || q.contains("japanese ")
    {
        return Some("jp");
    }

    // ── Spain ────────────────────────────────────────────────────────────────
    if q.contains("spain")
        || q.contains("madrid")
        || q.contains("barcelona")
        || q.contains("seville")
        || q.contains("valencia")
        || q.contains("spanish ")
        || q.contains("español")
    {
        return Some("es");
    }

    // ── Italy ────────────────────────────────────────────────────────────────
    if q.contains("italy")
        || q.contains(" rome")
        || q.contains(" milan")
        || q.contains("venice")
        || q.contains("florence")
        || q.contains("naples")
        || q.contains("italian ")
        || q.contains("italiano")
    {
        return Some("it");
    }

    // ── Brazil ───────────────────────────────────────────────────────────────
    if q.contains("brazil")
        || q.contains("brasil")
        || q.contains("são paulo")
        || q.contains("sao paulo")
        || q.contains("rio de janeiro")
        || q.contains("brasília")
    {
        return Some("br");
    }

    // ── China ────────────────────────────────────────────────────────────────
    if q.contains("china")
        || q.contains("beijing")
        || q.contains("shanghai")
        || q.contains("shenzhen")
        || q.contains("guangzhou")
        || q.contains("chinese ")
    {
        return Some("cn");
    }

    // ── South Korea ──────────────────────────────────────────────────────────
    if q.contains("korea")
        || q.contains("seoul")
        || q.contains("busan")
        || q.contains("korean ")
    {
        return Some("kr");
    }

    // ── India ────────────────────────────────────────────────────────────────
    if q.contains("india")
        || q.contains("mumbai")
        || q.contains("delhi")
        || q.contains("bangalore")
        || q.contains("bengaluru")
        || q.contains("chennai")
        || q.contains("hyderabad")
        || q.contains("indian ")
    {
        return Some("in");
    }

    // ── Russia ───────────────────────────────────────────────────────────────
    if q.contains("russia")
        || q.contains("moscow")
        || q.contains("st. petersburg")
        || q.contains("saint petersburg")
        || q.contains("russian ")
    {
        return Some("ru");
    }

    // ── Australia ────────────────────────────────────────────────────────────
    if q.contains("australia")
        || q.contains("sydney")
        || q.contains("melbourne")
        || q.contains("brisbane")
        || q.contains("perth")
        || q.contains("australian ")
    {
        return Some("au");
    }

    // ── Canada ───────────────────────────────────────────────────────────────
    if q.contains("canada")
        || q.contains("toronto")
        || q.contains("vancouver")
        || q.contains("montreal")
        || q.contains("calgary")
        || q.contains("canadian ")
    {
        return Some("ca");
    }

    // ── Mexico ───────────────────────────────────────────────────────────────
    if q.contains("mexico")
        || q.contains("ciudad de mexico")
        || q.contains("guadalajara")
        || q.contains("monterrey")
    {
        return Some("mx");
    }

    // ── Netherlands ──────────────────────────────────────────────────────────
    if q.contains("netherlands")
        || q.contains("amsterdam")
        || q.contains("rotterdam")
        || q.contains("dutch ")
        || q.contains("holland")
    {
        return Some("nl");
    }

    // ── Sweden ───────────────────────────────────────────────────────────────
    if q.contains("sweden")
        || q.contains("stockholm")
        || q.contains("gothenburg")
        || q.contains("swedish ")
    {
        return Some("se");
    }

    // ── Poland ───────────────────────────────────────────────────────────────
    if q.contains("poland") || q.contains("warsaw") || q.contains("krakow") {
        return Some("pl");
    }

    // ── Turkey ───────────────────────────────────────────────────────────────
    if q.contains("turkey") || q.contains("istanbul") || q.contains("ankara") {
        return Some("tr");
    }

    // ── Singapore ────────────────────────────────────────────────────────────
    if q.contains("singapore") {
        return Some("sg");
    }

    None
}

/// Language detection from common function words (Latin-script queries only).
/// Requires ≥2 matching signal words for confidence.
fn detect_language_words(q: &str) -> Option<&'static str> {
    let words: Vec<&str> = q.split_whitespace().collect();

    // Each tuple: (language, signal words, minimum matches needed)
    let signals: &[(&str, &[&str], usize)] = &[
        (
            "fr",
            &[
                "le", "la", "les", "du", "des", "est", "sont", "pour", "avec", "dans", "sur",
                "comment", "pourquoi", "quoi", "qui", "que", "quel", "quelle", "une", "un",
                "pas", "plus", "très", "aussi", "apprendre", "rapidement",
            ],
            2,
        ),
        (
            "de",
            &[
                "der", "die", "das", "und", "ist", "sind", "für", "mit", "bei", "wie", "was",
                "wo", "wann", "warum", "welche", "welcher", "ein", "eine", "nicht", "kann",
                "werden", "einfach", "erklärt", "erklaert",
            ],
            2,
        ),
        (
            "es",
            &[
                "el", "los", "las", "una", "para", "con", "por", "como", "qué", "dónde",
                "cuándo", "mejor", "cómo", "también", "más", "muy", "hacer", "cocinar",
                "valenciana", "paella",
            ],
            2,
        ),
        (
            "pt",
            &[
                "não", "são", "uma", "para", "com", "por", "como", "onde", "quando", "qual",
                "mais", "muito", "também", "fazer",
            ],
            2,
        ),
        (
            "it",
            &[
                "il", "della", "dello", "per", "con", "come", "dove", "quando", "perché",
                "cosa", "anche", "molto", "fare", "essere",
            ],
            2,
        ),
        (
            "nl",
            &[
                "de", "het", "een", "van", "in", "op", "met", "voor", "aan", "hoe", "wat",
                "waar", "waarom", "beste",
            ],
            3, // stricter: many short words overlap with English
        ),
    ];

    let mut best_lang = None;
    let mut best_count = 0usize;

    for &(lang, signal_words, min_matches) in signals {
        let count = words
            .iter()
            .filter(|&&w| signal_words.contains(&w))
            .count();
        if count >= min_matches && count > best_count {
            best_count = count;
            best_lang = Some(lang);
        }
    }

    best_lang
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_japanese_kana() {
        // の is hiragana → jp even without katakana
        assert_eq!(detect_query_locale("東京の天気"), Some("jp"));
        assert_eq!(detect_query_locale("東京のラーメン屋"), Some("jp")); // katakana ラーメン
        assert_eq!(detect_query_locale("how to make ramen in tokyo"), Some("jp"));
        // Pure CJK (no kana) → cn
        assert_eq!(detect_query_locale("量子计算机"), Some("cn"));
    }

    #[test]
    fn detects_french_query() {
        assert_eq!(
            detect_query_locale("comment apprendre le francais rapidement"),
            Some("fr")
        );
    }

    #[test]
    fn detects_spanish_recipe() {
        assert_eq!(
            detect_query_locale("como cocinar paella valenciana"),
            Some("es")
        );
    }

    #[test]
    fn detects_german_explainer() {
        assert_eq!(
            detect_query_locale("was ist quantencomputing einfach erklaert"),
            Some("de")
        );
    }

    #[test]
    fn detects_uk_from_city() {
        assert_eq!(detect_query_locale("best fish and chips in london"), Some("gb"));
    }

    #[test]
    fn detects_near_me_as_us() {
        assert_eq!(detect_query_locale("pizza near me"), Some("us"));
    }

    #[test]
    fn returns_none_for_generic_english() {
        // Generic English → None (default US routing is fine)
        assert_eq!(detect_query_locale("how does photosynthesis work"), None);
        assert_eq!(detect_query_locale("rust programming language"), None);
    }

    #[test]
    fn detects_cyrillic_as_russian() {
        assert_eq!(detect_query_locale("что такое квантовые вычисления"), Some("ru"));
    }

    #[test]
    fn detects_arabic_script() {
        assert_eq!(detect_query_locale("كيف تتعلم البرمجة"), Some("ae"));
    }
}
