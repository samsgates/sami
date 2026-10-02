use crate::{array, bounded_usize, invalid, number, Result};
use serde_json::{json, Value};

fn binary_data(input: &Value) -> Result<(Vec<f64>, Vec<f64>)> {
    let probabilities = array(input, "predictions", 20_000)?;
    let labels = array(input, "labels", 20_000)?;
    if probabilities.is_empty() || probabilities.len() != labels.len() {
        return Err(invalid(
            "nonempty predictions and labels must have the same length",
        ));
    }
    let p = probabilities
        .iter()
        .map(|v| {
            v.as_f64()
                .filter(|p| (0.0..=1.0).contains(p))
                .ok_or_else(|| invalid("predictions must be finite probabilities"))
        })
        .collect::<Result<Vec<_>>>()?;
    let y = labels
        .iter()
        .map(|v| match v {
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => v
                .as_f64()
                .filter(|x| *x == 0.0 || *x == 1.0)
                .ok_or_else(|| invalid("labels must be binary")),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok((p, y))
}
fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}
fn logit(p: f64) -> f64 {
    let q = p.clamp(1e-9, 1.0 - 1e-9);
    (q / (1.0 - q)).ln()
}
fn alpha(input: &Value) -> Result<f64> {
    let a = input.get("alpha").and_then(Value::as_f64).unwrap_or(0.05);
    if !a.is_finite() || a <= 0.0 || a >= 1.0 {
        return Err(invalid("alpha must be strictly between 0 and 1"));
    }
    Ok(a)
}

pub fn fit(input: &Value) -> Result<Value> {
    if input.get("split").and_then(Value::as_str) != Some("calibration") {
        return Err(invalid(
            "fit requires split=calibration; locked evaluation data may not be fitted",
        ));
    }
    let (p, y) = binary_data(input)?;
    if !y.contains(&0.0) || !y.contains(&1.0) {
        return Err(invalid("calibration requires both outcome classes"));
    }
    let method = input
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("platt");
    match method {
        "platt" | "logistic" => {
            let iterations = bounded_usize(input, "iterations", 400, 5000)?;
            let learning_rate = input
                .get("learning_rate")
                .and_then(Value::as_f64)
                .unwrap_or(0.05);
            let ridge = input.get("ridge").and_then(Value::as_f64).unwrap_or(0.001);
            if !(0.0..=1.0).contains(&learning_rate)
                || learning_rate == 0.0
                || !(0.0..=10.0).contains(&ridge)
            {
                return Err(invalid("invalid learning_rate or ridge"));
            }
            let x = p
                .iter()
                .map(|q| if method == "platt" { logit(*q) } else { *q })
                .collect::<Vec<_>>();
            let mean = x.iter().sum::<f64>() / x.len() as f64;
            let sd = (x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / x.len() as f64)
                .sqrt()
                .max(1e-8);
            let mut a = 0.0;
            let mut b = logit(y.iter().sum::<f64>() / y.len() as f64);
            for _ in 0..iterations {
                let mut ga = 0.0;
                let mut gb = 0.0;
                for (&xi, &yi) in x.iter().zip(&y) {
                    let z = (xi - mean) / sd;
                    let err = sigmoid(a * z + b) - yi;
                    ga += err * z;
                    gb += err;
                }
                ga = ga / x.len() as f64 + ridge * a;
                gb /= x.len() as f64;
                a -= learning_rate * ga;
                b -= learning_rate * gb;
            }
            Ok(
                json!({"calibrator":{"method":method,"a":a/sd,"b":b-a*mean/sd,"input":"probability"},"sample_count":p.len(),"split":"calibration","production_ready":false}),
            )
        }
        "isotonic" => {
            let mut pairs = p.iter().copied().zip(y.iter().copied()).collect::<Vec<_>>();
            pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
            // Collapse equal scores before PAV: a function cannot give two values at the same x.
            let mut blocks: Vec<(f64, f64, f64, usize)> = Vec::new();
            for (x, y) in pairs {
                if let Some(last) = blocks.last_mut() {
                    if last.1 == x {
                        last.2 += y;
                        last.3 += 1;
                        continue;
                    }
                }
                blocks.push((x, x, y, 1));
            }
            let mut pooled: Vec<(f64, f64, f64, usize)> = Vec::new();
            for block in blocks {
                pooled.push(block);
                while pooled.len() > 1 {
                    let n = pooled.len();
                    if pooled[n - 2].2 / pooled[n - 2].3 as f64
                        <= pooled[n - 1].2 / pooled[n - 1].3 as f64
                    {
                        break;
                    }
                    let right = pooled
                        .pop()
                        .ok_or_else(|| invalid("invalid isotonic state"))?;
                    let left = pooled
                        .pop()
                        .ok_or_else(|| invalid("invalid isotonic state"))?;
                    pooled.push((left.0, right.1, left.2 + right.2, left.3 + right.3));
                }
            }
            let intervals = pooled
                .iter()
                .map(|b| json!({"min":b.0,"max":b.1,"value":b.2/b.3 as f64,"count":b.3}))
                .collect::<Vec<_>>();
            Ok(
                json!({"calibrator":{"method":"isotonic","intervals":intervals},"sample_count":p.len(),"split":"calibration","production_ready":false}),
            )
        }
        _ => Err(invalid("method must be platt, logistic or isotonic")),
    }
}

pub fn predict(input: &Value) -> Result<Value> {
    let c = input
        .get("calibrator")
        .ok_or_else(|| invalid("calibrator required"))?;
    let values = array(input, "predictions", 20_000)?;
    let method = c
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("calibrator.method required"))?;
    let mut output = Vec::new();
    let mut intervals = Vec::new();
    if method == "isotonic" {
        let blocks = array(c, "intervals", 20_000)?;
        if blocks.is_empty() {
            return Err(invalid("isotonic intervals cannot be empty"));
        }
        let mut prev_max = f64::NEG_INFINITY;
        let mut prev_y = 0.0;
        for block in blocks {
            let lo = number(block, "min")?;
            let hi = number(block, "max")?;
            let y = number(block, "value")?;
            if lo > hi
                || lo <= prev_max
                || !(0.0..=1.0).contains(&lo)
                || !(0.0..=1.0).contains(&hi)
                || !(0.0..=1.0).contains(&y)
                || y < prev_y
            {
                return Err(invalid("invalid monotone isotonic intervals"));
            }
            intervals.push((lo, hi, y));
            prev_max = hi;
            prev_y = y;
        }
    }
    for value in values {
        let p = value
            .as_f64()
            .filter(|p| (0.0..=1.0).contains(p))
            .ok_or_else(|| invalid("predictions must be probabilities"))?;
        let calibrated = match method {
            "platt" => sigmoid(number(c, "a")? * logit(p) + number(c, "b")?),
            "logistic" => sigmoid(number(c, "a")? * p + number(c, "b")?),
            "isotonic" => {
                // Piecewise constant extension in each training interval; linear interpolation in gaps.
                let mut value = intervals
                    .last()
                    .map(|b| b.2)
                    .ok_or_else(|| invalid("missing intervals"))?;
                if p <= intervals[0].1 {
                    value = intervals[0].2;
                } else {
                    for i in 1..intervals.len() {
                        if p <= intervals[i].1 {
                            let l = intervals[i - 1];
                            let r = intervals[i];
                            value = if p >= r.0 {
                                r.2
                            } else {
                                l.2 + (r.2 - l.2) * (p - l.1) / (r.0 - l.1)
                            };
                            break;
                        }
                    }
                }
                value
            }
            _ => return Err(invalid("unknown calibrator method")),
        };
        output.push(calibrated);
    }
    Ok(json!({"predictions":output,"calibration_is_a_group_property":true}))
}

