# SAMI
Structured Adaptive Memory Intelligence

## Corrected product requirements and breakthrough strategy
Version 2.0 | 2 October 2026 | Proposed specification

Memory that can be corrected. Decisions that can be checked. Experience that can become reusable capability.

This document replaces the supplied 35-page v1.0 PRD as the proposed product and technical specification. It incorporates a critical review of all 154 original sections and current primary-source market research. It specifies a product to build; it does not describe an implemented or validated system.

The ambition is to establish a new standard for reliable operational intelligence. The initial proof is deliberately concrete: outperform strong competing systems on changing knowledge, constrained decisions, repeatability, offline operation, and total cost at matched quality.

Prepared for the SAMI founder and founding product, engineering, and research team. Research checked on 2 October 2026. Performance, staffing, budget, and commercial figures are planning assumptions unless explicitly attributed to a source.

[[PAGE]]
# Reading guide
This specification has four uses: assess the breakthrough thesis, plan a focused product, implement precise technical contracts, and decide whether release/research evidence is strong enough.

**Founder and product team:** start with 01-07 and 28-32. **Engineering team:** use 08-27, 33 and 35. **Research team:** focus on 04, 15-17, 20-21 and 28-29. **Reviewers:** compare the corrections and original traceability in 02 and 34, then inspect the source register.

[[CONTENTS]]

**Interpretation:** P0 is the paid production requirement; P1 is a later product feature; R is gated research. Words such as "must" and "required" describe proposed acceptance behavior. Targets and illustrative figures remain unmeasured. Links [S01]-[S19] lead to the source register.

**Status:** proposed v2.0 replacement. Customer access, engineering feasibility, market advantage and broad intelligence claims still require validation.

[[PAGE]]
# 01 | Executive decision
## Build an intelligence runtime with accountable memory
SAMI should become a CPU-first, memory-native runtime for operational decisions and controlled actions. Its core asset is a versioned evidence system that connects observations, claims, rules, decisions, procedures, and outcomes. Every consequential recommendation has inspectable dependencies; every action has an explicit authorization contract.

The strongest idea in v1 is the separation of memory, state, decisions, language, and execution. Preserve it. The weakest idea is the assumption that assembling retrieval, graphs, rules, and phrase statistics produces general intelligence. Replace that assumption with staged experiments on generalization and measurable customer outcomes.

> Breakthrough hypothesis: a system that repairs its own dependent knowledge after corrections and safely consolidates successful experience into reusable typed procedures can become more reliable and cheaper for bounded operational work than a frontier LLM agent with a strong memory and workflow stack.

## Three product layers
- **SAMI Core, P0:** temporal evidence memory, typed decisions, executable policy, abstention, replay, controlled workflow execution, native domain language, and tenant isolation.
- **SAMI Operations, P0/P1:** an industrial service desk application, administrator workbench, source connectors, offline deployment, evaluation, and correction workflows.
- **SAMI Research, gated:** typed procedure discovery, compositional transfer, information-seeking decisions, and experimentally grounded process models. Research features never become mandatory production dependencies.

## First customer and value
Launch with industrial-equipment OEMs and distributors whose service desks repeatedly reconcile serial numbers, service bulletins, manuals, entitlement records, and warranty policies. V1 recommends triage routes and warranty prequalification; humans retain final coverage and financial authority. Offline usefulness and temporal correctness create a plausible entry point.

## What success would justify
First prove reliable deployment and paid customer value. Then prove transfer across equipment families with less rule-authoring effort. Only then expand to other workflows and compete for the operational intelligence layer across models and applications. The PRD does not assert AGI, worldwide novelty, universal LLM superiority, or zero hallucinations.

[[PAGE]]
# 02 | What the original PRD gets right and wrong
| Original strength | Preserve and strengthen |
| --- | --- |
| Explicit state and typed decisions | Add versioned hypotheses, source dependencies, and decision contracts. |
| LLM-free core and CPU deployment | Preserve a fully disconnected native mode; define its bounded capabilities. |
| Graphs, rules, evidence, calibration | Specify truth maintenance, rule semantics, and measured selective error. |
| Learning quarantine and rollback | Add outcome attribution, cohort checks, immutable promotions, and incident response. |
| Tool authorization and durable workflows | Add stale-authority rejection, reconcile-first retries, and effect receipts. |

## Corrections that change the product
**Scope:** v1 tries to launch a conversational platform, agent framework, decision engine, roleplay product, and research architecture together. V2 gives P0 one customer, two workflows, explicit exclusions, and release gates.

**Scientific claims:** an n-gram continuation engine is still a language model. Sequential next-token composition can be autoregressive, even without a Transformer. Memory lookup is not a demonstrated replacement for abstraction, general reasoning, or creative generation. Infini-gram supports large-scale sequence queries; it does not establish these broader claims. [S01]

**Confidence:** a softmax score, source count, or low entropy is not proof of correctness. V2 separates calibrated class probabilities, measured error by cohort, evidence diagnostics, and explicit policy decisions. Unsupported scores are null rather than invented.

**Knowledge:** source provenance does not make a claim true. Duplicated documents are correlated evidence. A source correction must invalidate dependent conclusions and queued actions; v1 does not fully specify this.

**Execution:** a last-stage text policy filter cannot substitute for policy at ingestion, retrieval, planning, and commit. Queue delivery and external effects are not universally exactly once.

**Operations:** CPU-first does not mean cheap. Index construction, RAM, NVMe, replication, deletion, compaction, and human ontology maintenance belong in cost and performance evaluation.

**Delivery:** v1's production label is a specification status, not evidence of production readiness. Its embedded coding instructions are replaced here with release-specific obligations. The user's requested deliverable is this corrected PRD, not a software implementation.

[[PAGE]]
# 03 | Competition and the opportunity
Research uses public primary documentation. A listed capability is public product positioning, not an independent quality assessment. Absence from a reviewed page means unverified, not absent from the product.

| System / category | Existing overlap | SAMI must prove |
| --- | --- | --- |
| Letta | Stateful agents and persistent agent memory. [S02] | Native non-LLM decision support and correction-to-action consistency. |
| Mem0 | Memory infrastructure, self-hosted options, and configurable components. [S03] | End-to-end decision, repair, and authorized-effect value. |
| Zep / Graphiti | Temporal knowledge graphs, provenance and hybrid retrieval. [S04] | Better dependent-decision repair and bounded operational outcomes. |
| TypeSafe / Jev | Typed decision outputs and calibration. [S05] | Full memory lifecycle, workflow state, and evidence-grounded replay. |
| Rasa / Dialogflow CX | Separation of interpretation, flows, and response behavior. [S06, S07] | Stronger temporal evidence repair and practical offline coverage. |
| Palantir Ontology / AIP | Operational objects, logic and actions linked to enterprise data. [S08] | Faster, smaller, self-hostable deployment for a focused customer. |
| Temporal + OPA | Durable execution and explicit API authorization. [S09, S10] | Integrated intelligence above those existing capabilities. |

Recent research also studies pre-action authorization, commit-time freshness, and semantic isolation across durable agent workflows. Receipts and commit guards cannot be claimed as independently unprecedented. [S11, S12, S13]

## Differentiation is a falsifiable combination
SAMI proposes one coherent contract: a decision depends on named evidence and policy versions; corrections revoke affected eligibility; replay explains the change; effects require fresh authority; learned procedures remain bounded and reviewable. The market opportunity is making this affordable and usable for a specific workflow while preserving native offline operation.

The moat must come from implementation quality, domain workflow packages, customer-permitted evaluation data, verified procedure libraries, and deployment economics. Generic memory, graphs, rules, CPU execution, or an API-compatible endpoint are insufficient moats. This review is not an exhaustive market or patent search.

[[PAGE]]
# 04 | Meaning of "more powerful than an LLM"
Compare complete systems, not a bare model against a fully engineered SAMI installation. An LLM can be combined with databases, schemas, tools, policies, and workflows. The original architecture table understated those possibilities.

| Dimension | Proposed SAMI advantage | Boundary |
| --- | --- | --- |
| Changing knowledge | Correct once; invalidate and recompute dependent decisions. | Depends on complete lineage and correct source adapters. |
| Operational correctness | Typed outputs with executable eligibility checks. | Checks prove constraints relative to inputs, not world truth. |
| Repeatability | Replay a pinned decision snapshot with identical structured results. | Live external systems and optional language models may vary. |
| Offline operation | Approved workflows run without remote language APIs. | Freshness and revocation limits can block actions. |
| Adaptation | Update memory and approved procedures without full neural retraining. | Novel language and abstraction remain difficult. |
| Efficiency | Bounded CPU inference and amortized procedure execution. | Total cost includes ingestion, review, storage and build. |
| Open-ended reasoning | Research transfer through reusable programs. | No demonstrated general advantage over frontier models. |

## Hypotheses and evidence required
**H1, memory repair:** achieve 100% dependent-authority invalidation in generated graph tests and lower stale-decision error than the strongest baseline on held-out operational cases.

**H2, reliable decisions:** at matched case coverage, reduce materially wrong prequalification or triage recommendations. Report abstention, workload moved to humans, and subgroup performance alongside accuracy.

**H3, economics:** reduce total cost per policy-compliant completed case by at least 30% against the strongest deployable baseline at comparable quality. This is a research/commercial target, not a forecast.

**H4, capability accumulation:** learned typed procedures reduce authored rules or examples needed for a held-out equipment family by at least 30%, without worsening risk. Failure leaves SAMI a useful operational product but does not establish a new learning paradigm.

General superhuman intelligence, autonomous scientific discovery, and replacement of frontier coding models are outside v2 commitments. The long-term ambition remains broad; the evidence must earn each expansion.

[[PAGE]]
# 05 | Customer, jobs, and boundaries
## Initial customer profile - a hypothesis to validate
An OEM or distributor service organization with 10-100 support specialists, at least several thousand service cases per month, licensed access to manuals and service bulletins, and structured warranty/asset data. Start with one product family and one policy jurisdiction. These sizes are planning filters, not market estimates.

**Economic buyer:** head of service operations. **Champion:** service knowledge manager. **Daily user:** service desk specialist. **Control owner:** warranty policy manager. **Technical buyer:** IT/security lead. **Deployment partner:** dealer-system administrator.

