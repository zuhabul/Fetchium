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
    if let Some(cc) = detect_explicit_locale_hint(&q) {
        return Some(cc);
    }
    // Country/city mentions — explicit signals
    if let Some(cc) = detect_country_mention(&q) {
        return Some(cc);
    }
    // Language word patterns — useful for non-English queries without explicit location
    detect_language_words(&q)
}

/// Detect the query language when it appears to be non-English.
///
/// Unlike [`detect_query_locale`], this avoids country/city heuristics so callers
/// can distinguish foreign-language phrasing from English queries about a place.
pub fn detect_query_language(query: &str) -> Option<&'static str> {
    if let Some(cc) = detect_by_script(query) {
        return Some(cc);
    }

    let q = query.to_lowercase();
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
            // Bengali
            '\u{0980}'..='\u{09FF}' => return Some("bd"),
            // Greek
            '\u{0370}'..='\u{03FF}' => return Some("gr"),
            // CJK Unified — could be Chinese or Japanese kanji
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{20000}'..='\u{2A6DF}' => {
                has_cjk = true
            }
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

/// Detect locale from explicit search syntax such as `site:bbc.co.uk` or `lang:fr`.
fn detect_explicit_locale_hint(q: &str) -> Option<&'static str> {
    let normalized = normalize_for_matching(q);

    for (hint, cc) in [
        (" site bbc co uk ", "gb"),
        (" site gov uk ", "gb"),
        (" site co uk ", "gb"),
        (" site ac uk ", "gb"),
        (" site gouv fr ", "fr"),
        (" site lemonde fr ", "fr"),
        (" site de ", "de"),
        (" site fr ", "fr"),
        (" site es ", "es"),
        (" site it ", "it"),
        (" site jp ", "jp"),
        (" site co jp ", "jp"),
        (" site kr ", "kr"),
        (" site co kr ", "kr"),
        (" site cn ", "cn"),
        (" site com au ", "au"),
        (" site ca ", "ca"),
        (" site com br ", "br"),
        (" site com mx ", "mx"),
        (" lang fr ", "fr"),
        (" language french ", "fr"),
        (" lang de ", "de"),
        (" language german ", "de"),
        (" lang es ", "es"),
        (" language spanish ", "es"),
        (" lang it ", "it"),
        (" language italian ", "it"),
        (" lang ja ", "jp"),
        (" language japanese ", "jp"),
        (" lang ko ", "kr"),
        (" language korean ", "kr"),
        (" lang zh ", "cn"),
        (" language chinese ", "cn"),
        (" lang pt ", "pt"),
        (" language portuguese ", "pt"),
    ] {
        if normalized.contains(hint) {
            return Some(cc);
        }
    }

    None
}

