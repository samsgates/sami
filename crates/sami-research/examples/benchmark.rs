use sami_research::dispatch;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

fn tokens(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}
fn bm25_prediction(query: &str, train: &[Value], majority: bool) -> bool {
    let corpus = train
        .iter()
        .map(|r| tokens(r["x"]["message"].as_str().unwrap_or_default()))
        .collect::<Vec<_>>();
    let avg = corpus.iter().map(Vec::len).sum::<usize>() as f64 / corpus.len().max(1) as f64;
    let mut dfs = BTreeMap::new();
    for terms in &corpus {
        let unique = terms.iter().collect::<std::collections::BTreeSet<_>>();
        for t in unique {
            *dfs.entry(t.clone()).or_insert(0usize) += 1;
        }
    }
    let query = tokens(query);
    let mut best = 0.0;
    let mut result = majority;
    for (i, terms) in corpus.iter().enumerate() {
        let mut score = 0.0;
        for q in &query {
            let tf = terms.iter().filter(|w| *w == q).count() as f64;
            let df = *dfs.get(q).unwrap_or(&0) as f64;
            let idf = (1.0 + (corpus.len() as f64 - df + 0.5) / (df + 0.5)).ln();
            score +=
                idf * tf * 2.2 / (tf + 1.2 * (0.25 + 0.75 * terms.len() as f64 / avg.max(1.0)));
        }
        if score > best {
            best = score;
            result = train[i]["y"].as_bool().unwrap_or(majority);
        }
    }
    result
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    // This fixture is declared before induction; the synthesis operation never receives test rows.
    let dataset: Value = serde_json::from_str(include_str!(
        "../../../evaluation/generated_binary_cases.json"
    ))?;
    let partitions = &dataset["partitions"];
    let split_report = dispatch(
        "evaluation.splits",
        &json!({"partitions":partitions,"group_keys":["source_family"],"time_ordered":true}),
    )?;
    if split_report["valid"] != true {
        return Err("fixture split integrity failed".into());
    }
    let train = partitions["train"].as_array().ok_or("train missing")?;
    let test = partitions["locked_transfer_test"]
        .as_array()
        .ok_or("test missing")?;
    let induction = dispatch(
        "synthesis.search",
        &json!({"train":train,"validation":partitions["validation"],"max_candidates":256,"mdl_penalty":0.001}),
    )?;
    let frozen_program = induction["best_program"].clone();
    let fixed_policy = json!({"op":"compare","operator":"le","left":{"op":"field","path":"hours"},"right":{"op":"literal","value":2000}});
    let majority = train.iter().filter(|r| r["y"] == true).count() * 2 >= train.len();
    let labels = test.iter().map(|r| r["y"].clone()).collect::<Vec<_>>();
    let mut results = Vec::new();
    for name in [
        "majority_constant",
        "bm25_nearest_labeled_message",
        "numeric_one_nearest_case",
        "explicit_approved_generated_policy",
        "induced_program",
    ] {
        let start = Instant::now();
        let mut predictions = Vec::new();
        for row in test {
            let predicted = match name {
                "majority_constant" => majority,
                "bm25_nearest_labeled_message" => bm25_prediction(
                    row["x"]["message"].as_str().ok_or("message missing")?,
                    train,
                    majority,
                ),
                "numeric_one_nearest_case" => {
                    let h = row["x"]["hours"].as_f64().ok_or("hours missing")?;
                    let nearest = train
                        .iter()
                        .min_by(|a, b| {
                            let da = (a["x"]["hours"].as_f64().unwrap_or_default() - h).abs();
                            let db = (b["x"]["hours"].as_f64().unwrap_or_default() - h).abs();
                            da.total_cmp(&db)
                        })
                        .ok_or("empty train")?;
                    nearest["y"].as_bool().ok_or("label missing")?
                }
                "explicit_approved_generated_policy" => dispatch(
                    "dsl.execute",
                    &json!({"program":fixed_policy,"data":row["x"]}),
                )?["result"]
                    .as_bool()
                    .ok_or("policy nonboolean")?,
                _ => dispatch(
                    "dsl.execute",
                    &json!({"program":frozen_program,"data":row["x"]}),
                )?["result"]
                    .as_bool()
                    .ok_or("induced result nonboolean")?,
            };
            predictions.push(if predicted { 1.0 } else { 0.0 });
        }
        let elapsed = start.elapsed().as_micros();
        let report = dispatch(
            "evaluation.metrics",
            &json!({"predictions":predictions,"labels":labels,"thresholds":[0.5,0.99]}),
        )?;
        results.push(json!({"baseline":name,"metrics":report,"execution_us":elapsed,"probability_semantics":"hard class indicators; not calibrated confidence estimates"}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"dataset_id":dataset["dataset_id"],"data_classification":dataset["data_classification"],"split_integrity":split_report,"frozen_program":frozen_program,"inner_validation_error":induction["inner_validation_error"],"locked_test_results":results,"sample_count":test.len(),"customer_validation":false,"production_gate_passed":false,"scope":"actual CPU evaluation on small generated binary-hours exercise; not full warranty workflow, causal transfer evidence or LLM superiority","latency_caveat":"single-run harness duration includes native overhead and is not a throughput/service benchmark"})
        )?
    );
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("benchmark failed: {e}");
            ExitCode::FAILURE
        }
    }
}