## Two P0 jobs
- **Service triage:** identify the asset, match the applicable bulletin/manual revision, classify the issue, request missing information, and recommend the approved support queue. Create an internal ticket only with the required user authority.
- **Warranty prequalification:** assemble eligibility evidence using date, operating hours, serial scope, policy version, and required documentation. Return potentially eligible, potentially ineligible, or review required, with the responsible human and supporting facts.

## Explicit boundaries
V1 does not diagnose mechanical faults beyond approved categorical routing, instruct hazardous repairs, actuate equipment, authorize financial payouts, order parts autonomously, or determine legal entitlement. Missing or contradictory evidence routes to an authorized reviewer. Native English is the first supported language; other languages require separate evaluation and language packs.

The system is a decision-support and workflow product. An operator can use a structured form when free-text interpretation is unsupported. The offline package is for approved knowledge work; connected actions require current authority and connector capabilities.

## Discovery before implementation commitment
Interview 15-20 service leaders and specialists. Obtain five consented design partners, with at least three providing representative de-identified cases and source revisions. Validate error costs, integration complexity, manual-review burden, willingness to pay, and whether an existing rule engine solves the problem cheaply.

If customers cannot supply licensed authoritative data, or if access/onboarding dominates expected value, narrow the workflow or select a different vertical. Do not build a generic AI platform first and search for a customer later.

[[PAGE]]
# 06 | User journeys and experience
## Journey A - specialist handles an ambiguous case
1. Specialist opens a service case and authenticates in the dealer workspace.
2. SAMI proposes two possible asset matches and shows the identifiers it used. It asks for a serial number rather than silently choosing.
3. The specialist confirms the asset. SAMI retrieves only authorized sources and shows their revision, effective date, and relevant passage.
4. SAMI requests the missing operating-hours value and computes a prequalification recommendation from the approved policy.
5. The specialist sees the recommendation, missing evidence, applicable rules, and a concise explanation. An internal-ticket write is separately previewed and approved if required.
6. The resulting case includes a decision receipt and recorded connector outcome. Final warranty authority remains with the designated approver.

## Journey B - knowledge manager corrects a bulletin
The manager uploads the corrected revision, previews affected claims and decisions, resolves conflicts, and publishes through the approved workflow. Pending decisions become ineligible immediately under the source epoch fence. Recalculation is asynchronous. Completed cases remain historically visible with an invalidated/current label and a review task where needed.

## Journey C - disconnected site
An operator activates a signed, scoped bundle on an approved edge host. SAMI answers supported queries and creates local drafts. The interface displays last sync and authority expiry. After the freshness window, connected actions and affected recommendations block or defer according to policy. Reconnection reconciles drafts and conflicts before any write.

## P0 interaction requirements
- **UX-01:** distinguish observed facts, user assertions, inferred hypotheses, and policy conclusions visually and in API metadata.
- **UX-02:** display sources, effective dates, missing fields, and reasons for abstention without requiring a technical trace.
- **UX-03:** permit corrections and overrides only to authorized users; require a reason and never mutate the historical receipt.
- **UX-04:** make draft, proposed, approved, executed, uncertain, and blocked action states visible.
- **UX-05:** support keyboard operation, readable contrast, screen-reader labels, and exportable case summaries. Target WCAG 2.2 AA; verify before release.

Hidden roleplay facts and simulated personality variables belong to the later simulation application. They must never be treated as measured psychology of real customers.

[[PAGE]]
# 07 | Priorities and release contract
P0 means required before the first paid production release. P1 follows successful pilots. R means research-only until its independent gates pass. A feature being specified does not imply that it is implemented.

| Priority | Scope | Release evidence |
| --- | --- | --- |
| P0 | Scoped ingestion, temporal claims, contradictions, provenance and deletion | Correct source lineage, ACL filtering, retraction and deletion tests. |
| P0 | Intent/slot parsing, typed decisions, policy and abstention | Held-out operational cases; risk and coverage gates. |
| P0 | Receipts, replay, dependent-decision invalidation | Snapshot reproducibility and stale-authority rejection. |
| P0 | Two launch workflows and controlled ticket writes | End-state and failure-recovery verification. |
| P0 | Native language, console, REST API, TS/Python SDKs | Supported-domain usability and offline demonstrations. |
| P0 | Identity, RBAC/ABAC, tenant isolation, operations | Adversarial tests, backups, load tests and pilot observation. |
| P1 | Sales roleplay, richer process planning, edge synchronization | Separate scenario and conflict-resolution benchmarks. |
| P1 | Optional model adapters, additional connectors/languages | Adapter ablations, egress review and language evaluation. |
| R | Program induction, transfer, process intervention learning | Controlled research experiments, counterexamples and promotion gates. |

## P0 implementation boundary
Ship one integrated Rust service plus worker and console before multiplying microservices. Use a transactionally consistent durable store and a small number of reconstructable indexes. Native generation starts with curated grammar and source-grounded extractive summaries. A large custom suffix-array engine is optional unless ablations prove a material benefit.

## Product acceptance principle
Every P0 requirement must have an owner, a reproducible fixture or measurement, and a release artifact linked to its requirement ID. Critical isolation or authorization failures block release even when aggregate accuracy is high. Research novelty cannot waive a production gate.

The engineering backlog derives from the numbered requirements in this document. A team must implement and demonstrate behavior for the chosen release scope; placeholder modules, mocked production integrations, and unstated capability substitutions do not satisfy the release contract.

[[PAGE]]
# 08 | Architecture and authority boundaries
[[ARCHITECTURE]]

## Separate information from permission
The data plane ingests documents, conversations, and connector responses as untrusted evidence. The control plane contains approved schemas, policies, workflows, tool definitions, identities, and promotion decisions. No document content can directly create or modify control-plane authority.

**Ingress:** authenticate, scope the tenant and purpose, normalize the input, and identify supported language/intent hypotheses. **Evidence:** retrieve only authorized material at a named snapshot; apply claim validity and freshness. **Decision:** evaluate typed hypotheses, rules, constraints, and measured uncertainty. **Plan:** produce a finite action/response plan. **Commit:** refresh authority and external preconditions before any effect.

Optional encoders, LLMs, or local neural models may propose interpretations or render approved language. They never possess tool credentials or direct database mutation rights. Their proposed claims pass the same validation as other untrusted inputs.

**AR-01, P0:** every request binds tenant, principal, purpose, agent version, source snapshot, rule/workflow versions, parser/calibrator versions, and execution mode. Mixed or incompatible bundles fail closed.

**AR-02, P0:** policy applies at source publication, retrieval, state updates, response planning, memory writes, action proposal, and action commit. A text filter is an additional boundary, not the security foundation.

**AR-03, P0:** unavailable evidence or authority never silently changes the execution mode. Return a typed failure or a permitted degraded result with explicit limitations.

Replay uses recorded observations and pinned versions. Re-running a connector against today's system is a new evaluation, not historical replay. Durable-workflow designs already use event histories; SAMI should build on those principles. [S09]

[[PAGE]]
# 09 | Execution modes and capability budgets
| Mode | Permitted computation | Authority and advertised scope |
| --- | --- | --- |
| Native strict, P0 default | Rules, retrieval, graphs, deterministic parsing, classical statistical models, grammar composition | No neural component or remote LLM required; bounded validated domain only. |
| Native + encoder, P1 | Optional non-generative neural retrieval/parser adapter | Clearly marked neural dependency; explicit native fallback or unsupported result. |
| Hybrid, P1 | Local/remote LLM interpretation or verbalization | Model proposes; core validates. No model-owned state, authorization or effect. |

Non-neural does not mean no learned parameters. Logistic classifiers, sparse statistics, and calibrated decision models are learned models. Non-LLM does not mean every module is non-autoregressive. Advertise the execution mode and component manifest, not a misleading blanket architectural label.

## Capability budget per request
**AR-04:** cap retrieved documents, claims, graph hops, rule firings, candidate parses, planner nodes, memory-write bytes, tool calls, and wall-clock time. Initial defaults: 30 retrieved passages, 200 candidate claims, three graph hops, 1,000 rule firings, five parse hypotheses, 100 planner nodes, and a 250 ms decision budget. These are configurable initial limits, not measured optimal values.

When a limit is reached, return `budget_exhausted` with the completed evidence set and safe next step. Never treat truncated search as evidence that no relevant fact exists. Cache keys include authorization scope, snapshot, tokenizer/parser version, locale, and source epoch.

## Offline contract
**AR-05:** offline bundles have a signature, tenant/device scope, content manifest, minimum runtime version, authority lease, and source freshness policy. Network-disabled tests prove the supported P0 workflows operate locally. Revocation cannot propagate while fully disconnected; expiry is the enforcement boundary until synchronization.

Reads may return explicitly stale informational results where policy allows. Final financial decisions remain human work. Connector writes remain drafts if the destination or required authority is unreachable. Edge data is encrypted, and local retention and device-revocation handling are part of deployment acceptance.

[[PAGE]]
# 10 | Evidence and temporal memory model
SAMI stores claims, not a single unquestionable truth field. A claim can be an observation, assertion, verified extraction, or derivation. Source reliability, extraction quality, applicability, and decision authority are distinct properties.

```json
{
  "claim_id": "cl_184", "tenant_id": "t_oem",
  "subject": "asset:unit_42", "predicate": "in_service_date",
  "value": "2025-08-12", "value_type": "date",
  "valid_time": {"from": "2025-08-12", "to": null},
  "recorded_at": "2026-10-02T06:00:00Z",
  "source": {"id": "erp_7", "version": "v19", "locator": "row:42"},
  "status": "approved", "source_family": "erp_primary",
  "extraction": {"method": "typed_connector", "version": "1.0"},
  "depends_on": [], "access_policy_id": "svc_dealer_12",
  "purpose": ["service_prequalification"], "source_epoch": 28
}
```

The schema is illustrative; published OpenAPI/JSON Schema must specify all types and validation. `valid_time` describes when the fact applies in the world. `recorded_at` and a recorded-history interval describe when SAMI knew it. Retroactive corrections retain both timelines.

## Required memory semantics
- **MEM-01:** support observed, candidate, approved, superseded, disputed, retracted, and deleted states. Only approved applicable claims enter high-impact decision checks.
- **MEM-02:** retain typed units, currencies, locale, identifier namespaces, and interval inclusivity. Normalize before comparison; reject ambiguous conversions.
- **MEM-03:** deduplicate repeated observations but preserve independent source lineage. Ten copies of one bulletin are one evidence family, not ten independent confirmations.
- **MEM-04:** capture connector authority, source revision and source location. Unverifiable extraction cannot be promoted merely because its text looks plausible.
- **MEM-05:** maintain scope and purpose at claim, edge, index and cache levels. Namespace priority never overrides authoritative policy or temporal applicability.

