use sami_intelligence::{
    compose, decide, default_pack, evaluate_rules, graph_query, parse, plan, roleplay, search,
};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../packs/industrial-service/fixtures.json"
    ))
    .unwrap()
}
fn input(message: &str, asset: &str) -> Value {
    let f = fixture();
    json!({"message":message,"asset_id":asset,"as_of":f["as_of"],"known_at":f["known_at"],"sources":f["sources"]})
}
fn claims() -> Vec<Value> {
    fixture()["claims"].as_array().unwrap().clone()
}

#[test]
fn api_shape_routes_supported_service_and_never_authorizes_effect() {
    let out = decide(
        "service_triage",
        &input("Serial 1600 is overheating", "asset:unit_84"),
        &claims(),
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["status"], "resolved");
    assert_eq!(out["value"]["queue"], "thermal_service_team");
    assert_eq!(out["authorized_effects"], json!([]));
    assert!(out["probabilities"].is_null());
    assert!(out["response"]
        .as_str()
        .unwrap()
        .contains("thermal_service_team"));
    assert!(out["evidence"]["claim_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v == "bulletin_overheating"));
}
#[test]
fn warranty_calendar_and_hours_boundaries_are_explicit_and_human_reviewed() {
    let mut i = input("Check warranty for serial 1600", "asset:unit_84");
    let out = decide("warranty_prequalification", &i, &claims(), &default_pack()).unwrap();
    assert_eq!(out["status"], "needs_human");
    assert_eq!(out["value"]["prequalification"], "potentially_eligible");
    assert_eq!(out["checks"][0]["expiry_exclusive"], "2027-08-12");
    i["as_of"] = json!("2027-08-12");
    i["known_at"] = json!("2027-08-12T06:00:00Z");
    let expired = decide("warranty_prequalification", &i, &claims(), &default_pack()).unwrap();
    assert_eq!(
        expired["value"]["prequalification"],
        "potentially_ineligible"
    );
    let hours = decide(
        "warranty_prequalification",
        &input("Check warranty for serial 1200", "asset:unit_42"),
        &claims(),
        &default_pack(),
    )
    .unwrap();
    assert_eq!(hours["checks"][1]["result"], "fail");
}
#[test]
fn wrong_serial_and_unconfirmed_hours_cannot_overwrite_approved_records() {
    let out = decide(
        "warranty_prequalification",
        &input("Serial 1200 has 5 hours", "asset:unit_84"),
        &claims(),
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["status"], "conflicting_evidence");
    assert!(out["value"].is_null());
    assert_eq!(out["authorized_effects"], json!([]));
}
#[test]
fn source_revision_retraction_or_expiry_invalidates_operational_evidence() {
    let mut i = input("Serial 1600 is overheating", "asset:unit_84");
    i["sources"][1]["revision"] = json!("3");
    let out = decide("service_triage", &i, &claims(), &default_pack()).unwrap();
    assert_eq!(out["status"], "insufficient_evidence");
    i["sources"][0]["status"] = json!("retracted");
    let out = decide("warranty_prequalification", &i, &claims(), &default_pack()).unwrap();
    assert_eq!(out["status"], "insufficient_evidence");
    let mut i = input("Check warranty", "asset:unit_84");
    i["sources"][0]["max_age_seconds"] = json!(60);
    i["sources"][0]["permission_checked_at"] = json!("2026-10-01T06:00:00Z");
    assert_eq!(
        decide("warranty_prequalification", &i, &claims(), &default_pack()).unwrap()["status"],
        "insufficient_evidence"
    );
}
#[test]
fn missing_records_do_not_become_a_warranty_denial() {
    let out = decide(
        "warranty_prequalification",
        &input("Check warranty", "asset:unit_84"),
        &[],
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["status"], "insufficient_evidence");
    assert!(out["value"].is_null());
}
#[test]
fn duplicate_sources_do_not_create_confidence_and_conflicts_are_retained() {
    let mut c = claims();
    c.push(c[6].clone());
    let first = decide(
        "warranty_prequalification",
        &input("Check warranty", "asset:unit_84"),
        &c,
        &default_pack(),
    )
    .unwrap();
    assert_eq!(first["status"], "needs_human");
    assert!(first["confidence"].is_null());
    let mut conflict = c[9].clone();
    conflict["id"] = json!("conflicting_hours");
    conflict["value"] = json!(1500);
    c.push(conflict);
    assert_eq!(
        decide(
            "warranty_prequalification",
            &input("Check warranty", "asset:unit_84"),
            &c,
            &default_pack()
        )
        .unwrap()["status"],
        "conflicting_evidence"
    );
}
#[test]
fn unit_conversion_and_out_of_scope_policy_are_checked() {
    let mut c = claims();
    c[9]["value"] = json!(48000);
    c[9]["unit"] = json!("minute");
    let out = decide(
        "warranty_prequalification",
        &input("Check warranty", "asset:unit_84"),
        &c,
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["checks"][1]["value"], 800.0);
    c[8]["value"] = json!("US");
    assert_eq!(
        decide(
            "warranty_prequalification",
            &input("Check warranty", "asset:unit_84"),
            &c,
            &default_pack()
        )
        .unwrap()["status"],
        "unsupported"
    );
}
#[test]
fn bitemporal_claims_and_missing_or_cyclic_parents_fail_closed() {
    let mut c = claims();
    c[9]["system_from"] = json!("2026-10-03T00:00:00Z");
    assert_eq!(
        decide(
            "warranty_prequalification",
            &input("Check warranty", "asset:unit_84"),
            &c,
            &default_pack()
        )
        .unwrap()["status"],
        "insufficient_evidence"
    );
    c[9]["system_from"] = json!("2026-10-01T00:00:00Z");
    let id = c[9]["id"].clone();
    c[9]["depends_on"] = json!([id]);
    assert_eq!(
        decide(
            "warranty_prequalification",
            &input("Check warranty", "asset:unit_84"),
            &c,
            &default_pack()
        )
        .unwrap()["status"],
        "insufficient_evidence"
    );
}
#[test]
fn parser_preserves_negation_entity_and_time_ambiguity() {
    let p = default_pack();
    let out = parse(
        "It is not overheating but has an oil leak. Serial 1600 has 800 hours.",
        &json!({}),
        &p,
    );
    assert_eq!(out["fields"]["issue"], "issue.fluid_leak");
    assert_eq!(out["fields"]["operating_hours"], 800.0);
    assert_eq!(out["fields"]["asset_id"], "asset:unit_84");
    let out = parse(
        "It is not the 2023 unit; the newer one has 800 hours",
        &json!({"assets":p["assets"]}),
        &p,
    );
    assert_eq!(out["fields"]["asset_id"], "asset:unit_84");
    let out = parse(
        "Serial 1600 has 800 hours or 900 hours; service date 01/02/2026",
        &json!({}),
        &p,
    );
    assert_eq!(out["status"], "ambiguous");
    assert!(out["fields"].get("operating_hours").is_none());
    let out = parse("İstanbul: not overheating but oil leak", &json!({}), &p);
    assert_eq!(out["fields"]["issue"], "issue.fluid_leak");
}
#[test]
fn every_registered_typed_decision_has_an_executable_contract() {
    let p = default_pack();
    assert_eq!(
        decide(
            "financial_objection",
            &json!({"message":"This is too expensive"}),
            &[],
            &p
        )
        .unwrap()["value"],
        true
    );
    assert_eq!(
        decide(
            "financial_objection",
            &json!({"message":"This is not expensive"}),
            &[],
            &p
        )
        .unwrap()["value"],
        false
    );
    assert_eq!(
        decide(
            "objection_type",
            &json!({"fields":{"objection":"technical"}}),
            &[],
            &p
        )
        .unwrap()["value"],
        "technical"
    );
    assert_eq!(
        decide(
            "case_flags",
            &json!({"fields":{"urgent":true,"documents_missing":false}}),
            &[],
            &p
        )
        .unwrap()["value"],
        json!(["urgent"])
    );
    assert_eq!(
        decide(
            "simulation_readiness",
            &json!({"fields":{"readiness":62}}),
            &[],
            &p
        )
        .unwrap()["value"]["value"],
        62.0
    );
    assert_eq!(
        decide(
            "next_step_rank",
            &json!({"fields":{"documents_missing":true}}),
            &[],
            &p
        )
        .unwrap()["value"][0]["value"],
        "clarify"
    );
    assert_eq!(
        decide(
            "extract_case_fields",
            &json!({"message":"Serial 1600 has 800 hours"}),
            &[],
            &p
        )
        .unwrap()["value"]["serial_number"],
        "1600"
    );
    assert_eq!(
        decide(
            "next_action",
            &json!({"fields":{"documents_missing":false}}),
            &[],
            &p
        )
        .unwrap()["value"],
        "prepare_case"
    );
    assert!(decide("arbitrary free-form question", &json!({}), &[], &p).is_err());
}
#[test]
fn rules_deny_overrides_allow_and_unknown_is_not_false() {
    let r = json!([{"id":"allow","effect":"allow","priority":100,"when":true},{"id":"deny","effect":"deny","priority":1,"when":{"field":"action","op":"eq","value":"payout"}}]);
    let out = evaluate_rules(&r, &json!({"action":"payout"}), 100).unwrap();
    assert_eq!(out["status"], "policy_denied");
    assert_eq!(out["allowed"], false);
    let r = json!([{"id":"absence_is_not_false","effect":"deny","when":{"not":{"field":"entitled","op":"eq","value":true}}}]);
    let out = evaluate_rules(&r, &json!({}), 100).unwrap();
    assert_eq!(out["denied"], false);
    assert_eq!(out["unknown_conditions"], json!(["absence_is_not_false"]));
    assert!(evaluate_rules(&r, &json!({}), 0).is_err());
}
#[test]
fn equal_priority_rule_conflicts_and_authority_mutations_are_rejected() {
    let r = json!([{"id":"a","priority":5,"set":{"queue":"a"}},{"id":"b","priority":5,"set":{"queue":"b"}}]);
    assert_eq!(
        evaluate_rules(&r, &json!({}), 20).unwrap()["status"],
        "policy_conflict"
    );
    assert!(evaluate_rules(
        &json!([{"id":"bad","set":{"permissions.write":true}}]),
        &json!({}),
        10
    )
    .is_err());
}
#[test]
fn planner_finds_lowest_cost_bounded_path_without_executing_tools() {
    let p = default_pack();
    let registry = &p["workflows"]["service_case"];
    let out = plan(
        "case_prepared",
        &json!({"asset_confirmed":false,"evidence_complete":false,"case":{"status":"new"}}),
        registry,
        100,
    )
    .unwrap();
    assert_eq!(out["status"], "planned");
    assert_eq!(out["steps"].as_array().unwrap().len(), 3);
    assert_eq!(out["cost"], 4.0);
    assert_eq!(out["authorized_effects"], json!([]));
    assert_eq!(
        plan(
            "case_prepared",
            &json!({"asset_confirmed":false,"evidence_complete":false}),
            registry,
            1
        )
        .unwrap()["status"],
        "budget_exhausted"
    );
}
#[test]
fn graph_cycles_are_bounded_and_unapproved_relations_do_not_enter_paths() {
    let c = json!([{"id":"a","subject":"A","predicate":"part_of","value":"B","status":"approved"},{"id":"b","subject":"B","predicate":"part_of","value":"A","status":"approved"},{"id":"secret","subject":"B","predicate":"part_of","value":"C","status":"candidate"}]);
    let out = graph_query(
        &json!({"start":"A","predicate":"part_of"}),
        c.as_array().unwrap(),
        3,
    )
    .unwrap();
    assert_eq!(out["visited"], 2);
    assert!(!out["paths"].to_string().contains("secret"));
}
#[test]
fn search_exposes_rank_components_and_recovers_approved_aliases() {
    let out = search(
        "entitlement",
        &[
            json!({"id":"w","content":"Warranty coverage requirements and operating hours"}),
            json!({"id":"u","content":"Unrelated gardening tips"}),
        ],
        5,
    );
    assert_eq!(out["results"][0]["id"], "w");
    assert_eq!(out["fusion"]["calibrated"], false);
    assert!(
        out["results"][0]["components"]["sparse_alias_cosine"]
            .as_f64()
            .unwrap()
            > 0.0
    );
}
#[test]
fn grammar_protects_slots_and_varies_only_approved_wording() {
    let p = default_pack();
    let plan = json!({"action":"service_route","slots":{"asset_id":"unit 84","queue":"thermal"}});
    let first = compose(&plan, &p, &[]).unwrap();
    let second = compose(&plan, &p, &[first.clone()]).unwrap();
    assert_ne!(first, second);
    assert!(compose(
        &json!({"action":"service_route","slots":{"asset_id":"84"}}),
        &p,
        &[]
    )
    .is_err());
    assert!(compose(&json!({"action":"service_route","slots":{"asset_id":"84","queue":"secret"},"hidden_slots":["queue"]}),&p,&[]).is_err());
}
#[test]
fn roleplay_reveals_one_fact_and_does_not_postpone_or_expose_hidden_state() {
    let p = default_pack();
    let persona = &p["roleplay_persona"];
    let first = roleplay("What problems do you face?", &json!({}), persona);
    assert_eq!(first["revealed_this_turn"], json!(["fuel_consumption"]));
    assert!(!first["response"]
        .as_str()
        .unwrap()
        .contains("Service delays"));
    let second = roleplay("Let me understand maintenance", &first["state"], persona);
    assert_eq!(second["revealed_this_turn"], json!(["service_delay"]));
    let later = roleplay("Send logs tomorrow", &second["state"], persona);
    assert_eq!(later["revealed_this_turn"], json!([]));
    assert!(later["response"].as_str().unwrap().contains("this meeting"));
    assert_eq!(later["real_customer_psychology"], false);
}

#[test]
fn conflicting_applicable_bulletins_are_not_resolved_by_input_order() {
    let mut c = claims();
    let mut alternative = c[12].clone();
    alternative["id"] = json!("alternate_thermal_bulletin");
    alternative["value"]["instruction"] = json!("A conflicting approved instruction");
    c.push(alternative);
    let out = decide(
        "service_triage",
        &input("Serial 1600 is overheating", "asset:unit_84"),
        &c,
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["status"], "conflicting_evidence");
    assert!(out["value"].is_null());
}

#[test]
fn dimensional_comparisons_remain_unknown_and_higher_priority_resolves_preferences() {
    let mismatch = json!([{"id":"dimension","effect":"allow","when":{"field":"measurement","op":"gt","value":{"value":2,"unit":"second"}}}]);
    let out = evaluate_rules(
        &mismatch,
        &json!({"measurement":{"value":5,"unit":"meter"}}),
        20,
    )
    .unwrap();
    assert_eq!(out["allowed"], false);
    assert_eq!(out["unknown_conditions"], json!(["dimension"]));
    let preferences = json!([{"id":"a","priority":5,"set":{"queue":"a"}},{"id":"b","priority":5,"set":{"queue":"b"}},{"id":"mandatory","priority":10,"set":{"queue":"c"}}]);
    let out = evaluate_rules(&preferences, &json!({}), 20).unwrap();
    assert_eq!(out["status"], "resolved");
    assert_eq!(out["updates"]["queue"], "c");
}

#[test]
fn unsupported_language_and_partial_ranking_do_not_create_false_certainty() {
    let out = decide(
        "financial_objection",
        &json!({"message":"这是一个全新的话题"}),
        &[],
        &default_pack(),
    )
    .unwrap();
    assert_eq!(out["status"], "unsupported");
    assert!(out["value"].is_null());
    let out = decide("next_step_rank", &json!({}), &[], &default_pack()).unwrap();
    assert_eq!(out["status"], "insufficient_evidence");
    assert!(out["probabilities"].is_null());
}