fn wilson(errors: usize, n: usize) -> Option<(f64, f64)> {
    if n == 0 {
        return None;
    }
    let z: f64 = 1.959963984540054;
    let nf = n as f64;
    let p = errors as f64 / nf;
    let center = (p + z * z / (2.0 * nf)) / (1.0 + z * z / nf);
    let radius = z * (p * (1.0 - p) / nf + z * z / (4.0 * nf * nf)).sqrt() / (1.0 + z * z / nf);
    Some(((center - radius).max(0.0), (center + radius).min(1.0)))
}

pub fn metrics(input: &Value) -> Result<Value> {
    let (p, y) = binary_data(input)?;
    let bins = bounded_usize(input, "bins", 10, 100)?;
    let a = alpha(input)?;
    let n = p.len();
    let nf = n as f64;
    let mut brier = 0.0;
    let mut log_loss = 0.0;
    let mut correct = 0usize;
    let mut totals = vec![(0usize, 0.0, 0.0); bins];
    for (&p, &y) in p.iter().zip(&y) {
        brier += (p - y).powi(2);
        let q = p.clamp(1e-15, 1.0 - 1e-15);
        log_loss -= y * q.ln() + (1.0 - y) * (1.0 - q).ln();
        correct += usize::from((p >= 0.5) == (y == 1.0));
        let k = ((p * bins as f64) as usize).min(bins - 1);
        totals[k].0 += 1;
        totals[k].1 += p;
        totals[k].2 += y;
    }
    let mut ece = 0.0;
    let mut mce: f64 = 0.0;
    let mut calibration_bins = Vec::new();
    for (index, &(count, ps, ys)) in totals.iter().enumerate() {
        let (mean_p, rate, gap) = if count > 0 {
            (
                Some(ps / count as f64),
                Some(ys / count as f64),
                (ps - ys).abs() / count as f64,
            )
        } else {
            (None, None, 0.0)
        };
        ece += count as f64 / nf * gap;
        mce = mce.max(gap);
        calibration_bins
            .push(json!({"index":index,"n":count,"mean_probability":mean_p,"positive_rate":rate}));
    }
    let thresholds = match input.get("thresholds") {
        Some(v) => v
            .as_array()
            .filter(|v| v.len() <= 100)
            .ok_or_else(|| invalid("thresholds must be an array of at most 100 entries"))?
            .clone(),
        None => vec![json!(0.5), json!(0.7), json!(0.9), json!(0.95), json!(0.99)],
    };
    let mut selective = Vec::new();
    for t in thresholds {
        let t = t
            .as_f64()
            .filter(|t| (0.5..=1.0).contains(t))
            .ok_or_else(|| invalid("selective thresholds must be in [0.5,1]"))?;
        let mut selected = 0;
        let mut errors = 0;
        for (&p, &y) in p.iter().zip(&y) {
            if p.max(1.0 - p) >= t {
                selected += 1;
                errors += usize::from((p >= 0.5) != (y == 1.0));
            }
        }
        let ci = wilson(errors, selected).map(|(l, h)| json!([l, h]));
        let zero_upper = if errors == 0 && selected > 0 {
            Some(1.0 - a.powf(1.0 / selected as f64))
        } else {
            None
        };
        selective.push(json!({"threshold":t,"selected":selected,"errors":errors,"coverage":selected as f64/nf,"risk":if selected>0{Some(errors as f64/selected as f64)}else{None},"risk_wilson_95":ci,"zero_error_one_sided_upper":zero_upper,"zero_error_alpha":a}));
    }
    Ok(
        json!({"sample_count":n,"accuracy":correct as f64/nf,"brier":brier/nf,"ece":ece,"mce":mce,"log_loss":log_loss/nf,"calibration_bins":calibration_bins,"selective_risk":selective,"interval_assumption":"independent Bernoulli observations; clustered cases require a cluster-aware analysis","production_gate_passed":false}),
    )
}