Memory layers remain episodic, semantic, procedural, entity/claim, and optional sequence memory. A relationship is asserted or derived with its own dependencies; graph edges do not erase uncertainty or permissions.

[[PAGE]]
# 11 | Contradictions, correction and repair
## Resolve disagreement without fabricating consensus
Two claims conflict only when their subject, predicate, typed value constraints, applicability and valid-time intervals make them incompatible. Different dates, product variants or dealer scopes may explain apparent disagreement. Preserve alternatives until scope is resolved.

**MEM-06:** source precedence is policy-defined by predicate and use case. A warranty policy manager's approved revision can supersede an older approved policy; a service message cannot. If precedence is unresolved, return `conflicting_evidence` and request review. Do not average mutually exclusive dates or infer truth from document frequency.

## Correction transaction
1. Authorize the correction and append its source/claim event.
2. Increment the affected source or policy epoch in the durable transaction; mark dependent eligibility stale immediately.
3. Enqueue an idempotent reverse-dependency traversal over derived claims, cached results, decisions, pending approvals and queued actions.
4. Recompute impacted materializations at a new snapshot. Publish results only after validation.
5. Show the manager changed conclusions, pending blocked work, and completed effects requiring follow-up.

**MEM-07:** eligibility checks consult the current epoch fence even if asynchronous invalidation has not finished. Dispatch authorization serializes with correction transactions (ACT-03). Corrections block still-queued actions; already committed/in-flight effects require cancellation where supported or review, not a promised retroactive block.

**MEM-08:** dependency traversal is cycle-safe and resumable. On a missing dependency or unsupported derivation, invalidate conservatively and require review. Do not represent an incomplete trace as a full proof.

Negative/absence-based conclusions also depend on query scope and source watermarks. Adding a previously absent entitlement record can invalidate a conclusion without retracting any existing positive claim. Declare these dependencies explicitly.

**MEM-09:** historical receipts stay immutable. Label them superseded or invalidated through linked events. Source repair cannot undo an already completed external effect; create a review or compensating action with fresh authority.

## Acceptance example
A bulletin changes applicable serial numbers from 1000-2000 to 1500-2000. An asset with serial 1200 has a queued recommendation and approved internal-ticket proposal. Publishing the correction invalidates both before dispatch authorization, updates the recommendation, and preserves the old explanation. A dispatched/created ticket remains recorded and generates cancellation/review work rather than a false rollback claim.

P0 fixtures include retroactive corrections, duplicate sources, contradictory sources, out-of-order events, dependency cycles, and correction during action approval.

[[PAGE]]
# 12 | Ingestion, source governance and deletion
## Ingestion as a controlled publication process
P0 connectors: approved PDF/Markdown manuals and bulletins, CSV/JSON asset and entitlement exports, and one service-ticket API chosen with design partners. Database/web integrations follow only after their access, versioning and deletion semantics are defined. OCR and table extraction are separate, optional dependencies; scanned documents require measured extraction quality.

**DATA-01:** extract into a quarantine area, retain source coordinates, classify document/version, validate identifier and table parsing, deduplicate, preview claim candidates, and publish with authorized approval. Source text is data, including any embedded instructions.

**DATA-02:** use connector-scoped credentials, allowlisted destinations, incremental cursors, backoff, checksums and an error queue. Fetchers enforce size/type limits and prevent SSRF. Parser workers have no tool-execution privileges and run in resource-limited isolation.

**DATA-03:** register owner, license/usage rights, retention, permitted purpose, languages, freshness, and downstream redistribution restrictions. Disallowed corpora cannot enter phrase memory or learned procedure datasets. A broad scraped corpus is not a launch prerequisite.

**DATA-07:** connectors map per-record/group source ACLs and track permission changes. Connected access requires a fresh permission cursor/check no older than five minutes by default, or a stricter source policy. Deny access when it cannot be refreshed. Received revocations trigger immediate epoch publication; measure serving exclusion within 60 seconds of receipt, separately from upstream detection lag.

## Deletion is a product contract
**DATA-04:** an authorized deletion immediately tombstones the object and removes it from serving eligibility through a durable epoch fence. Purge derived claims, excerpts, lexical/sequence shards, caches, episodic references, exports and queued promotions according to the tenant's configured deletion deadline.

**DATA-05:** P0 default targets are serving exclusion within 60 seconds and physical active-store purge within 24 hours for the reference workload. Immutable shards require replacement/compaction; backups expire within a documented default 30-day retention. Restore applies tombstones before readiness. These are proposed engineering targets to verify, not assertions of legal compliance.

**DATA-06:** keep immutable minimal receipt/audit envelopes only where retention policy permits, with separately encrypted deletable sensitive payloads. Deletion removes payloads and appends an event without editing the envelope/hash chain. Exact replay becomes `not_replayable_due_to_deletion`. Opaque identifiers and commitments are also assessed for privacy; jurisdictional retention requires deployment-specific review.

Approved feedback or knowledge updates do not become global data automatically. No cross-tenant learning is enabled by default. Consent and license records must authorize each shared artifact independently.

[[PAGE]]
# 13 | Understanding and persistent state
## Native understanding is scoped, not magical
The native parser supports a registered domain ontology, intent taxonomy, entity/slot definitions, approved synonyms, and finite grammar patterns. Start with asset identification, issue classification, bulletin applicability, entitlement evidence, clarification, correction, and escalation. Arbitrary questions are not implicitly trained decision tasks.

**NLU-01:** preserve original text offsets and normalization mappings. Handle supported negation, dates, operating-hour units, serial ranges, quoted statements, corrections, and cross-turn references. Maintain alternative parses when ambiguity affects decisions.

**NLU-02:** merge state only after validation and entity resolution. Store asserted versus confirmed values, provenance, revision, and conflict flags. Example: "It is not the 2023 unit; the newer one has 800 hours" must not attach 800 hours to the older asset.

**NLU-03:** unsupported intent, language, reference, or ambiguous quantity returns a typed clarification/unsupported status. A structured form is an available fallback. Keyword overlap cannot authorize a high-impact state change.

## State contract
Session state contains confirmed entities, input hypotheses, selected workflow, pending fields, completed obligations, authorized goals, decision history, and visibility constraints. State updates use optimistic concurrency and a versioned reducer. Every write references a base revision; concurrent updates either merge through a defined rule or return a conflict.

**STATE-01:** survive worker restart without losing confirmed data, pending approval, or workflow progress. Reconstruct from committed events and snapshots; caches are disposable.

**STATE-02:** context/session/user preferences can affect wording and convenience, but cannot override tenant policy, authoritative asset records, or mandatory evidence checks. Simulated trust/purchase-intent variables remain explicitly labeled simulation state in the later roleplay product.

## Native semantic retrieval
BM25 plus approved aliases, typed entity matching, phrase matches, and temporal/applicability filters form P0. Sparse co-occurrence/PPMI and LSA are optional ablations. Normalize or learn fusion weights on held-out labels; raw BM25 and similarity scores cannot be treated as probabilities or directly summed across unrelated scales.

**NLU-04:** evaluate entity/slot accuracy and meaning preservation by critical field. Serial numbers, policy dates and negations receive separate tests rather than disappearing into aggregate intent F1. New language packs require their own release gates.

[[PAGE]]
# 14 | Typed task and decision contracts
## Register the question before predicting an answer
Each task has an ID, version, eligible population, input/output schemas, finite choices or score semantics, evidence requirements, loss matrix, supported language/domain, policy obligations, and evaluation dataset. A free-form description may refer to a registered task; it cannot create a validated model at runtime.

**DEC-01:** support boolean, choice, multi-choice, ordinal score, rank, extraction, and action selection only where label semantics and evaluation exist. Continuous quantities require units and documented interval semantics. Deterministic eligibility decisions return rule results; calibrated estimates are separate.

**DEC-02:** hard constraints determine feasible options before scoring. Deny overrides allow. Equal-priority incompatible business rules return `policy_conflict`; rule priority does not convert an untrusted source into authority.

**DEC-03:** use normalized interpretable features and validated classifiers/rankers. Evidence decomposition explains inputs and contributions; it is not automatically a causal explanation. Source families and shared derivations prevent duplicate evidence from inflating support.

## Decision response - example, not measured performance
```json
{
  "decision_id": "dec_82", "task": "warranty_prequal@1",
  "status": "needs_human", "value": "potentially_eligible",
  "probabilities": null, "calibration_ref": null,
  "evidence": {"complete": false, "missing": ["proof_of_service"],
    "conflicts": [], "snapshot": "snap_29"},
  "checks": [{"rule": "serial_scope@4", "result": "pass"}],
  "authorized_effects": [], "receipt_id": "rec_82"
}
```

**DEC-04:** statuses include resolved, ambiguous, insufficient evidence, conflicting evidence, stale evidence, unsupported, policy denied, policy conflict, needs human, and budget exhausted. Status semantics remain stable across SDKs.

**DEC-05:** missing mandatory evidence, unavailable authority, verifier timeout, or unresolved contradiction cannot produce an authorized action. Return a useful next question or escalation reason. Rank scores need not sum to one and must not be mislabeled probability distributions.

Registered action sets include ask clarification, fetch authorized evidence, prepare case, recommend route, propose ticket write, and escalate. They do not include unrestricted tool names or arbitrary generated code.

[[PAGE]]
# 15 | Uncertainty, abstention and risk budgets
## Separate five kinds of uncertainty
**Predictive probability:** empirical estimate for a named label on an eligible population. **Evidence diagnostics:** completeness, freshness, applicability, conflicts and source lineage. **Interpretation uncertainty:** competing entity/slot parses. **Distribution shift:** divergence from validated operating conditions. **Verification status:** which formal or policy obligations passed.

**UNC-01:** do not compress these into an unexplained universal confidence float. Return class probabilities only when a compatible validated calibration artifact is available. Otherwise use null plus the reason. Deterministic rule outputs do not acquire artificial 0.99 confidence.

**UNC-02:** separate training, threshold selection, calibration, and locked test sets by customer/equipment family/time where possible. Record class counts, eligible cohorts, label provenance, missing outcomes and confidence intervals. Data overlap invalidates comparative results.

