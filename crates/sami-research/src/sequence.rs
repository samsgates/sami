use crate::{array, bounded_usize, digest, invalid, text, ResearchError, Result};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

const MAX_TOKENS: usize = 8192;
pub fn build(input: &Value) -> Result<Value> {
    let docs = array(input, "documents", 128)?;
    if docs.is_empty() {
        return Err(invalid("documents cannot be empty"));
    }
    let mut documents = Vec::new();
    let mut suffixes = Vec::new();
    let mut ids = BTreeSet::new();
    let mut total = 0;
    for (document, d) in docs.iter().enumerate() {
        let id = text(d, "id")?;
        let revision = text(d, "revision")?;
        let license = text(d, "license")?;
        let content = text(d, "content")?;
        if id.is_empty() || revision.is_empty() || license.is_empty() || !ids.insert(id.to_owned())
        {
            return Err(invalid(
                "unique document id, revision and declared license required",
            ));
        }
        let tokens = content
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if tokens.iter().any(|t| t.len() > 256) {
            return Err(ResearchError::Limit(
                "sequence token exceeds 256 bytes".into(),
            ));
        }
        total += tokens.len();
        if total > MAX_TOKENS {
            return Err(ResearchError::Limit(format!(
                "bounded reference index supports at most {MAX_TOKENS} tokens"
            )));
        }
        for offset in 0..tokens.len() {
            suffixes.push((document, offset));
        }
        documents.push(json!({"id":id,"revision":revision,"license":license,"source_sha256":digest(&json!(content))?,"tokens":tokens}));
    }
    let token_lists = documents
        .iter()
        .map(|d| {
            d["tokens"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|x| x.as_str().unwrap_or_default().to_owned())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    suffixes.sort_by(|a, b| {
        token_lists[a.0][a.1..]
            .cmp(&token_lists[b.0][b.1..])
            .then(a.cmp(b))
    });
    let mut index = json!({"format_version":1,"tokenizer":"case_sensitive_whitespace_v1","document_boundary":"suffixes never include tokens from another document","token_count":total,"documents":documents,"suffix_array":suffixes,"implementation":"bounded_in_memory_reference_not_memory_mapped_production_shard"});
    let hash = digest(&index)?;
    index["checksum_sha256"] = json!(hash);
    Ok(json!({"index":index,"status":"research_artifact","production_promoted":false}))
}
type ParsedIndex = (Vec<Vec<String>>, Vec<(usize, usize)>);
fn validate(index: &Value) -> Result<ParsedIndex> {
    let checksum = text(index, "checksum_sha256")?;
    let mut unsigned = index.clone();
    unsigned
        .as_object_mut()
        .ok_or_else(|| invalid("index must be object"))?
        .remove("checksum_sha256");
    if digest(&unsigned)? != checksum {
        return Err(invalid("sequence checksum mismatch"));
    }
    if index.get("format_version").and_then(Value::as_u64) != Some(1)
        || index.get("tokenizer").and_then(Value::as_str) != Some("case_sensitive_whitespace_v1")
    {
        return Err(invalid("incompatible sequence index"));
    }
    let docs = array(index, "documents", 128)?;
    let mut tokens = Vec::new();
    let mut total = 0;
    for d in docs {
        let t = array(d, "tokens", MAX_TOKENS)?
            .iter()
            .map(|v| {
                v.as_str()
                    .filter(|s| s.len() <= 256)
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("invalid sequence tokens"))
            })
            .collect::<Result<Vec<_>>>()?;
        total += t.len();
        if total > MAX_TOKENS {
            return Err(ResearchError::Limit(
                "index corpus exceeds token budget".into(),
            ));
        }
        tokens.push(t);
    }
    if index.get("token_count").and_then(Value::as_u64) != Some(total as u64) {
        return Err(invalid("token count mismatch"));
    }
    let suffixes = array(index, "suffix_array", MAX_TOKENS)?;
    if suffixes.len() != total {
        return Err(invalid("suffix array length mismatch"));
    }
    let mut parsed = Vec::new();
    let mut unique = BTreeSet::new();
    for s in suffixes {
        let pair = s
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| invalid("suffix pointer must have two indices"))?;
        let d = pair[0]
            .as_u64()
            .ok_or_else(|| invalid("invalid suffix document index"))? as usize;
        let o = pair[1]
            .as_u64()
            .ok_or_else(|| invalid("invalid suffix offset"))? as usize;
        if d >= tokens.len() || o >= tokens[d].len() || !unique.insert((d, o)) {
            return Err(invalid("invalid or repeated suffix pointer"));
        }
        parsed.push((d, o));
    }
    for pair in parsed.windows(2) {
        let a = pair[0];
        let b = pair[1];
        if tokens[a.0][a.1..].cmp(&tokens[b.0][b.1..]).then(a.cmp(&b)) == Ordering::Greater {
            return Err(invalid("suffix array is not sorted"));
        }
    }
    Ok((tokens, parsed))
}
fn prefix_cmp(suffix: &[String], prefix: &[String]) -> Ordering {
    for (a, b) in suffix.iter().zip(prefix) {
        let c = a.cmp(b);
        if c != Ordering::Equal {
            return c;
        }
    }
    if suffix.len() < prefix.len() {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}
pub fn query(input: &Value) -> Result<Value> {
    let index = input
        .get("index")
        .ok_or_else(|| invalid("index required"))?;
    let (tokens, suffixes) = validate(index)?;
    let context = match input.get("context") {
        Some(Value::String(s)) => s.split_whitespace().map(str::to_owned).collect::<Vec<_>>(),
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("context tokens must be strings"))
            })
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(invalid("context must be text or token array")),
    };
    if context.is_empty() || context.len() > 64 {
        return Err(invalid("context must contain 1..64 tokens"));
    }
    let evidence_limit = bounded_usize(input, "evidence_limit", 32, 128)?;
    let mut lo = 0;
    let mut hi = suffixes.len();
    while lo < hi {
        let m = (lo + hi) / 2;
        let (d, o) = suffixes[m];
        if prefix_cmp(&tokens[d][o..], &context) == Ordering::Less {
            lo = m + 1;
        } else {
            hi = m;
        }
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut occurrences = 0;
    let mut end_of_document = 0;
    for &(d, o) in &suffixes[lo..] {
        if prefix_cmp(&tokens[d][o..], &context) != Ordering::Equal {
            break;
        }
        occurrences += 1;
        let next = o + context.len();
        if next < tokens[d].len() {
            *counts.entry(tokens[d][next].clone()).or_default() += 1;
            if evidence.len() < evidence_limit {
                evidence.push(json!({"document_id":index["documents"][d]["id"],"revision":index["documents"][d]["revision"],"token_offset":o,"next_token":tokens[d][next],"license":index["documents"][d]["license"]}));
            }
        } else {
            end_of_document += 1;
        }
    }
    let total = counts.values().sum::<usize>();
    let probabilities=counts.iter().map(|(token,n)|json!({"token":token,"count":n,"probability":*n as f64/total.max(1)as f64})).collect::<Vec<_>>();
    Ok(
        json!({"matched_occurrences":occurrences,"continuation_occurrences":total,"end_of_document_occurrences":end_of_document,"continuations":probabilities,"evidence":evidence,"evidence_truncated":total>evidence_limit,"checksum_verified":true,"document_boundaries_enforced":true,"meaning":"empirical corpus continuation frequencies, not calibrated factual probabilities"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn index() -> Value {
        build(&json!({"documents":[{"id":"a","revision":"1","license":"CC0","content":"a b"},{"id":"b","revision":"1","license":"CC0","content":"c a b d"}]})).expect("build")["index"].clone()
    }
    #[test]
    fn suffix_search_stays_inside_source_documents() {
        let r = query(&json!({"index":index(),"context":"a b"})).expect("query");
        assert_eq!(r["matched_occurrences"], 2);
        assert_eq!(r["end_of_document_occurrences"], 1);
        assert_eq!(r["continuations"][0]["token"], "d");
        assert_eq!(
            query(&json!({"index":index(),"context":"b c"})).expect("query")["matched_occurrences"],
            0
        );
    }
    #[test]
    fn corrupt_or_unsorted_index_fails_closed() {
        let mut i = index();
        i["documents"][0]["tokens"][0] = json!("poison");
        assert!(query(&json!({"index":i,"context":"a"})).is_err());
        let mut i = index();
        i["suffix_array"].as_array_mut().expect("array").swap(0, 1);
        let mut unsigned = i.clone();
        unsigned
            .as_object_mut()
            .expect("object")
            .remove("checksum_sha256");
        i["checksum_sha256"] = json!(digest(&unsigned).expect("hash"));
        assert!(query(&json!({"index":i,"context":"a"})).is_err());
    }
}