pub fn drift(input: &Value) -> Result<Value> {
    let bins = bounded_usize(input, "bins", 10, 100)?;
    let read = |key: &str| -> Result<Vec<f64>> {
        let v = array(input, key, 20_000)?;
        if v.is_empty() {
            return Err(invalid("drift samples must be nonempty"));
        }
        v.iter()
            .map(|x| {
                x.as_f64()
                    .filter(|x| (0.0..=1.0).contains(x))
                    .ok_or_else(|| invalid("drift scores must be in [0,1]"))
            })
            .collect()
    };
    let r = read("reference")?;
    let c = read("current")?;
    let hist = |xs: &[f64]| {
        let mut h = vec![0.5; bins];
        for &x in xs {
            h[((x * bins as f64) as usize).min(bins - 1)] += 1.0;
        }
        let sum = h.iter().sum::<f64>();
        h.iter_mut().for_each(|x| *x /= sum);
        h
    };
    let ph = hist(&r);
    let qh = hist(&c);
    let mut js = 0.0;
    let mut psi = 0.0;
    for (&p, &q) in ph.iter().zip(&qh) {
        let m = (p + q) / 2.0;
        js += 0.5 * p * (p / m).ln() + 0.5 * q * (q / m).ln();
        psi += (q - p) * (q / p).ln();
    }
    let mut r = r;
    let mut c = c;
    r.sort_by(f64::total_cmp);
    c.sort_by(f64::total_cmp);
    let mut i = 0;
    let mut j = 0;
    let mut ks: f64 = 0.0;
    while i < r.len() || j < c.len() {
        let x = match (r.get(i), c.get(j)) {
            (Some(a), Some(b)) => a.min(*b),
            (Some(a), None) => *a,
            (None, Some(b)) => *b,
            (None, None) => break,
        };
        while i < r.len() && r[i] <= x {
            i += 1;
        }
        while j < c.len() && c[j] <= x {
            j += 1;
        }
        ks = ks.max((i as f64 / r.len() as f64 - j as f64 / c.len() as f64).abs());
    }
    let threshold = input
        .get("js_threshold")
        .and_then(Value::as_f64)
        .unwrap_or(0.1);
    if !(0.0..=std::f64::consts::LN_2).contains(&threshold) {
        return Err(invalid("js_threshold must be in [0,ln(2)]"));
    }
    Ok(
        json!({"jensen_shannon_nats":js,"population_stability_index":psi,"ks_distance":ks,"reference_histogram":ph,"current_histogram":qh,"alert":js>threshold,"threshold":threshold,"interpretation":"descriptive score shift; does not measure accuracy or prove label drift","automatic_policy_change":false}),
    )
}