**UNC-03:** report Brier/log loss for probabilistic tasks, declared-bin ECE, cohort reliability, severe-error rate, selective risk versus coverage, and workload escalated to humans. ECE alone cannot certify safety. OOD signals include unsupported identifiers, low semantic overlap, contradictory interpretations and drift; entropy alone misses unseen inputs.

## Decision loss drives abstention
Define severe versus ordinary errors per workflow with the policy owner. Choose thresholds to minimize expected loss under a minimum useful coverage target. A system that escalates every case is accurate only in a vacuous sense and cannot pass the product gate.

**UNC-04:** meaningful parser, feature, ontology, source population or decision-model changes mark affected calibrators stale. Drift can lower authority, increase review or suspend probabilistic outputs; it cannot silently reuse old estimates as certified values.

**UNC-05, P1:** conformal sets/intervals may be added with explicit calibration assumptions. Standard marginal coverage under exchangeability does not guarantee correctness for an individual, each subgroup, or arbitrary distribution shift. [S16]

## Statistical meaning of zero observed failures
With zero severe errors among n independent evaluated accepted cases, the one-sided 95% upper bound is approximately 3/n. Therefore a target below 0.5% requires about 600 such cases even with zero errors. Correlated cases require cluster-aware analysis and may need more data. Security tests reporting zero bypasses are finite-set results, not universal guarantees.

[[PAGE]]
# 16 | Reasoning and the operational process model
## Use explicit, bounded semantics
**REAS-01:** implement typed relational queries, graph traversal, stratified rules, backward checks for required evidence, arithmetic/unit constraints, and approved procedure composition. Inference rules name their assumptions and source dependencies. Limit recursion, candidate counts and time.

**REAS-02:** retain alternative derivations. Retracting one supporting source invalidates the conclusion only if remaining permitted support is insufficient. An explicit unknown, contradicted, or absent state is not equivalent to false; use open-world semantics unless a task declares its data complete.

**REAS-03:** solver-backed checks use a named bounded/decidable fragment. Solver `unknown`, unsupported constraints and timeout are non-authorizing outcomes. A valid proof shows consistency with typed premises and rules, not that a manual or ERP record is true.

## Finite operational twin, P1
The operational model describes asset identity, case state, evidence state, actor roles, policy obligations, and permitted workflow transitions. It predicts process consequences such as whether a proposed ticket can be created, what evidence remains missing, and which queue owns the case. It is not a learned physical model of machinery or the world.

**PLAN-01:** search only registered actions with preconditions, effects, cost budgets, and termination criteria. Prefer a minimal policy-compliant plan. No autonomous open-ended objective creation is permitted.

**PLAN-02:** choose clarification using expected reduction in decision loss minus interaction cost, where probabilities are validated. Otherwise use a reviewed information-priority rule. Record why the question matters and stop when required evidence is present or escalation is warranted.

## Simulation and causality
Policy diff can show "this case would receive a different recommendation under policy v5." It does not establish what a real-world intervention would cause. A graph path, successful episode, or association rule is not causal evidence.

**REAS-04, R:** any causal module requires declared variables, intervention targets, confounding assumptions, identifiability, overlap and sensitivity analysis. Outputs distinguish identifiable effect, assumption-based simulation and not identifiable. Autonomous physical experimentation and causal claims extracted from text frequency remain excluded.

[[PAGE]]
# 17 | Decision receipts and replay
## A verifiable decision record
A receipt binds the input digest, principal and scope, session/state revision, memory snapshot, task/schema, source/claim revisions, policy and workflow versions, feasible alternatives, selected value, missing evidence, assumptions, probability artifact if any, verifier result, response plan, expiry and authority epochs.

**REC-01:** receipt envelopes are canonical, versioned, hash-linked and optionally signed. Protected facts/actor details live in separately encrypted, deletable payloads; the envelope uses governed opaque bindings/commitments. Record key rotation and linked deletion events. A signature establishes integrity and issuer, not factual correctness or action completion.

**REC-02:** a verifier accepts exactly the stated guarantee: the typed output satisfied the declared checks relative to named inputs and assumptions. Expose check coverage and unverified obligations. A trace alone is an explanation; it becomes a certificate only for obligations actually verified.

**REC-03:** replay reconstructs structured results from pinned versions, recorded external observations and a deterministic seed. Canonical hashes and specified numeric tolerances apply. Optional LLM wording need not replay exactly; its recorded output can be inspected.

**REC-04:** current eligibility is a live lookup, distinct from historical validity. A receipt can be historically valid and currently expired, invalidated, or insufficient for a new action. Deleted source content can make exact replay unavailable; expose that explicitly.

## Policy change preview
An administrator selects a proposed policy or source version and evaluates previously collected cases in a simulation branch. Show added/removed eligibility, new conflicts, review burden, and cost/coverage changes. Branches carry source and policy provenance, cannot execute tools, and never overwrite actual historical outcomes.

## External verification as a platform opportunity
P1 introduces a documented receipt verifier library and interoperability endpoints so other agent frameworks can request SAMI decisions or authorization checks. Do not describe this as a market standard before adoption. The protocol's usefulness depends on independent implementations and real integrations.

Recent work on authorization receipts and semantic isolation supplies relevant prior art. SAMI's contribution must be measured integration quality, repair consistency and deployment usefulness; these mechanisms alone do not establish novelty. [S11, S12, S13]

[[PAGE]]
# 18 | Controlled actions and durable execution
## Effect-bound action contract
**ACT-01:** the tool registry defines input/output schema, destination, effect class, required scopes, idempotency support, timeout, reconciliation query, and compensation limits. Tools receive least-privilege credentials from the execution service, never from text or a model.

**ACT-02:** action approval binds normalized arguments, tool/version, target object, actor, tenant, decision receipt, evidence/policy epochs, expiry and a single-use nonce. A changed payload or expired/revoked approval requires new authorization.

**ACT-03:** immediately before sending an effect, refresh actor authorization, source/policy eligibility and external preconditions. Use destination-side conditional writes or fencing where available. A local precheck alone cannot atomically prevent a remote change between check and effect; unsupported connectors must disclose that residual race and require constrained/human execution.

The local send boundary is a `dispatch_committed` transaction that checks epochs and records send intention, serialized with relevant correction/authorization updates. Earlier corrections block dispatch. Later corrections target in-flight cancellation/review. External completion remains separate; local ordering cannot prove authority remained unchanged at the remote effect instant.

## Durable state machine
`proposed -> verified -> awaiting_approval -> authorized -> executing -> succeeded | failed | unknown_completion`

Eligibility may independently become expired or invalidated. Persist action intention and outbox entry in the same local transaction. Consumers deduplicate event IDs. Delivery is at least once; local state transitions are idempotent.

**ACT-04:** use endpoint-enforced idempotency and stable request keys when supported. Keep the key-to-payload association across retries. The destination's retention window is part of the connector contract. Do not promise universal exactly-once external effects.

**ACT-05:** a timeout after a possible effect enters `unknown_completion`. Reconcile by external operation ID or effect query before retry. If the result is still ambiguous, route to a human. Best-effort compensation is a new authorized effect, not a universal rollback.

**WF-01:** workflows are versioned, durable, resumable and bounded. Support branching, retries, timers, approval waits, failure queues and cancellation. Pin long-lived workflow resources; upgrading a workflow follows an explicit migration strategy.

Acceptance fixtures include crash before send, crash after send, duplicate delivery, expired approval, changed policy during wait, destination version conflict, and connector success with local acknowledgment loss. Durable execution and declarative policy are established patterns to integrate, not reinvent without a reason. [S09, S10]

[[PAGE]]
# 19 | Native language and optional models
## Decide what is allowed to be said first
**LANG-01:** response plans identify speech act, approved facts, missing fields, disclosures, hidden/restricted facts, tone and length. P0 realizes plans through reviewed templates, grammar rules, typed slot filling, and short attributed excerpts.

**LANG-02:** unsupported free-form composition uses a safe reviewed explanation or clarification. Protect identifiers, numeric values, units, dates and negation from paraphrase drift. Native responses should be useful and clear within the domain; unconstrained literary fluency is outside the contract.

**LANG-03:** repetition control varies approved wording but never suppresses a necessary warning, changes a rule outcome, or rewards unsupported novelty. Domain terms override generic sequence fluency judgments. Low corpus likelihood is not grounds to reject a correct uncommon serial number or technical term.

## Optional sequence memory
Use sequence statistics to rank approved phrasing or retrieve examples only after an ablation demonstrates value. Repeated next-token prediction is autoregressive. Infini-gram's authors discuss limitations in open-ended generation; its indexing results do not establish native conversational intelligence. [S01]

**LANG-04:** store source licenses and reuse restrictions with phrase candidates. Do not stitch long protected excerpts into apparently original text. Enforce permitted excerpt length and attribution per source policy; avoid a universal copyrighted-content threshold.

## Optional model contracts, P1
Models can propose parses, candidate procedures, or language, but all are untrusted proposals. Pass only authorized minimum data, declare external egress, record model/version/configuration, enforce time/cost budgets, and provide a native fallback.

**LANG-05:** never send hidden/restricted facts to a verbalizer merely to instruct it not to reveal them. Output checks compare protected slots and required disclosures; unsupported additions trigger repair or template fallback. General semantic equivalence cannot be proved by a simple text matcher.

The compatibility API returns the actual execution profile. A hybrid deployment must not advertise a successful model-assisted result as a native no-LLM result. Separate quality metrics for structured outcomes, factual wording, and conversational preference.

[[PAGE]]
# 20 | Learning without losing control
## Learn from outcomes with known meaning
**LEARN-01:** episodic memory records pre-state, action, policy/source versions, intervention/approval, external outcome, delay, censoring, reviewer label, and evidence provenance. A thumbs-up measures satisfaction unless validated as another label; it is not proof of correctness or causal success.

**LEARN-02:** feedback enters quarantine. Deduplicate, detect suspicious sources, resolve label disputes, and evaluate candidate changes on independent data. Tenant-local learning is default. Shared learning requires explicit permitted use and anonymization; removing PII is not automatically sufficient for privacy.

**LEARN-03:** memory retention uses business obligations, purpose/consent, source authority, risk, reuse utility, redundancy and storage budgets. Rare critical policy facts remain pinned. Retire the v1 multiplicative importance formula; reward and frequency cannot decide truth or retention by themselves.

## Safe adaptation hierarchy
P0: approved source corrections and human-authored policy/phrase improvements. P1: validated classifier/threshold updates and low-impact strategy selection. R: procedure and abstraction discovery. No direct live update of permissions, mandatory policies, tool definitions or high-impact action behavior.

