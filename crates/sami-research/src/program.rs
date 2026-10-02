use crate::{array, bounded_usize, digest, invalid, text, ResearchError, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

struct Evaluator {
    remaining: usize,
    max_rows: usize,
    bytes_remaining: usize,
}
fn field(data: &Value, path: &str) -> Result<Value> {
    if path.len() > 256 || path.is_empty() {
        return Err(invalid("field path must be 1..256 bytes"));
    }
    let mut value = data;
    for part in path.split('.') {
        if part.is_empty() {
            return Err(invalid("empty field path component"));
        }
        value = value.get(part).unwrap_or(&Value::Null);
    }
    Ok(value.clone())
}
fn numeric(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .filter(|x| x.is_finite())
        .ok_or_else(|| invalid("numeric operand required"))
}
fn comparison(op: &str, a: &Value, b: &Value) -> Result<bool> {
    match op {
        "eq" | "ne" => {
            if a.is_number() && b.is_number() {
                let same = numeric(a)? == numeric(b)?;
                Ok(if op == "eq" { same } else { !same })
            } else {
                let same = a == b;
                Ok(if op == "eq" { same } else { !same })
            }
        }
        "lt" | "le" | "gt" | "ge" => {
            let x = numeric(a)?;
            let y = numeric(b)?;
            Ok(match op {
                "lt" => x < y,
                "le" => x <= y,
                "gt" => x > y,
                _ => x >= y,
            })
        }
        _ => Err(invalid("unknown comparison operator")),
    }
}
impl Evaluator {
    fn eval(&mut self, program: &Value, data: &Value, depth: usize) -> Result<Value> {
        if depth > 24 {
            return Err(ResearchError::Limit("DSL recursion exceeds 24".into()));
        }
        if self.remaining == 0 {
            return Err(ResearchError::Limit(
                "DSL instruction budget exhausted".into(),
            ));
        }
        self.remaining -= 1;
        let op = text(program, "op")?;
        let arg = |key: &str| {
            program
                .get(key)
                .ok_or_else(|| invalid(format!("{op}.{key} required")))
        };
        let output = match op {
            "literal" => Ok(arg("value")?.clone()),
            "field" => field(data, text(program, "path")?),
            "input" => Ok(data.clone()),
            "compare" => {
                let a = self.eval(arg("left")?, data, depth + 1)?;
                let b = self.eval(arg("right")?, data, depth + 1)?;
                Ok(json!(comparison(text(program, "operator")?, &a, &b)?))
            }
            "and" | "or" => {
                let args = array(program, "args", 16)?;
                if args.is_empty() {
                    return Err(invalid("boolean args cannot be empty"));
                }
                let mut result = op == "and";
                for a in args {
                    let v = self
                        .eval(a, data, depth + 1)?
                        .as_bool()
                        .ok_or_else(|| invalid("boolean operand required"))?;
                    if op == "and" {
                        result &= v;
                    } else {
                        result |= v;
                    }
                }
                Ok(json!(result))
            }
            "not" => Ok(json!(!self
                .eval(arg("value")?, data, depth + 1)?
                .as_bool()
                .ok_or_else(|| invalid("boolean operand required"))?)),
            "if" => {
                let condition = self
                    .eval(arg("condition")?, data, depth + 1)?
                    .as_bool()
                    .ok_or_else(|| invalid("if condition must be boolean"))?;
                self.eval(
                    arg(if condition { "then" } else { "else" })?,
                    data,
                    depth + 1,
                )
            }
            "add" | "subtract" | "multiply" | "divide" => {
                let a = numeric(&self.eval(arg("left")?, data, depth + 1)?)?;
                let b = numeric(&self.eval(arg("right")?, data, depth + 1)?)?;
                if op == "divide" && b == 0.0 {
                    return Err(invalid("division by zero"));
                }
                let x = match op {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    _ => a / b,
                };
                if !x.is_finite() {
                    return Err(invalid("nonfinite arithmetic result"));
                }
                Ok(json!(x))
            }
            "convert" => {
                let x = numeric(&self.eval(arg("value")?, data, depth + 1)?)?;
                Ok(json!(convert(
                    x,
                    text(program, "from")?,
                    text(program, "to")?
                )?))
            }
            "project" => {
                let fields = arg("fields")?
                    .as_object()
                    .ok_or_else(|| invalid("project.fields must be an object"))?;
                if fields.len() > 64 {
                    return Err(ResearchError::Limit("projection exceeds 64 fields".into()));
                }
                let mut output = Map::new();
                for (k, p) in fields {
                    output.insert(k.clone(), self.eval(p, data, depth + 1)?);
                }
                Ok(Value::Object(output))
            }
            "filter" | "map" => {
                let values = self.eval(arg("data")?, data, depth + 1)?;
                let values = values
                    .as_array()
                    .ok_or_else(|| invalid("collection data must be an array"))?;
                if values.len() > self.max_rows {
                    return Err(ResearchError::Limit(
                        "DSL collection exceeds row budget".into(),
                    ));
                }
                let mut rows = Vec::new();
                for row in values {
                    if op == "filter" {
                        if self
                            .eval(arg("predicate")?, row, depth + 1)?
                            .as_bool()
                            .ok_or_else(|| invalid("filter predicate must be boolean"))?
                        {
                            rows.push(row.clone());
                        }
                    } else {
                        rows.push(self.eval(arg("expr")?, row, depth + 1)?);
                    }
                }
                Ok(json!(rows))
            }
            "join" => {
                let left = self.eval(arg("left")?, data, depth + 1)?;
                let right = self.eval(arg("right")?, data, depth + 1)?;
                let left = left
                    .as_array()
                    .ok_or_else(|| invalid("join.left must resolve to array"))?;
                let right = right
                    .as_array()
                    .ok_or_else(|| invalid("join.right must resolve to array"))?;
                if left.len() > self.max_rows || right.len() > self.max_rows {
                    return Err(ResearchError::Limit("join inputs exceed row budget".into()));
                }
                let lk = text(program, "left_key")?;
                let rk = text(program, "right_key")?;
                let mut index: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
                for row in right {
                    let k = field(row, rk)?;
                    if !k.is_null() {
                        index.entry(join_key(&k)?).or_default().push(row);
                    }
                }
                let mut rows = Vec::new();
                for l in left {
                    let k = field(l, lk)?;
                    if !k.is_null() {
                        if let Some(matches) = index.get(&join_key(&k)?) {
                            for r in matches {
                                if self.remaining == 0 || rows.len() >= self.max_rows {
                                    return Err(ResearchError::Limit(
                                        "join expansion exceeds budget".into(),
                                    ));
                                }
                                self.remaining -= 1;
                                rows.push(json!({"left":l,"right":r}));
                            }
                        }
                    }
                }
                Ok(json!(rows))
            }
            "count" => {
                let value = self.eval(arg("data")?, data, depth + 1)?;
                let rows = value
                    .as_array()
                    .ok_or_else(|| invalid("count data must be an array"))?;
                if rows.len() > self.max_rows {
                    return Err(ResearchError::Limit(
                        "count input exceeds row budget".into(),
                    ));
                }
                Ok(json!(rows.len()))
            }
            _ => Err(ResearchError::Unsupported(format!("DSL operation {op}"))),
        }?;
        let size = serde_json::to_vec(&output)
            .map_err(|e| invalid(e.to_string()))?
            .len();
        if size > self.bytes_remaining {
            return Err(ResearchError::Limit(
                "DSL cumulative materialization exceeds 8 MiB".into(),
            ));
        }
        self.bytes_remaining -= size;
        Ok(output)
    }
}
fn join_key(k: &Value) -> Result<String> {
    if k.is_number() {
        return Ok(format!("number:{}", numeric(k)?));
    }
    if !(k.is_string() || k.is_boolean()) {
        return Err(invalid("join keys must be string, number or boolean"));
    }
    serde_json::to_string(k).map_err(|e| invalid(e.to_string()))
}
fn convert(x: f64, from: &str, to: &str) -> Result<f64> {
    let dimension = |u: &str| -> Option<(&str, f64, f64)> {
        match u {
            "Pa" => Some(("pressure", 1.0, 0.0)),
            "kPa" => Some(("pressure", 1000.0, 0.0)),
            "MPa" => Some(("pressure", 1_000_000.0, 0.0)),
            "bar" => Some(("pressure", 100_000.0, 0.0)),
            "psi" => Some(("pressure", 6894.757293168, 0.0)),
            "m" => Some(("length", 1.0, 0.0)),
            "mm" => Some(("length", 0.001, 0.0)),
            "cm" => Some(("length", 0.01, 0.0)),
            "km" => Some(("length", 1000.0, 0.0)),
            "in" => Some(("length", 0.0254, 0.0)),
            "K" => Some(("temperature", 1.0, 0.0)),
            "C" => Some(("temperature", 1.0, 273.15)),
            "F" => Some(("temperature", 5.0 / 9.0, 273.15 - 32.0 * 5.0 / 9.0)),
            "s" => Some(("time", 1.0, 0.0)),
            "min" => Some(("time", 60.0, 0.0)),
            "h" => Some(("time", 3600.0, 0.0)),
            _ => None,
        }
    };
    let f = dimension(from).ok_or_else(|| invalid("unsupported source unit"))?;
    let t = dimension(to).ok_or_else(|| invalid("unsupported target unit"))?;
    if f.0 != t.0 {
        return Err(invalid("unit dimensions do not match"));
    }
    let result = (x * f.1 + f.2 - t.2) / t.1;
    if !result.is_finite() {
        return Err(invalid("nonfinite conversion"));
    }
    if f.0 == "temperature" && x * f.1 + f.2 < 0.0 {
        return Err(invalid("temperature below absolute zero"));
    }
    Ok(result)
}
pub fn execute(input: &Value) -> Result<Value> {
    let program = input
        .get("program")
        .ok_or_else(|| invalid("program required"))?;
    let data = input.get("data").ok_or_else(|| invalid("data required"))?;
    let max_nodes = bounded_usize(input, "max_nodes", 10_000, 50_000)?;
    let max_rows = bounded_usize(input, "max_rows", 1000, 2000)?;
    let mut evaluator = Evaluator {
        remaining: max_nodes,
        max_rows,
        bytes_remaining: 8 * 1024 * 1024,
    };
    let result = evaluator.eval(program, data, 0)?;
    if serde_json::to_vec(&result)
        .map_err(|e| invalid(e.to_string()))?
        .len()
        > 2 * 1024 * 1024
    {
        return Err(ResearchError::Limit("DSL result exceeds 2 MiB".into()));
    }
    Ok(
        json!({"result":result,"instructions_used":max_nodes-evaluator.remaining,"side_effects":false}),
    )
}
fn classify(condition: Value, positive: &Value, negative: &Value) -> Value {
    json!({"op":"if","condition":condition,"then":{"op":"literal","value":positive},"else":{"op":"literal","value":negative}})
}
fn predicate(key: &str, operator: &str, value: Value) -> Value {
    json!({"op":"compare","operator":operator,"left":{"op":"field","path":key},"right":{"op":"literal","value":value}})
}
fn score(program: &Value, rows: &[Value]) -> Result<f64> {
    let mut errors = 0usize;
    for row in rows {
        let x = row
            .get("x")
            .ok_or_else(|| invalid("synthesis rows need x"))?;
        let y = row
            .get("y")
            .ok_or_else(|| invalid("synthesis rows need y"))?;
        let mut e = Evaluator {
            remaining: 200,
            max_rows: 64,
            bytes_remaining: 8 * 1024 * 1024,
        };
        if e.eval(program, x, 0)? != *y {
            errors += 1;
        }
    }
    Ok(errors as f64 / rows.len() as f64)
}
fn node_count(program: &Value) -> usize {
    match program {
        Value::Object(obj) => {
            usize::from(obj.contains_key("op")) + obj.values().map(node_count).sum::<usize>()
        }
        Value::Array(a) => a.iter().map(node_count).sum(),
        _ => 0,
    }
}
pub fn synthesize(input: &Value) -> Result<Value> {
    for key in [
        "test",
        "locked_test",
        "transfer_test",
        "locked_transfer_test",
    ] {
        if input.get(key).is_some() {
            return Err(invalid(
                "locked transfer data may not enter synthesis or model selection",
            ));
        }
    }
    let train = array(input, "train", 1000)?;
    let validation = array(input, "validation", 1000)?;
    if train.len() < 4 || validation.len() < 2 {
        return Err(invalid(
            "synthesis needs at least 4 training and 2 inner-validation cases",
        ));
    }
    let maximum = bounded_usize(input, "max_candidates", 256, 1024)?;
    let penalty = input
        .get("mdl_penalty")
        .and_then(Value::as_f64)
        .unwrap_or(0.001);
    if !(0.0..=1.0).contains(&penalty) {
        return Err(invalid("mdl_penalty must be in [0,1]"));
    }
    let mut ids = BTreeSet::new();
    let mut train_hashes = BTreeSet::new();
    let mut labels: BTreeMap<String, Value> = BTreeMap::new();
    let mut feature_values: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in train {
        let id = text(row, "id")?;
        if !ids.insert(id.to_owned()) {
            return Err(invalid("duplicate training case id"));
        }
        let x = row
            .get("x")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid("x must be a flat feature object"))?;
        if x.len() > 16 {
            return Err(ResearchError::Limit(
                "synthesis supports at most 16 features per case".into(),
            ));
        }
        train_hashes.insert(digest(&json!(x))?);
        let y = row
            .get("y")
            .ok_or_else(|| invalid("missing target label"))?;
        if !(y.is_boolean() || y.is_string() || y.is_number()) {
            return Err(invalid("target labels must be primitive"));
        }
        labels.insert(y.to_string(), y.clone());
        for (k, v) in x {
            if k.contains('.') || k.is_empty() {
                return Err(invalid("synthesis feature keys must be flat"));
            }
            if !(v.is_boolean() || v.is_string() || v.is_number()) {
                return Err(invalid("synthesis features must be primitive"));
            }
            feature_values.entry(k.clone()).or_default().push(v.clone());
        }
    }
    if labels.len() != 2 {
        return Err(invalid(
            "bounded synthesis currently supports binary classification only",
        ));
    }
    for row in validation {
        let id = text(row, "id")?;
        if !ids.insert(id.to_owned()) {
            return Err(invalid("case id overlap between training and validation"));
        }
        let x = row
            .get("x")
            .ok_or_else(|| invalid("validation x missing"))?;
        if train_hashes.contains(&digest(x)?) {
            return Err(invalid(
                "identical feature content leaks across training and validation",
            ));
        }
        let y = row
            .get("y")
            .ok_or_else(|| invalid("validation label missing"))?;
        if !labels.contains_key(&y.to_string()) {
            return Err(invalid("unseen validation target label"));
        }
    }
    let labels = labels.into_values().collect::<Vec<_>>();
    let mut conditions = Vec::new();
    for (key, values) in feature_values {
        if !train.iter().all(|row| row["x"].get(&key).is_some()) {
            continue;
        }
        if values.iter().all(Value::is_number) {
            let mut nums = values.iter().map(numeric).collect::<Result<Vec<_>>>()?;
            nums.sort_by(f64::total_cmp);
            nums.dedup();
            for pair in nums.windows(2).take(32) {
                let threshold = pair[0] / 2.0 + pair[1] / 2.0;
                conditions.push(predicate(&key, "le", json!(threshold)));
            }
        } else {
            let mut values = values;
            values.sort_by_key(Value::to_string);
            values.dedup();
            for value in values.into_iter().take(32) {
                conditions.push(predicate(&key, "eq", value));
            }
        }
    }
    let base = conditions.clone();
    for i in 0..base.len().min(16) {
        for j in i + 1..base.len().min(16) {
            if conditions.len() >= maximum / 2 {
                break;
            }
            conditions.push(json!({"op":"and","args":[base[i],base[j]]}));
        }
    }
    let possible_count = 2 * conditions.len() + 2;
    let mut programs = vec![
        json!({"op":"literal","value":labels[0]}),
        json!({"op":"literal","value":labels[1]}),
    ];
    for cond in conditions {
        if programs.len() >= maximum {
            break;
        }
        programs.push(classify(cond.clone(), &labels[0], &labels[1]));
        if programs.len() < maximum {
            programs.push(classify(cond, &labels[1], &labels[0]));
        }
    }
    let mut ranked = Vec::new();
    for program in programs {
        let train_error = score(&program, train)?;
        let validation_error = score(&program, validation)?;
        let nodes = node_count(&program);
        let objective = validation_error + penalty * nodes as f64;
        ranked.push((objective, validation_error, train_error, nodes, program));
    }
    ranked.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then(a.3.cmp(&b.3))
            .then(a.4.to_string().cmp(&b.4.to_string()))
    });
    let best = ranked
        .first()
        .ok_or_else(|| invalid("no valid synthesis candidate"))?
        .clone();
    let candidates=ranked.iter().take(10).map(|c|json!({"program":c.4,"train_error":c.2,"inner_validation_error":c.1,"nodes":c.3,"mdl_objective":c.0})).collect::<Vec<_>>();
    Ok(
        json!({"status":"research_candidate","best_program":best.4,"train_error":best.2,"inner_validation_error":best.1,"candidate_count":ranked.len(),"search_exhaustive_for_enumerated_grammar":possible_count<=maximum,"grammar":"constant or binary if of primitive threshold/equality; optional two-predicate conjunction","mdl":"inner-validation error + penalty * DSL node count; a structural proxy, not universal Kolmogorov complexity","top_candidates":candidates,"production_promoted":false,"locked_transfer_test_evaluated":false,"next_gate":"freeze best_program and evaluate once on a separately held locked transfer set"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_checks_dimensions_and_temperature() {
        assert!((convert(32.0, "F", "C").expect("conversion")).abs() < 1e-10);
        assert!(convert(10.0, "psi", "m").is_err());
        assert!(convert(-1.0, "K", "C").is_err());
    }
    #[test]
    fn filter_projection_and_join_execute_without_io() {
        let p = json!({"op":"filter","data":{"op":"input"},"predicate":{"op":"compare","operator":"gt","left":{"op":"field","path":"v"},"right":{"op":"literal","value":2}}});
        let r = execute(&json!({"program":p,"data":[{"v":1},{"v":3}]})).expect("execute");
        assert_eq!(r["result"], json!([{"v":3}]));
        let j = json!({"op":"join","left":{"op":"field","path":"a"},"right":{"op":"field","path":"b"},"left_key":"id","right_key":"id"});
        assert_eq!(
            execute(&json!({"program":j,"data":{"a":[{"id":1}],"b":[{"id":1.0},{"id":2}]}}))
                .expect("join")["result"]
                .as_array()
                .expect("rows")
                .len(),
            1
        );
    }
    #[test]
    fn joins_and_arbitrary_code_are_bounded() {
        assert!(
            execute(&json!({"program":{"op":"shell","command":"echo hacked"},"data":{}})).is_err()
        );
        let p = json!({"op":"join","left":{"op":"input"},"right":{"op":"input"},"left_key":"id","right_key":"id"});
        assert!(execute(&json!({"program":p,"data":[{"id":1},{"id":1}],"max_rows":2})).is_err());
    }
    #[test]
    fn induction_uses_inner_validation_and_rejects_transfer_access() {
        let input = json!({"train":[{"id":"1","x":{"hours":1},"y":true},{"id":"2","x":{"hours":2},"y":true},{"id":"3","x":{"hours":8},"y":false},{"id":"4","x":{"hours":9},"y":false}],"validation":[{"id":"5","x":{"hours":3},"y":true},{"id":"6","x":{"hours":7},"y":false}]});
        let r = synthesize(&input).expect("synthesize");
        assert_eq!(r["inner_validation_error"], 0.0);
        assert_eq!(
            execute(&json!({"program":r["best_program"],"data":{"hours":4}})).expect("execute")
                ["result"],
            true
        );
        let mut bad = input;
        bad["locked_test"] = json!([]);
        assert!(synthesize(&bad).is_err());
    }
    #[test]
    fn overlap_is_rejected() {
        let input = json!({"train":[{"id":"1","x":{"n":1},"y":true},{"id":"2","x":{"n":2},"y":true},{"id":"3","x":{"n":3},"y":false},{"id":"4","x":{"n":4},"y":false}],"validation":[{"id":"5","x":{"n":1},"y":true},{"id":"6","x":{"n":5},"y":false}]});
        assert!(synthesize(&input).is_err());
    }
}
