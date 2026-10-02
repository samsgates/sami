//! Durable, version-pinned human-confirmed workflows. Planning never executes tools.
use crate::{
    auth::Principal,
    error::{Error, Result},
    service::Service,
    storage::Write,
};
use serde_json::{json, Value};
fn wr(id: &str, v: Value, expected: i64) -> Write {
    Write {
        kind: "workflow_runs".into(),
        id: id.into(),
        payload: v,
        expected: Some(expected),
        immutable: false,
    }
}
fn set_path(v: &mut Value, path: &str, value: Value) -> Result<()> {
    let mut cur = v;
    let mut parts = path.split('.').peekable();
    while let Some(part) = parts.next() {
        if part.is_empty()
            || ["tenant", "permission", "policy", "tool"]
                .iter()
                .any(|p| path.starts_with(p))
        {
            return Err(Error::bad("Workflow effects cannot grant authority."));
        }
        let obj = cur
            .as_object_mut()
            .ok_or_else(|| Error::bad("Effect crosses a non-object."))?;
        if parts.peek().is_none() {
            obj.insert(part.into(), value);
            return Ok(());
        }
        cur = obj.entry(part).or_insert_with(|| json!({}));
    }
    Ok(())
}
impl Service {
    pub async fn start_workflow(&self, p: &Principal, id: &str, input: Value) -> Result<Value> {
        p.require("workflow.execute")?;
        let workflow = self.store.get(&p.tenant, "workflows", id).await?;
        self.authorize_record(p, &workflow.payload, "service_prequalification")
            .await?;
        if workflow.payload["status"] != "approved" {
            return Err(Error::forbidden("Workflow must be approved."));
        }
        let state = input.get("state").cloned().unwrap_or_else(
            || json!({"asset_confirmed":false,"evidence_complete":false,"case":{"status":"new"}}),
        );
        if !state.is_object() {
            return Err(Error::bad("Workflow state must be an object."));
        }
        let goal = input["goal"]
            .as_str()
            .or_else(|| {
                workflow.payload["goals"]
                    .as_object()
                    .and_then(|g| g.keys().next().map(String::as_str))
            })
            .unwrap_or("complete");
        let plan = if workflow.payload["actions"].is_array() {
            sami_intelligence::plan(goal, &state, &workflow.payload, 2000)
                .map_err(|e| Error::bad(e.to_string()))?
        } else {
            json!({"status":"state_machine","steps":[]})
        };
        if plan["status"] == "budget_exhausted" {
            return Err(Error::unavailable(
                "Workflow planner budget exhausted; review the registered process.",
            ));
        }
        if plan["status"] == "no_plan" {
            return Err(Error::bad("No registered plan reaches this process goal."));
        }
        let initial_status =
            if plan["status"] == "planned" && plan["steps"].as_array().is_some_and(Vec::is_empty) {
                "completed"
            } else {
                "awaiting_confirmation"
            };
        let run_id = uuid::Uuid::new_v4().to_string();
        let step = workflow.payload["initial"].as_str().unwrap_or("pending");
        let v = json!({"workflow_id":id,"workflow_revision":workflow.revision,"definition":workflow.payload,"status":initial_status,"step":step,"step_index":0,"state":state,"plan":plan,"goal":goal,"actor_ref":crate::crypto::digest(p.subject.as_bytes()),"started_at":chrono::Utc::now().to_rfc3339(),"history":[],"external_effects":false});
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "workflow.start",
                vec![wr(&run_id, v, 0)],
                None,
            )
            .await?;
        Ok(self
            .store
            .get(&p.tenant, "workflow_runs", &run_id)
            .await?
            .payload)
    }
    pub async fn advance_workflow(&self, p: &Principal, id: &str, input: Value) -> Result<Value> {
        p.require("workflow.execute")?;
        let run = self.store.get(&p.tenant, "workflow_runs", id).await?;
        let mut v = run.payload;
        if ["cancelled", "completed", "failed"].contains(&v["status"].as_str().unwrap_or("")) {
            return Err(Error::conflict("Workflow is terminal."));
        }
        if input["revision"].as_i64() != Some(run.revision) {
            return Err(Error::conflict(
                "Advancement requires the current run revision.",
            ));
        }
        let reason = input["reason"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() <= 4000)
            .ok_or_else(|| Error::bad("Confirmation reason is required."))?;
        if let Some(wait) = input["wait_seconds"].as_u64() {
            if !(1..=86400).contains(&wait) {
                return Err(Error::bad("Timer must be 1..86400 seconds."));
            }
            v["status"] = json!("waiting_timer");
            v["wake_at"] = json!(chrono::Utc::now().timestamp() + wait as i64);
            v["timer_reason"] = json!(reason);
            self.store
                .commit(
                    &p.tenant,
                    &p.subject,
                    "workflow.timer_scheduled",
                    vec![wr(id, v, run.revision)],
                    None,
                )
                .await?;
            return Ok(self
                .store
                .get(&p.tenant, "workflow_runs", id)
                .await?
                .payload);
        }
        if v["status"] == "waiting_timer"
            && v["wake_at"].as_i64().unwrap_or(i64::MAX) > chrono::Utc::now().timestamp()
        {
            return Err(Error::conflict("Workflow timer is not due."));
        }
        if input["report_failure"] == true {
            let retry = v["retry_count"].as_u64().unwrap_or(0) + 1;
            v["retry_count"] = json!(retry);
            v["status"] = json!(if retry > 3 {
                "failed"
            } else {
                "awaiting_confirmation"
            });
            v["failure_reason"] = json!(reason);
            self.store
                .commit(
                    &p.tenant,
                    &p.subject,
                    "workflow.failure_recorded",
                    vec![wr(id, v, run.revision)],
                    None,
                )
                .await?;
            return Ok(self
                .store
                .get(&p.tenant, "workflow_runs", id)
                .await?
                .payload);
        }
        let live = self
            .store
            .get(
                &p.tenant,
                "workflows",
                v["workflow_id"].as_str().unwrap_or(""),
            )
            .await?;
        if live.revision != v["workflow_revision"].as_i64().unwrap_or(-1)
            || live.payload["status"] != "approved"
        {
            return Err(Error::conflict(
                "Workflow authority changed; start a new run.",
            ));
        }
        let def = v["definition"].clone();
        let index = v["step_index"].as_u64().unwrap_or(0) as usize;
        if def["actions"].is_array() {
            let step = v["plan"]["steps"]
                .as_array()
                .and_then(|a| a.get(index))
                .cloned()
                .ok_or_else(|| Error::bad("No valid planned step remains."))?;
            if input["confirm_step"] != step["id"] {
                return Err(Error::bad(
                    "confirm_step must name the exact pending planned step.",
                ));
            }
            if step.get("tool").is_some_and(|x| !x.is_null()) {
                return Err(Error::bad(
                    "Tools must use the action proposal/approval/execution lifecycle.",
                ));
            }
            if let Some(effects) = step["effects"].as_object() {
                for (k, value) in effects {
                    set_path(&mut v["state"], k, value.clone())?;
                }
            }
            v["step_index"] = json!(index + 1);
            v["status"] = json!(
                if index + 1 >= v["plan"]["steps"].as_array().map_or(0, Vec::len) {
                    "completed"
                } else {
                    "awaiting_confirmation"
                }
            );
        } else {
            let current = v["step"].as_str().unwrap_or("");
            let definition = &def["states"][current];
            let event = input["event"]
                .as_str()
                .ok_or_else(|| Error::bad("event required for a state machine."))?;
            let transition = definition["transitions"][event].clone();
            let next = transition
                .as_str()
                .or_else(|| transition["next"].as_str())
                .ok_or_else(|| Error::bad("This event is not a registered transition."))?;
            if def["states"].get(next).is_none() {
                return Err(Error::bad("Transition destination is unregistered."));
            }
            if let Some(pre) = transition.get("when") {
                let check = sami_intelligence::evaluate_rules(
                    &json!([{"id":"transition","when":pre,"then":true}]),
                    &v["state"],
                    1000,
                )
                .map_err(|e| Error::bad(e.to_string()))?;
                if check["matched"].as_array().is_none_or(|a| a.is_empty()) {
                    return Err(Error::forbidden(
                        "Transition precondition is not satisfied.",
                    ));
                }
            }
            if transition.get("tool").is_some() {
                return Err(Error::bad("Use the approved action lifecycle for effects."));
            }
            v["step"] = json!(next);
            v["step_index"] = json!(index + 1);
            v["status"] = json!(if def["states"][next]["terminal"] == true {
                "completed"
            } else {
                "awaiting_confirmation"
            });
        }
        let entry = json!({"step_index":index,"reason":reason,"actor_ref":crate::crypto::digest(p.subject.as_bytes()),"at":chrono::Utc::now().to_rfc3339(),"confirmed_assertion":true});
        let history = v["history"].as_array_mut().ok_or_else(Error::internal)?;
        if history.len() >= 500 {
            return Err(Error::bad("Workflow step budget exceeded."));
        }
        history.push(entry);
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "workflow.advance",
                vec![wr(id, v, run.revision)],
                None,
            )
            .await?;
        Ok(self
            .store
            .get(&p.tenant, "workflow_runs", id)
            .await?
            .payload)
    }
}