**LEARN-04, P1:** contextual bandits may choose among preapproved low-risk wording or clarification strategies. Record action eligibility, exploration probability, delayed outcome and exposure cohort. Never explore payout, safety, permission, or source-trust choices. Off-policy evaluation depends on support and valid propensity data; it cannot prove results for actions never observed.

**LEARN-05:** promotion creates an immutable artifact with lineage, evaluation report, reviewer, limits and rollback plan. Shadow first, then a bounded canary, then wider rollout only after risk/coverage checks. A rollback restores the prior artifact and reconciles pending runs; it does not erase completed effects.

## Measure net learning value
Report performance gain, review time, new errors, correction burden, storage/build cost and transfer. A memory entry count or higher training accuracy is not evidence that the system learned a reusable capability. An adaptation feature must improve held-out outcomes or reduce maintenance cost to remain enabled.

[[PAGE]]
# 21 | Research: experience becomes reusable programs
## The strongest research bet
Convert repeated verified episodes into small, typed, inspectable procedures. Start with pure functions for applicability, evidence assembly and escalation. Compose them into approved workflows only after testing. The target is learning structure that transfers, rather than retrieving a near-duplicate case.

**R-01:** define a restricted DSL with typed comparisons, units, relational joins, valid-time filters, predicates, known subprocedure calls, clarification and escalation. Candidate programs cannot access networks, file systems, secrets, or unrestricted code execution.

**R-02:** use enumerative/syntax-guided search and library compression under a complexity budget. Selection combines training/inner-validation task loss, program length, cost and constraint violations. The final third-family transfer set stays locked and unused for selection/tuning. Minimum description length encourages compact explanations; it does not certify truth.

**R-03:** each candidate declares input/output schema, pre/postconditions, permitted scope, evidence/policy dependencies, termination limits, learned-from episode IDs, test cases and known counterexamples. An approved procedure may be invalidated by later source/policy corrections.

## Transfer experiment
Train on two equipment families and selected procedure combinations. Hold out a third family, new serial/date combinations, paraphrases, and retrospective policy changes. Compare hand-authored rules, classifier plus retrieval, retrieval of episodes, synthesis without a reusable library, and library-based synthesis.

Gate: at least 30% lower measured authoring effort or labeled-example requirement at matched coverage and risk, across three held-out task groups, with no new critical failure in the declared adversarial set. Pre-register stopping rules and assess statistical uncertainty. This is a target, not an achieved result.

**R-04:** review counterexamples, run property/metamorphic tests, shadow in the domain simulator, and require authorized human promotion. No learned rule silently enters production. Stop or narrow the research if gains vanish under leakage-controlled tests or review costs outweigh savings.

DreamCoder demonstrates library-learning and compositional program synthesis as relevant prior work, including neural-guided search in its published approach. It is inspiration, not evidence that a purely non-neural SAMI implementation will achieve the same abilities. [S17]

[[PAGE]]
# 22 | Storage and deployment architecture
## Simplify the first implementation
Use Rust/Tokio/Axum for the online runtime and TypeScript/React for the console. Python is appropriate for offline evaluation, calibration and research. Reuse tested components where practical; new engineering concentrates on evidence contracts, repair, decision semantics, and verification.

| Responsibility | Proposed default | Boundary |
| --- | --- | --- |
| Authoritative state | PostgreSQL with transactions/outbox | Claims, revisions, dependencies, sessions, policies, workflows, receipts. |
| Scoped local/edge | SQLite and encrypted local bundle | Local journal, snapshots and approved domain pack. |
| Search | Reconstructable Tantivy index | BM25 and typed metadata filters; not authority source. [S18] |
| Documents/bundles | S3-compatible object store or local files | Versioned originals, manifests and signed release artifacts. |
| Hot optimizations | In-process cache; RocksDB if measured | Disposable/rebuildable projections only. |
| Distribution | Outbox worker first; NATS later | At-least-once event delivery and idempotent consumers. |
| Operations | OpenTelemetry-compatible traces/metrics | Avoid sensitive evidence in generic logs. |

This is an intentional replacement of the original mandatory MongoDB/Redis/NATS stack. MongoDB remains a viable alternative if an architecture review chooses it with equivalent transactional/versioning guarantees; do not require both databases. Redis, ClickHouse and Kubernetes are optional scale additions.

**OPS-01:** development Compose profile runs API, worker, console and durable store with sample licensed/de-identified data. Production profiles declare dependencies and readiness checks. No paid language API is required for native mode.

**OPS-02:** schemas and domain packs are versioned. Migrations test rollback/forward recovery and pinned workflow compatibility. Safe deployment keeps old and new artifacts available for outstanding runs.

## Consistency and scaling
Authoritative transactions write state and an outbox together. Derived indexes publish a snapshot watermark. Reads require a compatible watermark or explicitly return stale/degraded status. Per-tenant partitions and bounded caches protect noisy neighbors; horizontal stateless API scaling follows demonstrated bottlenecks.

A database choice is an engineering decision, not the intelligence breakthrough. V2 prioritizes coherent semantics and an installation small enough for the initial customer to operate.

[[PAGE]]
# 23 | Indexing, memory growth and build cost
## Sequence indexing is optional and accountable
**IDX-01:** if adopted, use licensed, domain-scoped corpora and immutable checksummed shards. Manifest records tokenizer/version, corpus digest, document boundaries, shard pointer width, endianness, count, format version and source rights. Corrupt or incompatible shards fail closed.

**IDX-02:** sequence queries never cross document or tenant boundaries. Reserve boundary symbols and validate hot/cold merge semantics. Context-hash collisions require original-context verification. Counts from overlapping shard versions must not be double-counted.

**IDX-03:** dynamic memory comes from a durable journal and small hot indexes. Compaction creates a new artifact, validates it, then atomically switches the manifest. Old artifacts remain for authorized replay and retire under retention/deletion rules.

## Capacity planning example - not a benchmark
For N tokens, an uncompressed stream plus suffix pointer array requires approximately N multiplied by token-ID bytes plus pointer bytes, before metadata and all other indexes.

| Tokens | 2-byte IDs + 4-byte local pointers | 2-byte IDs + 8-byte pointers |
| --- | --- | --- |
| 100 million | 0.6 GB | 1.0 GB |
| 1 billion | 6.0 GB | 10.0 GB |
| 10 billion | 60.0 GB in bounded shards | 100.0 GB |

Decimal GB; pointers represent token offsets within the chosen shard format. Four-byte offsets cannot address more than about 4.29 billion positions in one shard. Build workspace, originals, metadata, search statistics, replicas, backups and retained versions are additional. Memory mapping does not eliminate page faults or SSD bandwidth limits.

**IDX-04:** before building, estimate peak RAM, temporary disk, final disk, elapsed time, source duplication and operational copies. Refuse a build when capacity policy fails. Report deletion/compaction and incremental update cost in TCO.

**IDX-05:** retention prioritizes source obligations and utility, not popularity alone. Cap sparse co-occurrence vocabulary and counts; prune with measurable recall loss. A billion-token phrase corpus is not required for the first useful service desk application.

Infini-gram establishes large-index feasibility under its own hardware and implementation. SAMI must benchmark its own format and workload rather than inherit published throughput claims. [S01]

[[PAGE]]
# 24 | Security, privacy and control
## P0 trust boundaries
**SEC-01:** derive tenant/principal scope from authenticated identity. Never trust a body-supplied tenant ID as authorization. API keys are scoped, hashed at rest, revocable and rotated; console uses OIDC. RBAC governs roles, while ABAC applies dealer, workspace, purpose and source-level constraints.

**SEC-02:** enforce isolation in database queries, graph traversals, search prefilters, cache keys, bundle distribution, traces, exports and receipt inspection. Database row policies are defense in depth, not a substitute for application checks. Test cross-tenant identifiers and malicious mixed-scope requests.

**SEC-03:** retrieved text and external tool output cannot change executable policies, permission scopes, tool definitions, or goals. Treat prompt injection and poisoned memory as data attacks even in native mode; regex/rule systems can also be abused.

**SEC-04:** use TLS for connected services, encryption at rest, managed secrets, scoped connector identities, dependency scanning, SBOMs, signed releases and restricted worker networks. Protect index/bundle signing keys and receipt-verifier trust roots.

**SEC-05:** run file/OCR parsers and research DSL evaluators in isolated, resource-limited workers. Validate archive paths, decompression ratios, MIME types and file sizes. No untrusted document can invoke a shell or fetch arbitrary internal URLs.

## Privacy and incident response
**SEC-06:** classify PII and sensitive operational information, minimize collection, restrict purpose, configure retention, and mask logs/exports. Document which optional models receive data and where processing occurs. Certification and legal-compliance claims require separate evidence and review.

**SEC-07:** keep append-only minimal audit events for access, source/rule publication, overrides, approvals, execution, learning promotion, deletion and key changes. Sensitive payloads are separately governed so deletion does not require preserving private text forever.

**SEC-08:** provide kill switches for tool effects, learning promotion, source/bundle eligibility and compromised connector keys. Incident response identifies affected tenants/receipts, quarantines authority, restores safe versions, and creates accountable follow-up tasks.

Release testing covers tenant escape, authorization bypass, stale approval, poisoned source publication, data resurrection, parser exploitation, replay tampering and secret leakage. All critical failures block production; zero failures in a finite test set is reported with its scope.

[[PAGE]]
# 25 | APIs, SDKs and developer experience
## Native interfaces, P0
| Endpoint | Contract |
| --- | --- |
| POST /v1/decisions | Registered task, typed input, snapshot/profile; no implicit free-form model creation. |
| POST /v1/respond | Session revision and message; typed plan, native response and decision status. |
| POST /v1/claims/candidates | Governed memory proposal; publication is a separate authorized operation. |
| POST /v1/sources/{id}/publish | Approved revision, impact preview and epoch-fenced publication. |
| GET /v1/receipts/{id} | Scoped historical evidence/checks and current eligibility state. |
| POST /v1/receipts/{id}/replay | Pinned historical replay; no external tool effects. |
| POST /v1/actions/propose | Validated registered effect with argument digest and required authority. |
| POST /v1/actions/{id}/approve | Effect-bound approval, revision and expiry. |
| POST /v1/actions/{id}/execute | Commit checks, idempotency key and reconciliation-aware status. |
| POST /v1/feedback | Attributed label/outcome; quarantine rather than immediate promotion. |

