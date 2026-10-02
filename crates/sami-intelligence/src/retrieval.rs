use crate::util::tokens;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn expand(words: &[String]) -> BTreeSet<String> {
    // A published, domain-bounded alias lexicon. No hidden neural similarity.
    let aliases: &[&[&str]] = &[
        &["warranty", "entitlement", "coverage"],
        &["overheating", "overheated", "temperature", "hot"],
        &["leak", "leaking", "leakage"],
        &["fault", "failure", "broken"],
        &["hours", "hrs", "hour"],
        &["serial", "identifier"],
        &["manual", "procedure", "instructions"],
        &["price", "cost", "expensive", "costly"],
    ];
    let mut result: BTreeSet<_> = words.iter().cloned().collect();
    for group in aliases {
        if group.iter().any(|term| result.contains(*term)) {
            result.extend(group.iter().map(|v| v.to_string()));
        }
    }
    result
}

/// BM25 and sparse alias-cosine fusion. Input documents must already be ACL and
/// purpose filtered by the caller. Scores rank documents; none is a probability.
pub fn search(query: &str, documents: &[Value], limit: usize) -> Value {
    if query.len() > 65_536 || documents.len() > 10_000 || limit > 1_000 {
        return json!({"status":"budget_exhausted", "results":[], "score_kind":"ranking_only"});
    }
    let query_words = tokens(query);
    let expanded_query = expand(&query_words);
    if expanded_query.is_empty() || documents.is_empty() {
        return json!({"status":"resolved", "results":[], "score_kind":"ranking_only", "count":0});
    }
    let mut docs = Vec::new();
    let mut df = BTreeMap::<String, usize>::new();
    let mut total_length = 0usize;
    let mut total_bytes = 0usize;
    for (index, document) in documents.iter().enumerate() {
        let text = document
            .get("content")
            .or_else(|| document.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if text.len() > 1_048_576 {
            return json!({"status":"budget_exhausted", "results":[], "reason":"document_byte_limit", "document_index":index});
        }
        let title = document.get("title").and_then(Value::as_str).unwrap_or("");
        total_bytes = total_bytes
            .saturating_add(text.len())
            .saturating_add(title.len());
        if total_bytes > 16_777_216 {
            return json!({"status":"budget_exhausted","results":[],"reason":"aggregate_corpus_byte_limit"});
        }
        let words = tokens(&format!("{title} {text}"));
        let mut frequencies = BTreeMap::<String, usize>::new();
        for word in &words {
            *frequencies.entry(word.clone()).or_default() += 1;
        }
        for word in frequencies.keys() {
            *df.entry(word.clone()).or_default() += 1;
        }
        let mut concepts = expand(&words);
        for alias in document
            .get("semantic_aliases")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            concepts.extend(tokens(alias));
        }
        total_length += words.len();
        docs.push((document, frequencies, words.len(), concepts));
    }
    let n = docs.len() as f64;
    let average_length = (total_length as f64 / n).max(1.0);
    let query_set: BTreeSet<_> = query_words.into_iter().collect();
    let mut results = Vec::<(f64, usize, Value)>::new();
    for (index, (doc, tf, length, concepts)) in docs.into_iter().enumerate() {
        let mut bm25 = 0.0;
        let mut terms = Vec::new();
        for word in &query_set {
            if let Some(count) = tf.get(word) {
                let frequency = df.get(word).copied().unwrap_or(0) as f64;
                let idf = (1.0 + (n - frequency + 0.5) / (frequency + 0.5)).ln();
                let count = *count as f64;
                bm25 += idf * count * 2.2
                    / (count + 1.2 * (0.25 + 0.75 * length as f64 / average_length));
                terms.push(word.clone());
            }
        }
        let overlap = expanded_query.intersection(&concepts).count();
        let cosine =
            overlap as f64 / ((expanded_query.len() * concepts.len().max(1)) as f64).sqrt();
        let normalized_bm25 = bm25 / (bm25 + 3.0);
        let score = 0.7 * normalized_bm25 + 0.3 * cosine;
        if score <= 0.0 {
            continue;
        }
        let id = doc.get("id").cloned().unwrap_or(json!(index));
        results.push((score, index, json!({"id":id, "document":doc, "score":score, "components":{"bm25":bm25, "normalized_bm25":normalized_bm25, "sparse_alias_cosine":cosine}, "matched_terms":terms, "source_id":doc.get("source_id"), "source_revision":doc.get("source_revision")})));
    }
    results.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let count = results.len();
    json!({"status":"resolved", "results":results.into_iter().take(limit).map(|(_,_,v)|v).collect::<Vec<_>>(), "count":count, "score_kind":"ranking_only", "fusion":{"bm25":0.7,"sparse_alias_cosine":0.3,"calibrated":false}, "authorization":"caller_prefilter_required"})
}
