//! Chart/table data extraction from HTML (PRD §34).

use crate::error::FetchiumResult;
use serde::{Deserialize, Serialize};

/// Extracted tabular / chart data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Extract all `<table>` elements from an HTML string.
pub fn extract_tables(html: &str) -> FetchiumResult<Vec<TableData>> {
    let mut results = Vec::new();

    let fragment = scraper::Html::parse_fragment(html);
    let table_sel = scraper::Selector::parse("table").unwrap();
    let tr_sel = scraper::Selector::parse("tr").unwrap();
    let th_sel = scraper::Selector::parse("th").unwrap();
    let td_sel = scraper::Selector::parse("td").unwrap();

    for table in fragment.select(&table_sel) {
        let mut headers = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();

        for (row_idx, row) in table.select(&tr_sel).enumerate() {
            if row_idx == 0 {
                headers = row
                    .select(&th_sel)
                    .map(|th| th.text().collect::<String>().trim().to_string())
                    .collect();
                if headers.is_empty() {
                    // First row is data, not a header row
                    let cells: Vec<String> = row
                        .select(&td_sel)
                        .map(|td| td.text().collect::<String>().trim().to_string())
                        .collect();
                    if !cells.is_empty() {
                        rows.push(cells);
                    }
                    continue;
                }
            }
            let cells: Vec<String> = row
                .select(&td_sel)
                .map(|td| td.text().collect::<String>().trim().to_string())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }

        if !rows.is_empty() {
            results.push(TableData { headers, rows });
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_html_table() {
        let html =
            "<table><tr><th>Name</th><th>Value</th></tr><tr><td>Rust</td><td>95</td></tr></table>";
        let tables = extract_tables(html).unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0].headers, vec!["Name", "Value"]);
        assert_eq!(tables[0].rows[0], vec!["Rust", "95"]);
    }
}