Administration also exposes sessions, source deletion/export, workflow runs, domain packs, evaluation artifacts, health and readiness. Pagination, scoped filtering, cancellation, rate limits and stable error codes are defined in OpenAPI.

**API-01:** SDKs for TypeScript and Python are P0; Rust SDK follows P1. Client retries distinguish safe reads, idempotent writes and uncertain effects. Errors include request ID and recovery action without secrets, stack traces or internal paths.

**API-02:** optimistic concurrency is required for state/control changes; stale revisions return conflict. API mutation retries use an idempotency key bound to payload and principal. SSE streams decision, response and action-state events with resumable event IDs where supported.

## Compatibility, P1
Offer a documented `/v1/chat/completions` subset for existing clients. Map content deltas and output length explicitly. Unsupported fields receive a validation error or explicit warning. Do not silently equate temperature, token probabilities, reasoning tokens, function calling or arbitrary model capability with SAMI semantics.

**DX-01:** CLI supports init, serve, ingest, validate, publish, evaluate, replay, doctor, export, backup and restore. Provide a runnable launch-domain example, schema documentation, supported-intent list, migration instructions and an offline tutorial. A ten-minute local demonstration is a usability target to verify.

[[PAGE]]
# 26 | Domain packs and administration console
## Domain pack as the reusable product unit
A pack contains ontology and identifier schemas, source applicability definitions, supported intents/slots, decision tasks, rules, workflows, response grammar, calibrators where valid, fixtures, licenses, version manifest and limits. Dynamic tenant claims remain separately versioned. Rename v1's broad "model bundle" where it implies a pretrained general model.

**PACK-01:** validate schema compatibility, dependencies, policy conflicts, missing response slots, unreachable workflow states, permitted licenses and required fixtures before signing/publishing. Manifest pins hashes, versions and supported execution modes.

**PACK-02:** show migration impact on outstanding sessions and actions. New policy packs cannot silently reinterpret old approvals. Publish immutable versions with rollback and explicit live-authority rules.

## P0 console workspaces
- **Service workbench:** current case, entity candidates, missing fields, recommendation, sources, human review and action preview.
- **Knowledge workbench:** ingestion status, source versions, claim candidates, contradiction groups, applicability filters, deletion and correction impact.
- **Policy/workflow workbench:** tested rule editor, decision-task definition, tool registration and release review.
- **Evaluation/release workbench:** risk/coverage, drift, case cohorts, replay diff, shadow/canary results, and gate status.
- **Operations/admin:** identity/scopes, connector health, authority expiry, audit, storage/cost, retention and incident controls.

**ADMIN-01:** explain changes in terms a service manager can use: which assets/cases are affected, what conclusion changed, why, and who must review. Raw graph visualization is optional; the task outcome is primary.

**ADMIN-02:** separate edit and approve roles for consequential policy publication. Every production artifact has an owner, change rationale, review record and release report. Provide deterministic policy tests before approval.

**ADMIN-03:** capture authoring and integration hours. If packs require extensive bespoke engineering for every customer, the proposed scaling advantage fails. By the third pilot, target deployment within ten working days after clean approved source access; record actual effort and exclusions.

P1 may add rich graph inspection, sales simulations, additional languages, third-party procedure packages and marketplace distribution. Do not introduce a marketplace until pack licensing, security and independent verification work.

[[PAGE]]
# 27 | Performance and reliability targets
All numbers below are proposed acceptance targets. No SAMI benchmark or implementation has been run in this review. Test on a declared reference profile: 16 vCPU, 64 GB RAM, 1 TB NVMe, Linux; native English; 50 million indexed tokens, 500,000 approved claims, 50,000 assets, and bounded three-hop reasoning. Publish actual CPU/storage models and corpus layout.

| Metric | Initial GA target | Required measurement |
| --- | --- | --- |
| Structured decision latency | p95 <= 250 ms; p99 <= 750 ms | Warm and cold cache, under sustained declared load. |
| Native response latency | p95 <= 500 ms; p99 <= 1.5 s | Complete checked response; exclude external tool/network wait separately. |
| Sustained mixed load | 100 decisions/s or 20 responses/s | Separate workloads, 60-minute run; no claim of simultaneous capacity. |
| Approved-source visibility | p95 <= 60 s | Accepted publication to searchable compatible snapshot. |
| Revocation eligibility fence | Within durable publication transaction | Stale dependent requests blocked before async repair completes. |
| Repair completion | p95 <= 5 min for 10,000 dependents | Defined fan-out, load and queue backlog. |
| Serving exclusion after deletion | <= 60 s | All active serving paths; offline replicas subject to authority lease. |
| Hosted service availability | 99.9% monthly | Explicit availability definition and maintenance policy. |

**OPS-03:** record p50/p95/p99, throughput, CPU, RSS, page faults, IO, corpus/shard size, request mix, concurrent load, cache conditions, accuracy and coverage. Report timeouts and abstentions; do not improve speed by hiding difficult requests.

**OPS-04:** back up durable state, approved packs and sources under retention policy. Target connected-service RPO <= 15 minutes and RTO <= 4 hours. A quarterly restore drill, tombstone reapplication and readiness verification substantiate these goals. Local device journal loss follows its documented backup policy.

**OPS-05:** expose per-stage traces and operational alerts for stale indexes, repair backlog, connector ambiguity, failed publication, storage limits, drift and authority expiry. Logs avoid source text and identifiers by default.

**OPS-06:** loss of the authoritative store, mandatory policy artifact, or compatible source epoch blocks effects and unverified decisions. Informational cached results require an explicitly permitted degraded mode. Do not market a cache hit as full availability.

[[PAGE]]
# 28 | Evaluation: prove the actual advantage
## Compare strong systems fairly
Freeze model versions, prompt/schema policies, data, budget, connector behavior and task definitions at experiment start. Evaluate at least these arms:

| Arm | Included capability | Purpose |
| --- | --- | --- |
| A: conventional baseline | BM25, classical classifier, hand rules, forms/templates and durable workflow | Tests whether ordinary software already solves the problem. |
| B: hosted LLM system | Strong current models from two suppliers, proper retrieval/memory, structured outputs and the same policy/tool guard | Tests SAMI against competent modern deployment. |
| C: local model system | Strong practical local model with identical sources, workflows and permissions | Tests private/CPU or local-hardware alternative economics. |
| D: SAMI | Native; then optional encoder/hybrid separately | Establishes native contribution and adapter value. |

Ablate temporal memory, repair propagation, sequence memory, episodic reuse and learned procedure library. Report authoring effort and component cost. Giving SAMI manually curated answers while competitors receive raw documents would invalidate the comparison; either supply the same curated data or account for curation differences.

## Dataset and scoring
**EVAL-01:** collect consented, de-identified real cases plus synthetic corner cases. Lock at least 1,200 test cases per P0 task as an initial plan, with at least 600 accepted cases where the severe-error gate is assessed. Increase sample size if clustering, rare failures or subgroup analysis require it.

**EVAL-02:** split by time, equipment family and source revision. Include contradictions, missing records, negation, wrong serials, stale policies, ACL changes, unknown completion, and unseen intent. Prevent evaluation labels from entering memory.

**EVAL-03:** primary score is correct, authorized case outcome plus human-review burden. Verify database/workflow end state. Independently review factual explanation and source applicability. Report repeated-trial consistency, risk/coverage curves, cohort metrics and uncertainty.

LongMemEval supplies memory tasks; use the cleaned pinned release and separately inspect its newer agentic variant. Retrieval recall is not answer correctness. Tau-bench supplies interactive-policy evaluation patterns; simulated-user fidelity and benchmark-version changes require care. Neither alone proves industrial deployment readiness. [S14, S15]

This document includes no competitive results. A breakthrough claim must name the tested workload, mode, baseline, metric and experimental limits.

[[PAGE]]
# 29 | Release gates and definition of done
## Gate 0 - customer and data feasibility
At least three representative data-sharing design partners, documented source rights, baseline costs/error taxonomy, named control owners, and access to one credible connector. If unmet, continue discovery rather than claiming production implementation readiness.

## Gate 1 - native vertical prototype
Two workflows execute end to end in a simulator and a network-disabled environment. Approved input scope is explicit. Entity/critical-slot accuracy and applicability are measured on held-out cases; prototype errors remain supervised. Replay, missing-evidence handling, epoch invalidation and action reconciliation work in integrated fixtures.

## Gate 2 - controlled paid pilot
Run recommendation-only shadow evaluation first, followed by specialist-assistance with human review. Observe four weeks of representative use. Target >= 20% lower median triage/review time versus matched baseline cases. Pre-register a 14-day reopen-rate noninferiority margin of two percentage points and use a one-sided 95% interval; extend observation if underpowered. Publish risk/coverage, applicability, overrides, connector uncertainty and support effort. No autonomous consequential effects are introduced.

## Gate 3 - GA for the declared scope
- **Quality:** >= 99% source/version applicability accuracy on the locked applicable test set; critical identity/date/negation fields reported separately. Recommendation severe-error one-sided 95% upper bound <= 0.5% at >= 60% supported-case coverage, where sample/independence assumptions support it.
- **Control:** zero observed tenant escapes, unauthorized effects or stale-authority bypasses in the declared adversarial/property test sets. All critical defects closed; residual risks documented.
- **Consistency:** 100% structured replay agreement within declared numeric tolerances for retained deterministic fixtures; complete deletion and invalidation tests.
- **Operations:** performance workload met, restore drill passed, incident runbook exercised, signed artifacts and monitored authority/repair queues.
- **Usability:** >= 80% unaided completion on representative scripted tasks among at least ten specialists, with no critical unsafe completion; accessible core flows, named approval owners and tested fallbacks.
- **Business:** three paid pilots/commitments; measured positive customer productivity value after platform, infrastructure and amortized onboarding costs. At actual GA prices, direct serving/support contribution margin >= 50% over a representative month; record exclusions and pack deployment effort. Pilot pricing may be deliberately lower.

These gates may be tightened after risk review; they cannot be waived because a demo is impressive. A failed gate restricts supported scope or keeps the release in pilot.

## Gate 4 - breakthrough research claim
Require the H1-H4 experiment reports, leakage review, strongest-baseline comparison, reproducible harness and external technical review. GA success alone does not demonstrate a new intelligence paradigm.