pub fn conformal_fit(input: &Value) -> Result<Value> {
    if input.get("split").and_then(Value::as_str) != Some("calibration") {
        return Err(invalid("conformal fit requires split=calibration"));
    }
    let (p, y) = binary_data(input)?;
    let a = alpha(input)?;
    let mut scores = p
        .iter()
        .zip(&y)
        .map(|(&p, &y)| if y == 1.0 { 1.0 - p } else { p })
        .collect::<Vec<_>>();
    scores.sort_by(f64::total_cmp);
    let rank = ((scores.len() + 1) as f64 * (1.0 - a)).ceil() as usize;
    let q = if rank > scores.len() {
        1.0
    } else {
        scores[rank.saturating_sub(1)]
    };
    Ok(
        json!({"conformal":{"method":"binary_split_conformal","quantile":q,"alpha":a,"calibration_count":scores.len()},"assumption":"calibration and future cases must be exchangeable; drift and adaptive selection can invalidate marginal coverage","conditional_coverage_guaranteed":false}),
    )
}
pub fn conformal_predict(input: &Value) -> Result<Value> {
    let model = input
        .get("conformal")
        .ok_or_else(|| invalid("conformal model required"))?;
    if model.get("method").and_then(Value::as_str) != Some("binary_split_conformal") {
        return Err(invalid("unsupported conformal method"));
    }
    let q = number(model, "quantile")?;
    if !(0.0..=1.0).contains(&q) {
        return Err(invalid("invalid conformal quantile"));
    }
    let mut sets = Vec::new();
    for p in array(input, "predictions", 20_000)? {
        let p = p
            .as_f64()
            .filter(|p| (0.0..=1.0).contains(p))
            .ok_or_else(|| invalid("predictions must be probabilities"))?;
        let mut set = Vec::new();
        if p <= q {
            set.push(false);
        }
        if 1.0 - p <= q {
            set.push(true);
        }
        sets.push(json!({"labels":set,"status":match set.len(){0=>"empty_set",1=>"singleton",_=>"ambiguous"}}));
    }
    Ok(json!({"prediction_sets":sets,"automatic_action_allowed":false}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_metrics_and_zero_failure_bound() {
        let r = metrics(&json!({"predictions":[0.1,0.9],"labels":[0,1],"thresholds":[0.9]}))
            .expect("metrics");
        assert!((r["brier"].as_f64().expect("brier") - 0.01).abs() < 1e-12);
        assert_eq!(r["accuracy"], 1.0);
        assert!(
            r["selective_risk"][0]["zero_error_one_sided_upper"]
                .as_f64()
                .expect("upper")
                > 0.7
        );
    }
    #[test]
    fn pav_merges_violations_and_equal_scores() {
        let r=fit(&json!({"method":"isotonic","split":"calibration","predictions":[0.1,0.1,0.4,0.9],"labels":[1,0,0,1]})).expect("fit");
        let p = predict(&json!({"calibrator":r["calibrator"],"predictions":[0.1,0.4,0.9]}))
            .expect("predict");
        assert_eq!(p["predictions"][0], p["predictions"][1]);
        assert!(p["predictions"][2].as_f64().expect("p") > 0.9);
    }
    #[test]
    fn locked_test_cannot_be_fitted() {
        assert!(fit(
            &json!({"split":"locked_transfer_test","predictions":[0.1,0.9],"labels":[0,1]})
        )
        .is_err());
    }
    #[test]
    fn conformal_small_sample_does_not_overclaim() {
        let r = conformal_fit(
            &json!({"split":"calibration","predictions":[0.1,0.9],"labels":[0,1],"alpha":0.05}),
        )
        .expect("fit");
        assert_eq!(r["conformal"]["quantile"], 1.0);
        let p = conformal_predict(&json!({"conformal":r["conformal"],"predictions":[0.99]}))
            .expect("predict");
        assert_eq!(
            p["prediction_sets"][0]["labels"]
                .as_array()
                .expect("set")
                .len(),
            2
        );
    }
    #[test]
    fn drift_detects_shift_and_identical_samples_are_zero() {
        let same = drift(&json!({"reference":[0.1,0.2],"current":[0.1,0.2]})).expect("drift");
        assert_eq!(same["ks_distance"], 0.0);
        let shift =
            drift(&json!({"reference":[0.0,0.0,0.0],"current":[1.0,1.0,1.0]})).expect("drift");
        assert_eq!(shift["ks_distance"], 1.0);
    }
}
