use sami_research::dispatch;
use serde_json::{json, Value};

#[test]
fn every_published_lab_example_executes() {
    let examples: Value =
        serde_json::from_str(include_str!("../../../examples/research/operations.json"))
            .expect("valid example fixture");
    for (operation, input) in examples.as_object().expect("examples object") {
        if operation.contains('.') {
            dispatch(operation, input)
                .unwrap_or_else(|error| panic!("documented {operation} example failed: {error}"));
        }
    }
}

#[test]
fn fitted_and_built_artifacts_work_as_followup_inputs() {
    let calibrator=dispatch("calibration.fit",&json!({"method":"platt","split":"calibration","predictions":[0.1,0.2,0.7,0.9],"labels":[0,0,1,1]})).expect("fit");
    let predictions = dispatch(
        "calibration.predict",
        &json!({"calibrator":calibrator["calibrator"],"predictions":[0.15,0.85]}),
    )
    .expect("predict");
    assert!(
        predictions["predictions"][0].as_f64().expect("p")
            < predictions["predictions"][1].as_f64().expect("p")
    );
    let built=dispatch("sequence.build",&json!({"documents":[{"id":"a","revision":"1","license":"Apache-2.0","content":"warranty evidence approved"}]})).expect("build");
    let query = dispatch(
        "sequence.query",
        &json!({"index":built["index"],"context":"warranty evidence"}),
    )
    .expect("query");
    assert_eq!(query["continuations"][0]["token"], "approved");
    let ppmi = dispatch(
        "semantic.ppmi",
        &json!({"documents":["pump oil pressure","pressure oil pump","engine temperature"]}),
    )
    .expect("ppmi");
    let lsa = dispatch("semantic.lsa", &json!({"matrix":ppmi["matrix"],"rank":2})).expect("lsa");
    assert_eq!(
        lsa["input_rows"],
        ppmi["vocabulary"].as_array().expect("vocabulary").len()
    );
}

#[test]
fn materialization_budget_rejects_copy_amplification() {
    let literal = "x".repeat(64 * 1024);
    let rows = vec![json!({}); 200];
    let program = json!({"op":"map","data":{"op":"input"},"expr":{"op":"literal","value":literal}});
    assert!(dispatch("dsl.execute", &json!({"program":program,"data":rows})).is_err());
}
