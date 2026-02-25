//! `hsx research` — multi-source research with citations (Mode B).

use crate::cli::ResearchArgs;
use hsx_core::citation::types::CitationStyle as CoreCitationStyle;
use hsx_core::config::HsxConfig;
use hsx_core::research::pipeline::ResearchPipeline;
use hsx_core::research::ResearchConfig;
use hsx_core::validate::types::ValidationMode;

pub async fn run(args: ResearchArgs, config_obj: &HsxConfig) -> anyhow::Result<()> {
    let citation_style = match args.citations {
        crate::cli::CitationStyle::Apa => CoreCitationStyle::Apa,
        crate::cli::CitationStyle::Mla => CoreCitationStyle::Mla,
        crate::cli::CitationStyle::Chicago => CoreCitationStyle::Chicago,
        crate::cli::CitationStyle::Harvard => CoreCitationStyle::Apa, // map Harvard -> APA
        crate::cli::CitationStyle::Ieee => CoreCitationStyle::Ieee,
        crate::cli::CitationStyle::Bibtex => CoreCitationStyle::Bibtex,
    };

    let validation_mode = ValidationMode::from_str_loose(&args.validate);

    let config = ResearchConfig {
        query: args.query.clone(),
        max_sources: args.max_sources,
        token_budget: None,
        citation_style,
        validation_mode,
        strict_evidence: args.strict_evidence,
        evidence_graph: args.evidence_graph,
        trace_sources: args.trace_sources,
        trust_verify: args.trust_verify,
        max_rar_loops: 3,
    };

    println!("Researching: {} ...", args.query);

    let http_client = hsx_core::http::client::HttpClient::new(config_obj)?;
    let report = ResearchPipeline::execute(&config, config_obj, &http_client).await?;

    // Format output
    let output = format_report(&report, &args);

    if let Some(ref path) = args.output {
        std::fs::write(path, &output)?;
        eprintln!("Report written to {path}");
    } else {
        println!("{output}");
    }

    // Write evidence graph if requested
    if args.evidence_graph {
        if let Some(ref graph) = report.evidence_graph {
            let json = serde_json::to_string_pretty(graph)?;
            let graph_path = args
                .output
                .as_deref()
                .map(|p| p.replace(".md", "_evidence.json"))
                .unwrap_or_else(|| "evidence_graph.json".into());
            std::fs::write(&graph_path, &json)?;
            eprintln!("Evidence graph written to {graph_path}");
        }
    }

    Ok(())
}

fn format_report(report: &hsx_core::research::ResearchReport, args: &ResearchArgs) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Research: {}\n\n", report.query));

    if report.sub_queries.len() > 1 {
        out.push_str("## Sub-queries\n\n");
        for sq in &report.sub_queries {
            out.push_str(&format!("- {sq}\n"));
        }
        out.push('\n');
    }

    out.push_str("## Findings\n\n");
    if report.synthesis.is_empty() {
        out.push_str(
            "*Research pipeline pending full implementation — Phase 3 scaffold complete.*\n",
        );
    } else {
        out.push_str(&report.synthesis);
    }

    if !report.reference_section.is_empty() {
        out.push_str("\n\n## Sources\n\n");
        out.push_str(&report.reference_section);
    }

    out.push_str(&format!(
        "\n\n---\n*Confidence: {:.0}% | Sources: {} | Validated: {} | Duration: {}ms | Validate: {}*\n",
        report.meta.overall_confidence * 100.0,
        report.meta.sources_fetched,
        report.meta.sources_validated,
        report.meta.duration_ms,
        args.validate,
    ));

    out
}