[[PAGE]]
# 30 | Roadmap, team and execution plan
Illustrative 12-month plan from an approved kickoff. Calendar milestones are conditional on data access and measured progress; not delivery promises.

| Window | Deliverable | Exit condition / accountable lead |
| --- | --- | --- |
| Months 0-2 | Discovery, ontology, dataset, baseline and source-rights register | Gate 0; product lead with domain specialist. |
| Months 3-4 | Integrated native prototype, temporal memory, decisions and repair | Gate 1; core engineering lead. |
| Months 5-6 | Pilot console, connector, receipts, controlled actions, offline profile | Pilot readiness; integration/security leads. |
| Months 7-9 | Paid pilots, risk/coverage tuning, packs and restore/load tests | Gate 2 and GA evidence; product/QA leads. |
| Months 10-12 | Scoped GA if qualified; independent benchmark and P1 prioritization | Gate 3; engineering/product joint sign-off. |
| Parallel, bounded | Pure typed-program induction experiments | R-01 to R-04; research lead; no GA dependency. |

## Initial team - planning assumption
Eight FTE: one product/domain lead, two Rust/data-runtime engineers, one integrations/workflow engineer, one frontend/product engineer, one evaluation/ML researcher, one QA/security/reliability engineer, and one domain implementation engineer. Obtain part-time domain reviewers and security assessment; roles can be combined initially with an explicit scope reduction.

**Budget model:** eight FTE at an illustrative fully loaded $8,000 per month equals $768,000 annually. Add $100,000 for infrastructure, source preparation, reviewers and independent assessment: $868,000 subtotal. A 20% contingency gives approximately $1.04 million. These are location-independent planning inputs to replace with actual hiring/vendor quotes, not researched salary figures.

## Architectural milestones before expansion
Pick one durable store and version semantics. Specify task labels and risk taxonomy before model development. Establish a conventional baseline before custom sequence work. Implement correction and deletion before accumulating large memory. Complete connector reconciliation before permitting writes. Keep procedure research isolated and bounded.

A smaller team should extend the timeline and ship fewer integrations. Scope must follow actual capacity; a 150-module feature checklist is not a staffing plan. Each milestone ends with working behavior and evidence, not code volume or scaffolding counts.

[[PAGE]]
# 31 | Economics, go-to-market and moat
## Customer-value example - assumptions, not revenue forecast
At 10,000 cases/month and 12 minutes/case, baseline work is 2,000 hours. At an illustrative $30/hour, labor allocation is $60,000/month. If 60% of cases use SAMI and average three minutes are saved, the potential productivity value is 300 hours or $9,000/month. This does not imply headcount reduction or realized cash savings.

Illustrative customer costs: $2,500/month platform fee, $500/month customer infrastructure, and $20,000 onboarding amortized over 24 months ($833/month). Total $3,833/month gives potential net productivity value of $5,167/month under those assumptions. If only one minute is saved on 30% of cases, value falls to $1,500/month and the purchase does not pay back on time savings alone.

## Supplier unit economics
Illustrative direct monthly serving/support costs of $800 infrastructure + $1,000 implementation/support allocation + $300 audit/monitoring = $2,100. At 6,000 policy-compliant assisted cases, cost is $0.35/case before R&D, sales, general overhead, optional models and build amortization. Against $2,500 platform revenue this yields only 16% contribution margin. The first pilot price is not a scalable business model.

Track true support/onboarding effort and whether it falls with reusable packs. Raise realized value and pricing, reduce serving/support cost, or narrow the customer. Do not claim high margins because inference uses CPUs. Meter attempted cases, accepted cases, successful outcomes, human reviews, storage and optional model usage separately.

## Commercial sequence
Sell paid, scoped discovery/pilots with a measurable before/after study. Start with a founder-accessible equipment segment, then a reusable pack for a second OEM/dealer group. Price experimentation may test a platform fee plus bounded usage and paid onboarding; validate willingness to pay rather than inventing a market size.

## Defensibility
Build reusable verified domain packs, permissioned outcome datasets, reliable connectors, independent receipt verification, and evidence of lower lifecycle cost. Customer raw data remains customer-owned under contract. A proprietary dataset moat cannot depend on unauthorized cross-tenant learning.

Expansion follows successful transfer: more service workflows, related industrial domains, then a model-independent operational runtime for third-party agents. Competitive advantage must survive the strongest conventional and LLM-assisted alternatives.

[[PAGE]]
# 32 | Risks, pivots and decisions to resolve
| Risk | Mitigation and decision gate |
| --- | --- |
| Native parsing limits useful coverage | Forms/clarification, scoped ontology, evaluate optional adapters separately; narrow domain if coverage remains low. |
| Knowledge curation is too expensive | Measure authoring hours; improve pack reuse; stop general-platform expansion if every customer requires bespoke engineering. |
| Rules outperform research cheaply | Ship the operational product; do not mislabel procedure retrieval as a new learning breakthrough. |
| Wrong source/identity creates confident error | Typed scope, critical-field validation, authority policy, conflicts and review; no score can waive evidence obligations. |
| Dependencies miss stale conclusions | Conservative invalidation, epoch fences, property tests, coverage audit and bounded graph semantics. |
| Connector effect remains uncertain | Reconcile-first status, external conditional writes, human review for unsupported guarantees. |
| Learning rewards wrong outcomes | Separate satisfaction/correctness/outcome labels, quarantine, cohort evaluation and review. |
| Offline authority becomes stale | Signed leases, restricted writes, sync reconciliation; disclose revocation delay. |
| Incumbent copies feature set | Win on pack deployment, measured outcomes, verifier interoperability and support economics. |
| Index/storage growth erodes CPU advantage | Optional sequence engine, quotas, measured build cost and retention/deletion design. |

## Stop or pivot criteria
After two evaluated pilot iterations, narrow or change the wedge if supported coverage stays below 60% at the risk target, customers reject the effort/value tradeoff, clean authoritative data cannot be obtained, or conventional tools achieve equal outcomes at substantially lower total cost. Pause program-learning claims if gains vanish on held-out families or review cost exceeds saved authoring effort.

## Decisions requiring founder/team validation
Confirm real customer access and equipment segment; assign policy/final-approval owners; choose the first ticket connector; agree what counts as severe error; set data residency/retention needs; replace staffing and pricing assumptions with quotes and interviews. These are explicit pre-build discovery tasks rather than hidden assumptions.

Worldwide originality requires a broader competitor/prior-art review, and patentability would require specialist assessment. Neither is established by this document. Perfection is not an engineering exit criterion; a supported scope, measurable utility and controlled failure behavior are.

[[PAGE]]
# 33 | Acceptance scenarios and integration tests
These executable scenarios represent product behavior. They accompany, rather than replace, unit/property, migration, fuzz, load and adversarial testing.

| Scenario | Required result and evidence |
| --- | --- |
| Ambiguous asset | Preserve candidates; request serial; no inferred operating hours merged into the wrong asset. NLU-02. |
| Missing warranty record | Return insufficient evidence unless completeness is explicitly guaranteed; absence is not a denial. REAS-02. |
| Wrong bulletin revision | Only applicable approved revision supports the case; show effective date and serial scope. MEM-01/02. |
| Equally authoritative contradiction | Return conflict and reviewer; no frequency-based winner. MEM-06. |
| Retroactive source correction | Old receipt remains visible; future eligibility fenced; affected queued action blocked. MEM-07/09. |
| Duplicate document evidence | Source copies do not multiply independent support or predictive certainty. MEM-03. |
| Policy update during approval | Existing approval cannot execute changed arguments or eligibility without refresh. ACT-02/03. |
| Tool timeout after effect | Enter unknown completion, query destination, and avoid blind duplicate send. ACT-05. |
| Deleted user data | No active retrieval/learning resurrection; restore reapplies tombstone; replay indicates erased content. DATA-04/05/06. |
| Disconnected expired authority | Informational output is labeled if allowed; connector effects blocked/drafted. AR-05. |
| Malicious manual instruction | Document cannot alter tool/goal/policy authority. DATA-01/SEC-03. |
| High model score, missing evidence | Clarify or escalate; no authorized effect. DEC-05/UNC-01. |
| Learning candidate fails counterexample | Remains quarantined; current production behavior unchanged. R-04. |
| LLM adds an unsupported date | Repair/template fallback; response cannot mutate case state. LANG-05. |

## Original application retention
Duplicate-billing support becomes a P1 domain pack with identity, settlement checks, reconciliation and human financial approval. Sales roleplay remains P1: one hidden pain point per turn, explicit reveal conditions, and simulation-only persona state. Immediate-meeting-resolution behavior is a scenario policy, never a demand to fabricate a solution when evidence is unavailable.

Each failed production scenario becomes a regression fixture with versioned expected structured outcomes. Repetition/fluency preferences are evaluated separately from policy and factual correctness. Evaluators must not accept a pleasing response that violates the required workflow.

[[PAGE]]
# 34 | Traceability to the original 154 sections
All original sections were reviewed. This mapping is grouped by original range and records disposition. "Preserved" means preserved as a design requirement, not already implemented.

| Original v1 sections | V2 destination and disposition |
| --- | --- |
| 1-9: vision, principles, scope | 01-07, 09: vision preserved; general platform reduced to staged claims and execution profiles. |
| 10-17: architecture and memory | 08-12, 22-23: temporal evidence and repair added; sequence engine optional. |
| 18-23: rules, procedures, episodes, state | 13, 16, 18, 20-21: typed semantics, ambiguity, outcomes and bounded learning. |
| 24-34: decisions and calibration | 14-15: registered tasks, null probabilities, risk/coverage and drift handling. |
| 35-42: conversation and roleplay | 06, 13, 19, 33: state preserved; persona/roleplay moved P1 and labeled simulation. |
| 43-53: support, language, tools, workflows | 05-06, 18-19, 25: vertical scope, effect-bound authority and reconciliation. |
| 54-64: learning, ingestion, bundles | 12, 20-21, 26: governed sources, deletion, safe promotion and domain packs. |
| 65-73: storage and implementation | 22-23, 30: simplified store; Rust/TypeScript preserved; offline Python research. |
| 74-84: console, APIs, SDKs | 06, 25-26: task-oriented console, explicit compatibility subset, phased SDKs. |
| 85-92: namespaces and security | 09-12, 24: authority distinct from scope; prefiltered retrieval and governance. |
| 93-105: operations and scaling | 22-23, 27: workload-defined targets, build/TCO, restore/degraded-mode contract. |
| 106-108: CLI and configuration | 09, 22, 25: operational commands and mode/dependency manifest. |
| 109-124: evaluation and quality | 15, 24, 27-29, 33: fair baselines, risk gates, adversarial and integration fixtures. |
| 125-130: explanation/reasoning/build | 10-11, 16-17, 23: verifier limits, repair semantics and reproducible artifacts. |
| 131-138: phases, apps, comparisons | 03-07, 28-31, 33: launch wedge and honest full-system comparison. |
| 139-147: risks and abstraction | 13, 20-21, 32: generalization research, retention correction and pivot gates. |
| 148-154: providers, DoD, coding instructions | 08-09, 19, 29-30, 35: controlled adapters; staged DoD replaces all-features mandate. |

