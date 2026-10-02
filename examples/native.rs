use serde_json::{json, Value};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fixture: Value =
        serde_json::from_str(include_str!("../packs/industrial-service/fixtures.json"))?;
    let claims = fixture["claims"].as_array().ok_or("claims missing")?;
    let input = json!({"asset_id":"asset:unit_84","message":"Serial 1600 is overheating","as_of":fixture["as_of"],"known_at":fixture["known_at"],"sources":fixture["sources"]});
    let decision = sami_intelligence::decide(
        "service_triage",
        &input,
        claims,
        &sami_intelligence::default_pack(),
    )?;
    println!("{}", serde_json::to_string_pretty(&decision)?);
    Ok(())
}
