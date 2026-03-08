use fetchium_core::query::locale::detect_query_locale;

#[test]
fn detects_japanese_kana() {
    assert_eq!(detect_query_locale("東京の天気"), Some("jp"));
    assert_eq!(detect_query_locale("東京のラーメン屋"), Some("jp"));
    assert_eq!(
        detect_query_locale("how to make ramen in tokyo"),
        Some("jp")
    );
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
    assert_eq!(
        detect_query_locale("best fish and chips in london"),
        Some("gb")
    );
}

#[test]
fn detects_near_me_as_us() {
    assert_eq!(detect_query_locale("pizza near me"), Some("us"));
}

#[test]
fn returns_none_for_generic_english() {
    assert_eq!(detect_query_locale("how does photosynthesis work"), None);
    assert_eq!(detect_query_locale("rust programming language"), None);
}

#[test]
fn detects_cyrillic_as_russian() {
    assert_eq!(
        detect_query_locale("что такое квантовые вычисления"),
        Some("ru")
    );
}

#[test]
fn detects_arabic_script() {
    assert_eq!(detect_query_locale("كيف تتعلم البرمجة"), Some("ae"));
}

#[test]
fn detects_explicit_site_and_language_hints() {
    assert_eq!(
        detect_query_locale("site:bbc.co.uk prime minister"),
        Some("gb")
    );
    assert_eq!(
        detect_query_locale("lang:fr meilleures universites"),
        Some("fr")
    );
}

#[test]
fn avoids_turkey_food_false_positive() {
    assert_eq!(
        detect_query_locale("best turkey recipe with stuffing"),
        None
    );
}

#[test]
fn avoids_substring_false_positive() {
    assert_eq!(detect_query_locale("chrome extension review"), None);
}