/// Detect country from explicit mentions in the (lowercased) query.
fn detect_country_mention(q: &str) -> Option<&'static str> {
    let normalized = normalize_for_matching(q);

    // ── United States ────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "near me")
        || contains_phrase(&normalized, "new york")
        || contains_phrase(&normalized, "los angeles")
        || contains_phrase(&normalized, "chicago")
        || contains_phrase(&normalized, "san francisco")
        || contains_phrase(&normalized, "houston")
        || contains_phrase(&normalized, "las vegas")
        || contains_phrase(&normalized, "miami")
        || contains_phrase(&normalized, "usa")
        || contains_phrase(&normalized, "united states")
        || contains_phrase(&normalized, "american")
    {
        return Some("us");
    }

    // ── United Kingdom ───────────────────────────────────────────────────────
    if contains_phrase(&normalized, "london")
        || contains_phrase(&normalized, "manchester")
        || contains_phrase(&normalized, "edinburgh")
        || contains_phrase(&normalized, "birmingham")
        || contains_phrase(&normalized, "united kingdom")
        || contains_phrase(&normalized, "u k")
        || contains_phrase(&normalized, "england")
        || contains_phrase(&normalized, "britain")
        || contains_phrase(&normalized, "british")
        || contains_phrase(&normalized, "scotland")
        || contains_phrase(&normalized, "wales")
    {
        return Some("gb");
    }

    // ── France ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "france")
        || contains_phrase(&normalized, "paris")
        || contains_phrase(&normalized, "lyon")
        || contains_phrase(&normalized, "marseille")
        || contains_phrase(&normalized, "bordeaux")
        || contains_phrase(&normalized, "toulouse")
        || contains_phrase(&normalized, "french")
    {
        return Some("fr");
    }

    // ── Germany ──────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "germany")
        || contains_phrase(&normalized, "berlin")
        || contains_phrase(&normalized, "munich")
        || contains_phrase(&normalized, "münchen")
        || contains_phrase(&normalized, "hamburg")
        || contains_phrase(&normalized, "frankfurt")
        || contains_phrase(&normalized, "cologne")
        || contains_phrase(&normalized, "düsseldorf")
        || contains_phrase(&normalized, "deutsch")
        || contains_phrase(&normalized, "german")
    {
        return Some("de");
    }

    // ── Japan ────────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "japan")
        || contains_phrase(&normalized, "tokyo")
        || contains_phrase(&normalized, "osaka")
        || contains_phrase(&normalized, "kyoto")
        || contains_phrase(&normalized, "hiroshima")
        || contains_phrase(&normalized, "japanese")
    {
        return Some("jp");
    }

    // ── Spain ────────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "spain")
        || contains_phrase(&normalized, "madrid")
        || contains_phrase(&normalized, "barcelona")
        || contains_phrase(&normalized, "seville")
        || contains_phrase(&normalized, "valencia")
        || contains_phrase(&normalized, "spanish")
        || contains_phrase(&normalized, "español")
    {
        return Some("es");
    }

    // ── Italy ────────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "italy")
        || contains_phrase(&normalized, "rome")
        || contains_phrase(&normalized, "milan")
        || contains_phrase(&normalized, "venice")
        || contains_phrase(&normalized, "florence")
        || contains_phrase(&normalized, "naples")
        || contains_phrase(&normalized, "italian")
        || q.contains("italiano")
    {
        return Some("it");
    }

    // ── Brazil ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "brazil")
        || contains_phrase(&normalized, "brasil")
        || contains_phrase(&normalized, "são paulo")
        || contains_phrase(&normalized, "sao paulo")
        || contains_phrase(&normalized, "rio de janeiro")
        || contains_phrase(&normalized, "brasília")
    {
        return Some("br");
    }

    // ── China ────────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "china")
        || contains_phrase(&normalized, "beijing")
        || contains_phrase(&normalized, "shanghai")
        || contains_phrase(&normalized, "shenzhen")
        || contains_phrase(&normalized, "guangzhou")
        || contains_phrase(&normalized, "chinese")
    {
        return Some("cn");
    }

    // ── South Korea ──────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "south korea")
        || contains_phrase(&normalized, "korea")
        || contains_phrase(&normalized, "seoul")
        || contains_phrase(&normalized, "busan")
        || contains_phrase(&normalized, "korean")
    {
        return Some("kr");
    }

    // ── India ────────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "india")
        || contains_phrase(&normalized, "mumbai")
        || contains_phrase(&normalized, "delhi")
        || contains_phrase(&normalized, "bangalore")
        || contains_phrase(&normalized, "bengaluru")
        || contains_phrase(&normalized, "chennai")
        || contains_phrase(&normalized, "hyderabad")
        || contains_phrase(&normalized, "indian")
    {
        return Some("in");
    }

    // ── Russia ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "russia")
        || contains_phrase(&normalized, "moscow")
        || contains_phrase(&normalized, "st petersburg")
        || contains_phrase(&normalized, "saint petersburg")
        || contains_phrase(&normalized, "russian")
    {
        return Some("ru");
    }

    // ── Australia ────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "australia")
        || contains_phrase(&normalized, "sydney")
        || contains_phrase(&normalized, "melbourne")
        || contains_phrase(&normalized, "brisbane")
        || contains_phrase(&normalized, "perth")
        || contains_phrase(&normalized, "australian")
    {
        return Some("au");
    }

    // ── Canada ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "canada")
        || contains_phrase(&normalized, "toronto")
        || contains_phrase(&normalized, "vancouver")
        || contains_phrase(&normalized, "montreal")
        || contains_phrase(&normalized, "calgary")
        || contains_phrase(&normalized, "canadian")
    {
        return Some("ca");
    }

    // ── Mexico ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "mexico")
        || contains_phrase(&normalized, "ciudad de mexico")
        || contains_phrase(&normalized, "guadalajara")
        || contains_phrase(&normalized, "monterrey")
    {
        return Some("mx");
    }

    // ── Netherlands ──────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "netherlands")
        || contains_phrase(&normalized, "amsterdam")
        || contains_phrase(&normalized, "rotterdam")
        || contains_phrase(&normalized, "dutch")
        || contains_phrase(&normalized, "holland")
    {
        return Some("nl");
    }

    // ── Sweden ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "sweden")
        || contains_phrase(&normalized, "stockholm")
        || contains_phrase(&normalized, "gothenburg")
        || contains_phrase(&normalized, "swedish")
    {
        return Some("se");
    }

    // ── Poland ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "poland")
        || contains_phrase(&normalized, "warsaw")
        || contains_phrase(&normalized, "krakow")
    {
        return Some("pl");
    }

    // ── Turkey ───────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "istanbul")
        || contains_phrase(&normalized, "ankara")
        || (contains_phrase(&normalized, "turkey") && !looks_like_food_query(&normalized))
    {
        return Some("tr");
    }

    // ── Singapore ────────────────────────────────────────────────────────────
    if contains_phrase(&normalized, "singapore") {
        return Some("sg");
    }

    None
}

