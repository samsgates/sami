use crate::{array, bounded_usize, invalid, text, ResearchError, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn tokenize(content: &str) -> Vec<String> {
    content
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}
pub fn ppmi(input: &Value) -> Result<Value> {
    let docs = array(input, "documents", 128)?;
    let window = bounded_usize(input, "window", 3, 8)?;
    let maximum = bounded_usize(input, "max_vocabulary", 64, 64)?;
    let mut corpora = Vec::new();
    let mut frequencies: BTreeMap<String, usize> = BTreeMap::new();
    let mut total = 0;
    for d in docs {
        let content = if let Some(s) = d.as_str() {
            s
        } else {
            text(d, "content")?
        };
        let tokens = tokenize(content);
        total += tokens.len();
        if total > 8192 {
            return Err(ResearchError::Limit(
                "semantic corpus exceeds 8192 tokens".into(),
            ));
        }
        for t in &tokens {
            *frequencies.entry(t.clone()).or_default() += 1;
        }
        corpora.push(tokens);
    }
    if frequencies.is_empty() {
        return Err(invalid("semantic corpus contains no terms"));
    }
    let mut vocabulary = frequencies.into_iter().collect::<Vec<_>>();
    vocabulary.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    vocabulary.truncate(maximum);
    let indices = vocabulary
        .iter()
        .enumerate()
        .map(|(i, (w, _))| (w.clone(), i))
        .collect::<BTreeMap<_, _>>();
    let n = vocabulary.len();
    let mut counts = vec![vec![0.0; n]; n];
    for words in corpora {
        for i in 0..words.len() {
            if let Some(&a) = indices.get(&words[i]) {
                for token in words
                    .iter()
                    .take((i + window + 1).min(words.len()))
                    .skip(i + 1)
                {
                    if let Some(&b) = indices.get(token) {
                        counts[a][b] += 1.0;
                        counts[b][a] += 1.0;
                    }
                }
            }
        }
    }
    let row_sums = counts
        .iter()
        .map(|r| r.iter().sum::<f64>())
        .collect::<Vec<_>>();
    let pair_total = row_sums.iter().sum::<f64>();
    let mut matrix = vec![vec![0.0; n]; n];
    if pair_total > 0.0 {
        for i in 0..n {
            for j in 0..n {
                if counts[i][j] > 0.0 && row_sums[i] > 0.0 && row_sums[j] > 0.0 {
                    matrix[i][j] = (counts[i][j] * pair_total / (row_sums[i] * row_sums[j]))
                        .ln()
                        .max(0.0);
                }
            }
        }
    }
    Ok(
        json!({"vocabulary":vocabulary.iter().map(|v|v.0.clone()).collect::<Vec<_>>(),"term_frequencies":vocabulary.iter().map(|v|v.1).collect::<Vec<_>>(),"matrix":matrix,"cooccurrence_count":pair_total,"window":window,"tokenizer":"unicode_alphanumeric_lowercase_v1","document_boundaries_enforced":true,"neural_embeddings_required":false,"scope":"bounded word-context statistics; not evidence of general semantic understanding"}),
    )
}
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn normalize(v: &mut [f64]) -> f64 {
    let norm = dot(v, v).sqrt();
    if norm > 1e-12 {
        for x in v {
            *x /= norm;
        }
    }
    norm
}
fn orthogonalize(v: &mut [f64], basis: &[Vec<f64>]) {
    for b in basis {
        let p = dot(v, b);
        for (x, y) in v.iter_mut().zip(b) {
            *x -= p * y;
        }
    }
}
pub fn lsa(input: &Value) -> Result<Value> {
    let rows = array(input, "matrix", 64)?;
    if rows.is_empty() {
        return Err(invalid("matrix cannot be empty"));
    }
    let cols = rows[0]
        .as_array()
        .map(Vec::len)
        .filter(|n| *n > 0 && *n <= 64)
        .ok_or_else(|| invalid("matrix must have 1..64 columns"))?;
    let mut matrix = Vec::new();
    for row in rows {
        let row = row
            .as_array()
            .filter(|r| r.len() == cols)
            .ok_or_else(|| invalid("matrix must be rectangular"))?;
        matrix.push(
            row.iter()
                .map(|v| {
                    v.as_f64()
                        .filter(|n| n.is_finite() && n.abs() <= 1e6)
                        .ok_or_else(|| invalid("bounded finite matrix values required"))
                })
                .collect::<Result<Vec<_>>>()?,
        );
    }
    let rank = bounded_usize(input, "rank", 4, 16)?
        .min(cols)
        .min(matrix.len());
    let iterations = bounded_usize(input, "iterations", 100, 400)?;
    let mut gram = vec![vec![0.0; cols]; cols];
    for row in &matrix {
        for i in 0..cols {
            for j in 0..cols {
                gram[i][j] += row[i] * row[j];
            }
        }
    }
    let mut components: Vec<Vec<f64>> = Vec::new();
    let mut singular = Vec::new();
    let mut residuals = Vec::new();
    for k in 0..rank {
        let mut v = (0..cols)
            .map(|i| ((i + 1) as f64 * (k + 1) as f64 * 1.61803398875).sin() + 0.25)
            .collect::<Vec<_>>();
        orthogonalize(&mut v, &components);
        if normalize(&mut v) < 1e-12 {
            break;
        }
        for _ in 0..iterations {
            let mut next = gram.iter().map(|row| dot(row, &v)).collect::<Vec<_>>();
            orthogonalize(&mut next, &components);
            if normalize(&mut next) < 1e-12 {
                break;
            }
            v = next;
        }
        let gv = gram.iter().map(|row| dot(row, &v)).collect::<Vec<_>>();
        let lambda = dot(&v, &gv).max(0.0);
        if lambda.sqrt() < 1e-10 {
            break;
        }
        let residual = gv
            .iter()
            .zip(&v)
            .map(|(g, v)| (g - lambda * v).powi(2))
            .sum::<f64>()
            .sqrt();
        components.push(v);
        singular.push(lambda.sqrt());
        residuals.push(residual);
    }
    let embeddings = matrix
        .iter()
        .map(|r| components.iter().map(|c| dot(r, c)).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut reconstruction_error = 0.0;
    for (row, e) in matrix.iter().zip(&embeddings) {
        for (j, &x) in row.iter().enumerate() {
            let reconstruction = components.iter().zip(e).map(|(c, s)| c[j] * s).sum::<f64>();
            reconstruction_error += (x - reconstruction).powi(2);
        }
    }
    Ok(
        json!({"components":components,"singular_values":singular,"embeddings":embeddings,"eigen_residuals":residuals,"reconstruction_frobenius_error":reconstruction_error.sqrt(),"solver":"bounded_orthogonalized_power_iteration","convergence_certified":false,"iterations":iterations,"input_rows":matrix.len(),"input_columns":cols,"effective_rank":singular.len(),"neural_embeddings_required":false}),
    )
}
fn hash64(seed: u64, text: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    hasher.update(text.as_bytes());
    let bytes = hasher.finalize();
    let mut first = [0u8; 8];
    first.copy_from_slice(&bytes[..8]);
    u64::from_le_bytes(first)
}
pub fn signature(input: &Value) -> Result<Value> {
    let words = tokenize(text(input, "content")?);
    if words.len() > 8192 {
        return Err(ResearchError::Limit(
            "signature content exceeds 8192 terms".into(),
        ));
    }
    let k = bounded_usize(input, "shingle_size", 1, 5)?;
    if words.len() < k {
        return Err(invalid("not enough terms to build requested shingles"));
    }
    let shingles = words.windows(k).map(|w| w.join(" ")).collect::<Vec<_>>();
    let set = shingles.iter().collect::<BTreeSet<_>>();
    let permutations = bounded_usize(input, "permutations", 32, 128)?;
    let minhash = (0..permutations)
        .map(|seed| {
            set.iter()
                .map(|s| hash64(seed as u64, s))
                .min()
                .map(|h| format!("{h:016x}"))
                .ok_or_else(|| invalid("empty shingle set"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut bits = [0i32; 64];
    for shingle in &shingles {
        let h = hash64(0x53414d49, shingle);
        for (i, b) in bits.iter_mut().enumerate() {
            *b += if h & (1u64 << i) != 0 { 1 } else { -1 };
        }
    }
    let mut simhash = 0u64;
    for (i, &b) in bits.iter().enumerate() {
        if b >= 0 {
            simhash |= 1u64 << i;
        }
    }
    Ok(
        json!({"minhash":minhash,"simhash":format!("{simhash:016x}"),"shingle_size":k,"unique_shingles":set.len(),"tokenizer":"unicode_alphanumeric_lowercase_v1","hash":"sha256_first64_le","scope":"duplicate/set-similarity signals; not semantic equivalence"}),
    )
}
pub fn similarity(input: &Value) -> Result<Value> {
    match text(input, "kind")? {
        "minhash" => {
            let l = array(input, "left", 128)?;
            let r = array(input, "right", 128)?;
            if l.is_empty() || l.len() != r.len() {
                return Err(invalid("equal nonempty signatures required"));
            }
            for value in l.iter().chain(r) {
                let s = value
                    .as_str()
                    .ok_or_else(|| invalid("signature entries must be hex"))?;
                if s.len() != 16 || u64::from_str_radix(s, 16).is_err() {
                    return Err(invalid("signature hash must have 16 hex digits"));
                }
            }
            let matches = l.iter().zip(r).filter(|(a, b)| a == b).count();
            Ok(
                json!({"estimated_jaccard":matches as f64/l.len()as f64,"permutations":l.len(),"requires_matching_tokenizer_shingle_size_and_hash_version":true}),
            )
        }
        "simhash" => {
            let parse = |key: &str| -> Result<u64> {
                let s = text(input, key)?;
                if s.len() != 16 {
                    return Err(invalid("simhash must have 16 hex digits"));
                }
                u64::from_str_radix(s, 16).map_err(|_| invalid("invalid simhash"))
            };
            let d = (parse("left")? ^ parse("right")?).count_ones();
            Ok(json!({"hamming_distance":d,"bit_agreement":1.0-d as f64/64.0}))
        }
        "cosine" => {
            let l = array(input, "left", 64)?;
            let r = array(input, "right", 64)?;
            if l.len() != r.len() || l.is_empty() {
                return Err(invalid("equal nonempty vectors required"));
            }
            let parse = |v: &[Value]| {
                v.iter()
                    .map(|x| {
                        x.as_f64()
                            .filter(|x| x.is_finite() && x.abs() <= 1e6)
                            .ok_or_else(|| invalid("finite vector values required"))
                    })
                    .collect::<Result<Vec<_>>>()
            };
            let l = parse(l)?;
            let r = parse(r)?;
            let norms = dot(&l, &l).sqrt() * dot(&r, &r).sqrt();
            Ok(json!({"cosine":if norms>0.0{Some((dot(&l,&r)/norms).clamp(-1.0,1.0))}else{None}}))
        }
        _ => Err(invalid("kind must be minhash, simhash or cosine")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ppmi_never_counts_cross_document_context() {
        let r = ppmi(&json!({"documents":["apple","banana"],"window":1})).expect("ppmi");
        assert_eq!(r["cooccurrence_count"], 0.0);
    }
    #[test]
    fn lsa_reconstructs_a_rank_one_matrix() {
        let r = lsa(&json!({"matrix":[[1,2],[2,4]],"rank":1})).expect("lsa");
        assert!(
            r["reconstruction_frobenius_error"]
                .as_f64()
                .expect("residual")
                < 1e-8
        );
    }
    #[test]
    fn duplicate_signatures_have_full_agreement() {
        let a = signature(&json!({"content":"the same sentence twice"})).expect("signature");
        let b = signature(&json!({"content":"THE same sentence TWICE"})).expect("signature");
        assert_eq!(a["minhash"], b["minhash"]);
        assert_eq!(
            similarity(&json!({"kind":"minhash","left":a["minhash"],"right":b["minhash"]}))
                .expect("similarity")["estimated_jaccard"],
            1.0
        );
    }
    #[test]
    fn zero_norm_cosine_is_unknown() {
        assert!(
            similarity(&json!({"kind":"cosine","left":[0,0],"right":[1,1]})).expect("similarity")
                ["cosine"]
                .is_null()
        );
    }
}