V2 is a complete proposed replacement for planning and implementation. It deliberately retires unsupported blanket claims and defers low-priority breadth. Building every original module at once would conflict with the validated-release approach.

[[PAGE]]
# 35 | Implementation handoff and decision record
## Obligations of the implementation team
Create an integrated working product for the approved phase, with durable data and real connector behavior. Maintain a requirement-to-test matrix, documented schemas and error semantics. Do not substitute placeholder modules, mocked production storage, silent LLM fallbacks or unsupported score calculations for the defined behavior.

Use established search, storage, parser, policy and workflow libraries where their semantics fit. Build custom sequence indexing or general workflow engines only when evidence supports their incremental value. Keep native no-network acceptance runnable independently of optional adapters.

## Suggested code ownership boundaries
`core` owns types and version semantics; `evidence` owns claims/lineage/repair; `interpretation` owns scoped parse hypotheses; `decision` owns tasks/calibration; `policy` owns admissibility; `execution` owns effects/reconciliation; `language` owns response plans; `evaluation` owns datasets/gates; `console` owns user workflows. These are ownership boundaries, not a requirement for separate services or dozens of crates.

## Required release artifacts
Versioned domain pack and component manifest; OpenAPI/SDK docs; data-rights register; benchmark recipe and locked test digest; quality/cohort report; adversarial/replay/deletion report; performance and build/TCO report; deployment and migration instructions; tested backup/restore; incident/rollback runbooks; signed artifact/SBOM; supported-capability declaration.

## Decision record for v2
| Decision | Reason and review trigger |
| --- | --- |
| Industrial service/warranty wedge | Fits original domain examples and offline value; validate customer/data access before commitment. |
| Native core, optional adapters | Keeps independence while permitting measured quality improvements; disclose each mode. |
| PostgreSQL + SQLite default | Simplifies versioned transactions; replace only with reviewed equivalent semantics. |
| Sequence engine optional | Generalization benefit unproven; require ablation before large engineering investment. |
| Human final consequential authority | Limits initial failure cost and supports evidence gathering; automation requires separate risk gate. |
| Procedure induction isolated research | Strong capability hypothesis with uncertain feasibility; no production dependency. |

The founder, product owner, engineering lead and policy owner should sign off after discovery and risk definitions. This document is proposed, not approved by those parties. The user requested a stronger project and a corrected PDF; no software, external communication, customer commitment or deployment was performed here.

[[PAGE]]
# 36 | Sources: market and established capabilities
All live documentation accessed 2 October 2026. Source links support public capabilities and prior art, not independent performance claims. The supplied v1.0 PRD is the local source document reviewed in full; this v2 is a proposed redesign.

[S01] Liu et al., **Infini-gram: Scaling Unbounded n-gram Language Models to a Trillion Tokens**. Submitted 30 January 2024; revised 7 April 2025. [Paper](https://arxiv.org/abs/2401.17377), [full text and limitations](https://arxiv.org/html/2401.17377v4).

[S02] Letta, **Introduction to Stateful Agents**. Persistent state and memory documentation. [Official documentation](https://docs.letta.com/v1-sdk/concepts/stateful-agents).

[S03] Mem0, **Open Source Overview** and local companion configuration. [Overview](https://docs.mem0.ai/open-source/overview), [local companion](https://docs.mem0.ai/cookbooks/companions/local-companion-ollama).

[S04] Zep / Graphiti, **Official Graphiti repository**. Temporal facts, episodes, ontology and hybrid retrieval. [Repository](https://github.com/getzep/graphiti).

[S05] TypeSafe, **Introducing System One Models & Jev**, 15 September 2026. [Announcement](https://typesafe.ai/blog/introducing-system-one-models-and-jev), [System One semantics](https://docs.typesafe.ai/concepts/system-one), [confidence semantics](https://docs.typesafe.ai/confidence). Typed outputs do not establish individual-answer correctness.

[S06] Rasa, **Conversational AI with Language Models (CALM)**. Interpretation, flow and wording separation. [Official documentation](https://rasa.com/docs/learn/concepts/calm/).

[S07] Google Cloud, **Generative versus deterministic**, Dialogflow CX. Updated 24 September 2026 UTC. [Official documentation](https://docs.cloud.google.com/dialogflow/cx/docs/generative-deterministic).

[S08] Palantir, **AIP Logic overview** and **App Building overview**. Enterprise operational logic, ontology and actions. [AIP Logic](https://www.palantir.com/docs/foundry/logic), [App Building](https://www.palantir.com/docs/foundry/app-building/overview/).

[S09] Temporal, **What is Temporal?** and **Workflow Definition**. Durable execution and deterministic replay semantics. [Overview](https://docs.temporal.io/temporal), [workflow constraints](https://docs.temporal.io/workflow-definition).

[S10] Open Policy Agent, **HTTP API authorization** and **Policy Language**. Declarative fine-grained access controls. [API authorization](https://www.openpolicyagent.org/docs/http-api-authorization), [policy language](https://www.openpolicyagent.org/docs/policy-language).

No vendor performance percentages or market-size claims are imported into SAMI targets. A research scan cannot prove that an implementation is commercially unavailable or patentable.

[[PAGE]]
# 37 | Sources: research, evaluation and limits
[S11] Santos-Grueiro, **Temporary Authority, Permanent Effects: Commit-Time Authorization for LLM Agents**. Submitted 11 July 2026; preprint. Prior research on authority freshness and effect binding at commit. [Paper](https://arxiv.org/abs/2607.10487).

[S12] Uchibeke, **Before the Tool Call: Deterministic Pre-Action Authorization for Autonomous AI Agents**. Submitted 21 March 2026; preprint. Declarative pre-action checks and signed authorization records. [Paper](https://arxiv.org/abs/2603.20953).

[S13] Mozafari, **BEGIN AI TRANSACTION: Semantic Isolation for Durable AI Workflows**. Submitted 5 August 2026; preprint. Versioned semantic resource consistency in long-lived execution. [Paper](https://arxiv.org/abs/2608.05412).

[S14] Wu et al., **LongMemEval: Benchmarking Chat Assistants on Long-Term Interactive Memory**. Submitted 14 October 2024; revised 4 March 2025; ICLR 2025. [Paper](https://arxiv.org/abs/2410.10813), [official repository and cleaned-data/version notices](https://github.com/xiaowu0162/LongMemEval). Pin the benchmark revision; score retrieval and answer correctness separately.

[S15] Yao et al., **Tau-bench: A Benchmark for Tool-Agent-User Interaction in Real-World Domains**. Submitted 17 June 2024. End-state and repeated-trial evaluation pattern. [Paper](https://arxiv.org/abs/2406.12045), [official benchmark repository](https://github.com/sierra-research/tau-bench).

[S16] Barber et al., **Conformal prediction beyond exchangeability**. Submitted 27 February 2022. Coverage assumptions and distribution-shift limitations. [Paper](https://arxiv.org/abs/2202.13415).

[S17] Ellis et al., **DreamCoder: Growing generalizable, interpretable knowledge with wake-sleep Bayesian program learning**. Submitted 15 June 2020. Compositional library learning, with neural-guided components in the published method. [Paper](https://arxiv.org/abs/2006.08381).

[S18] Tantivy, **Query module and crate documentation**. Full-text/BM25 component; distributed behavior is a system-level responsibility. [Query API](https://docs.rs/tantivy/latest/tantivy/query/index.html), [crate documentation](https://docs.rs/crate/tantivy/latest).

[S19] Jon Doyle, **A Glimpse of Truth Maintenance**, MIT AI Memo 461, 1 February 1978. Dependency maintenance has long-standing prior art. [Primary archive](https://dspace.mit.edu/entities/publication/e274a3b1-dcb7-4d62-abe7-a4b0db173191).

Published results of these systems are their authors' findings, not SAMI results. The source register is selective and public; private vendor implementations and unpublished research were not examined.

[[PAGE]]
# 38 | The ambition and the evidence standard
## The most ambitious credible version of SAMI
An organization gives SAMI approved knowledge, typed goals, policies and workflows. SAMI builds a memory whose claims carry time, scope and dependencies. It handles supported decisions locally, knows when evidence is inadequate, and exposes what each conclusion depends on. A correction repairs the eligible future behavior without rewriting history. Actions carry fresh authority and reconcile actual external outcomes.

Over time, verified cases can yield reusable typed procedures, subject to independent testing and controlled promotion. If those procedures compose and transfer with less manual engineering than strong alternatives, SAMI earns a claim beyond memory storage: an experimentally demonstrated way to accumulate operational capability.

> The breakthrough to pursue is accountable capability accumulation: useful experience becoming reusable, verifiable behavior that remains correctable as knowledge and policy change.

## What the evidence must demonstrate
The company must show useful supported-case coverage, lower severe-error risk or lower total cost at matched quality, rapid correct memory repair, reliable external effects, and substantially reduced domain-authoring effort. Results must remain strong against complete LLM systems and conventional enterprise software. Independent review and reproducibility strengthen the claim.

Memory, knowledge graphs, truth maintenance, workflow engines, typed decisions and program synthesis all have prior art. [S19, S17] The product can still be valuable and the integration can still advance the field. Global novelty and broad intelligence superiority are questions for evidence, not adjectives for a PRD.

## Next execution order
Validate customer and source access; define task labels and error costs; establish the conventional and LLM-system baselines; build the two native vertical workflows; prove correction, deletion, replay and effect control; run paid pilots; pursue program-transfer experiments under separate gates.

This roadmap preserves the user's ambition to compete with major technology companies while giving it a testable path. A successful SAMI could compete for the infrastructure that makes operational AI reliable across model suppliers, and for bounded tasks where independent native operation wins. Frontier general intelligence remains a longer-term research possibility, not a promised outcome.