fn normalize_for_matching(query: &str) -> String {
    let mut normalized = String::with_capacity(query.len() + 2);
    normalized.push(' ');
    let mut previous_was_space = true;

    for ch in query.chars() {
        if ch.is_alphanumeric() {
            normalized.push(ch);
            previous_was_space = false;
        } else if !previous_was_space {
            normalized.push(' ');
            previous_was_space = true;
        }
    }

    if !previous_was_space {
        normalized.push(' ');
    }

    normalized
}

fn contains_phrase(normalized_query: &str, phrase: &str) -> bool {
    let normalized_phrase = normalize_for_matching(phrase);
    normalized_query.contains(&normalized_phrase)
}

fn looks_like_food_query(normalized_query: &str) -> bool {
    [
        " recipe ",
        " recipes ",
        " roast ",
        " roasted ",
        " gravy ",
        " stuffing ",
        " sandwich ",
        " burger ",
        " breast ",
        " thighs ",
        " leftovers ",
        " oven ",
        " cook ",
        " cooking ",
    ]
    .iter()
    .any(|signal| normalized_query.contains(signal))
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
                "le",
                "la",
                "les",
                "du",
                "des",
                "est",
                "sont",
                "pour",
                "avec",
                "dans",
                "sur",
                "comment",
                "pourquoi",
                "quoi",
                "qui",
                "que",
                "quel",
                "quelle",
                "une",
                "un",
                "pas",
                "plus",
                "très",
                "aussi",
                "apprendre",
                "rapidement",
            ],
            2,
        ),
        (
            "de",
            &[
                "der", "die", "das", "und", "ist", "sind", "für", "mit", "bei", "wie", "was", "wo",
                "wann", "warum", "welche", "welcher", "ein", "eine", "nicht", "kann", "werden",
                "einfach", "erklärt", "erklaert",
            ],
            2,
        ),
        (
            "es",
            &[
                "el",
                "los",
                "las",
                "una",
                "para",
                "con",
                "por",
                "como",
                "qué",
                "dónde",
                "cuándo",
                "mejor",
                "cómo",
                "también",
                "más",
                "muy",
                "hacer",
                "cocinar",
                "valenciana",
                "paella",
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
                "il", "della", "dello", "per", "con", "come", "dove", "quando", "perché", "cosa",
                "anche", "molto", "fare", "essere",
            ],
            2,
        ),
        (
            "nl",
            &[
                "de", "het", "een", "van", "in", "op", "met", "voor", "aan", "hoe", "wat", "waar",
                "waarom", "beste",
            ],
            3, // stricter: many short words overlap with English
        ),
    ];

    let mut best_lang = None;
    let mut best_count = 0usize;

    for &(lang, signal_words, min_matches) in signals {
        let count = words.iter().filter(|&&w| signal_words.contains(&w)).count();
        if count >= min_matches && count > best_count {
            best_count = count;
            best_lang = Some(lang);
        }
    }

    best_lang
}
