import { useEffect, useMemo, useState } from 'react';
import researchInputExamples from './research-examples.json';
import defaultPack from './default-pack.json';
import { Api, type RecordValue, identifier, objectValue, parseJson, pretty, records } from './api';

type View = 'cases' | 'evidence' | 'authoring' | 'decisions' | 'actions' | 'research' | 'governance';
type Run = (label: string, operation: () => Promise<unknown>, onSuccess?: (result: unknown) => void) => Promise<void>;
const views: { id: View; label: string; mark: string }[] = [
  { id: 'cases', label: 'Case workbench', mark: '01' },
  { id: 'evidence', label: 'Evidence & memory', mark: '02' },
  { id: 'authoring', label: 'Rules & domain packs', mark: '03' },
  { id: 'decisions', label: 'Decisions & receipts', mark: '04' },
  { id: 'actions', label: 'Action approvals', mark: '05' },
  { id: 'research', label: 'Research & evaluation', mark: '06' },
  { id: 'governance', label: 'Access & operations', mark: '07' },
];

const sourceExample: RecordValue = {
  id: 'manual-reviewed', content: '', status: 'candidate', source_family: 'oem-primary',
  acl: ['service.read'], purpose: ['service_prequalification'],
  license: 'Customer-owned; indexing and internal excerpts permitted',
  local_authority: true, max_age_seconds: 86400,
  valid_from: '2026-01-01', valid_to: null,
};
const claimExample: RecordValue = {
  id: 'claim-reviewed', subject: 'asset:unit-42', predicate: 'operating_hours',
  value: 800, value_type: 'number', unit: 'hour', source_id: 'manual-reviewed',
  source_revision: 1, source_family: 'oem-primary', locator: 'page:1', status: 'candidate',
  depends_on: [], authority: 'oem', qualifiers: { product: 'industrial-pump', jurisdiction: 'IN' },
  acl: ['service.read'], purpose: ['service_prequalification'], valid_from: '2026-01-01', valid_to: null,
};
const packExample: RecordValue = { ...objectValue(defaultPack), id: 'industrial-service-custom' };
const ruleExample: RecordValue = {
  id: 'safety-review', priority: 900, effect: 'require_review',
  when: { field: 'issue', op: 'eq', value: 'issue.safety' },
};

function JsonOutput({ value, title = 'Server result' }: { value: unknown; title?: string }) {
  if (value === undefined) return <div className="empty"><span className="empty-glyph">↗</span><strong>No result yet</strong><p>Connect a credential and run an operation to inspect the actual response.</p></div>;
  return <section className="result"><div className="result-heading"><h3>{title}</h3><button className="quiet" onClick={() => {
    const blob = new Blob([pretty(value)], { type: 'application/json' });
    const url = URL.createObjectURL(blob); const link = document.createElement('a');
    link.href = url; link.download = 'sami-result.json'; link.click(); URL.revokeObjectURL(url);
  }}>Export JSON</button></div><pre tabIndex={0} aria-label={title}>{pretty(value)}</pre></section>;
}

function JsonEditor({ label, value, setValue, hint }: { label: string; value: string; setValue: (value: string) => void; hint?: string }) {
  const id = useMemo(() => `json-${crypto.randomUUID()}`, []);
  return <label className="field" htmlFor={id}><span>{label}</span><textarea id={id} className="code-input" value={value} spellCheck={false} onChange={e => setValue(e.target.value)} rows={14} />{hint && <small>{hint}</small>}</label>;
}

function Field({ label, value, setValue, type = 'text', hint, placeholder }: { label: string; value: string; setValue: (value: string) => void; type?: string; hint?: string; placeholder?: string }) {
  const id = useMemo(() => `field-${crypto.randomUUID()}`, []);
  return <label className="field" htmlFor={id}><span>{label}</span><input id={id} type={type} value={value} placeholder={placeholder} onChange={e => setValue(e.target.value)} />{hint && <small>{hint}</small>}</label>;
}

function CaseWorkbench({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [message, setMessage] = useState('');
  const [serial, setSerial] = useState(''); const [product, setProduct] = useState('');
  const [hours, setHours] = useState(''); const [date, setDate] = useState('');
  const [jurisdiction, setJurisdiction] = useState(''); const [pack, setPack] = useState('industrial-service');
  const [asset, setAsset] = useState(''); const [asOf, setAsOf] = useState(() => new Date().toISOString().slice(0, 10));
  const [task, setTask] = useState('service_triage'); const [session, setSession] = useState('');
  const [result, setResult] = useState<unknown>(); const [receipt, setReceipt] = useState<unknown>();
  const top = objectValue(result); const decision = top.decision ? objectValue(top.decision) : top;
  const input: RecordValue = { message };
  if (serial) input.serial_number = serial; if (product) input.product = product;
  if (hours !== '') input.operating_hours = Number(hours);
  if (date) input.purchase_date = date; if (jurisdiction) input.jurisdiction = jurisdiction;
  if (asset) input.asset_id = asset; if (asOf) input.as_of = asOf;
  return <><div className="page-intro"><div><span className="eyebrow">SERVICE OPERATIONS</span><h1>A case. Its evidence. A clear next step.</h1><p>Prepare triage and warranty prequalification with current approved knowledge. Final service and coverage authority stays with your reviewer.</p></div><span className="mode">Native runtime</span></div>
    <div className="notice"><span>i</span><p>This workbench supports internal recommendations and administrative tickets. Physical repairs, machinery control and financial approval are outside its scope.</p></div>
    <div className="split"><section className="panel"><div className="section-heading"><h2>Case details</h2><span className="label">Draft input</span></div>
      <label className="field"><span>Customer description</span><textarea rows={4} value={message} onChange={e => setMessage(e.target.value)} placeholder="Describe the issue and information the customer provided." /></label>
      <div className="field-grid"><Field label="Confirmed asset record ID" value={asset} setValue={setAsset} hint="Use the approved entity ID. Demo fixtures include asset:unit_42 and asset:unit_84." /><Field label="Decision as-of date" type="date" value={asOf} setValue={setAsOf} /><Field label="Asset serial number" value={serial} setValue={setSerial} /><Field label="Product / model" value={product} setValue={setProduct} /><Field label="Operating hours" type="number" value={hours} setValue={setHours} /><Field label="Purchase date (user assertion)" type="date" value={date} setValue={setDate} /><Field label="Policy jurisdiction" value={jurisdiction} setValue={setJurisdiction} /><Field label="Domain pack" value={pack} setValue={setPack} /></div>
      <label className="field"><span>Registered task</span><select value={task} onChange={e => setTask(e.target.value)}><option value="service_triage">Service triage</option><option value="warranty_prequalification">Warranty prequalification</option></select></label>
      <button className="primary" disabled={busy || !message.trim()} onClick={() => void run('Evaluate case', () => api.post('/decisions', { task_id: task, input, pack_id: pack }), r => { setResult(r); setReceipt(undefined); })}>Evaluate case <span>→</span></button>
      <details className="details"><summary>Persistent conversation</summary><p>Create a session, then send a message to update its validated state. Decisions and actions remain separate operations.</p><Field label="Session ID" value={session} setValue={setSession} /><div className="button-row"><button disabled={busy} onClick={() => void run('Create session', () => api.post('/sessions', { state: {} }), r => { setSession(String(objectValue(r).id ?? objectValue(r).session_id ?? '')); setResult(r); })}>Create session</button><button disabled={busy || !session || !message} onClick={() => void run('Send conversation turn', () => api.post('/respond', { session_id: session, message }), setResult)}>Send turn</button></div></details>
    </section><section className="panel"><div className="section-heading"><h2>Decision and evidence</h2><span className="label">Actual response</span></div>
      {result !== undefined && <div className="decision-summary"><span className="eyebrow">{String(decision.status ?? 'Recorded result')}</span><h3>{typeof decision.value === 'string' ? decision.value.replaceAll('_', ' ') : 'Review structured output'}</h3><p>{String(decision.response ?? decision.message ?? decision.reason ?? 'Review applicable sources, completeness and checks before proceeding.')}</p><div className="chips">{['decision_id', 'receipt_id', 'snapshot'].map(key => decision[key] && <span key={key}>{key}: {String(decision[key])}</span>)}</div></div>}
      <JsonOutput value={result} title="Case result" />
      {decision.receipt_id && <button disabled={busy} onClick={() => void run('Inspect receipt', () => api.get(`/receipts/${identifier(decision.receipt_id)}`), setReceipt)}>Inspect evidence receipt</button>}
      {receipt !== undefined && <JsonOutput value={receipt} title="Evidence receipt" />}
    </section></div></>;
}

function ResourceWorkbench({ api, run, busy, kind, initial, sourceActions = false }: { api: Api; run: Run; busy: boolean; kind: string; initial: RecordValue; sourceActions?: boolean }) {
  const [rows, setRows] = useState<RecordValue[]>([]); const [payload, setPayload] = useState(pretty(initial));
  const [selected, setSelected] = useState(''); const [result, setResult] = useState<unknown>();
  const [reason, setReason] = useState(''); const [deleteConfirmed, setDeleteConfirmed] = useState(false);
  const load = () => run(`Load ${kind}`, () => api.list(kind), r => setRows(records(r)));
  useEffect(() => { setPayload(pretty(initial)); setSelected(''); setRows([]); setResult(undefined); }, [kind, initial]);
  const select = (row: RecordValue) => { setSelected(String(row.id ?? '')); setPayload(pretty(row)); setDeleteConfirmed(false); };
  return <div className="resource-grid"><section className="panel resource-list"><div className="section-heading"><h2>{kind.replaceAll('_', ' ')}</h2><button disabled={busy} onClick={() => void load()}>Refresh</button></div>
    <p className="muted">Load records visible to your authenticated principal.</p>
    {rows.length === 0 && <p className="empty-list">No records loaded.</p>}
    {rows.map((row, i) => <button className={`record-row ${selected === row.id ? 'selected' : ''}`} key={String(row.id ?? i)} onClick={() => select(row)}><strong>{String(row.id ?? `Record ${i + 1}`)}</strong><span>{String(row.status ?? row.version ?? row.revision ?? 'Stored record')}</span></button>)}
    <button className="quiet" onClick={() => { setPayload(pretty(initial)); setSelected(''); setDeleteConfirmed(false); }}>+ New record</button>
  </section><section className="panel"><div className="section-heading"><h2>{selected ? 'Inspect / update record' : 'Create governed record'}</h2><span className="label">{selected || 'New'}</span></div>
    <JsonEditor label="Record fields" value={payload} setValue={setPayload} hint="Keep the returned revision when editing. The API validates permissions, schemas and concurrent updates." />
    <div className="button-row"><button className="primary" disabled={busy} onClick={() => void run(`Save ${kind}`, () => api.save(kind, parseJson(payload)), r => { setResult(r); setPayload(pretty(r)); setSelected(String(objectValue(r).id ?? '')); })}>Save record</button>
    {selected && <button disabled={busy} onClick={() => void run('Read current record', () => api.get(`/admin/${encodeURIComponent(kind)}/${encodeURIComponent(selected)}`), r => { setResult(r); setPayload(pretty(r)); })}>Read current version</button>}</div>
    {sourceActions && <div className="governed-controls"><h3>Source lifecycle</h3><p>Publish only after source rights, applicability and extraction have been reviewed. Retraction fences future dependent decisions. Deletion removes serving eligibility and schedules physical purge.</p><Field label="Review / retraction reason" value={reason} setValue={setReason} /><div className="button-row"><button disabled={busy || !selected || !reason.trim()} onClick={() => void run('Publish approved source', () => api.post(`/sources/${encodeURIComponent(selected)}/publish`, { reason }), setResult)}>Publish source</button><button disabled={busy || !selected || !reason.trim()} onClick={() => void run('Retract source', () => api.post(`/sources/${encodeURIComponent(selected)}/retract`, { reason }), setResult)}>Retract source</button></div><label className="check"><input type="checkbox" checked={deleteConfirmed} onChange={e => setDeleteConfirmed(e.target.checked)} />I reviewed the source identifier and its deletion impact.</label><button className="danger" disabled={busy || !selected || !deleteConfirmed} onClick={() => void run('Delete source', () => api.delete(`/sources/${encodeURIComponent(selected)}`), r => { setResult(r); setDeleteConfirmed(false); })}>Delete selected source</button></div>}
    {!sourceActions && kind === 'claims' && <p className="muted">To retract a claim, retain its current revision and save its status as <code>retracted</code>. Supporting source publication remains a separate governed operation.</p>}
    {kind === 'packs' && <div className="governed-controls"><h3>Publish reviewed domain pack</h3><p>A saved pack remains a candidate until validation and an authorized publication. Evaluate it in research and review its task and rule semantics before publishing.</p><Field label="Pack publication reason" value={reason} setValue={setReason} /><button disabled={busy || !selected || !reason.trim()} onClick={() => void run('Publish reviewed domain pack', () => api.post(`/packs/${encodeURIComponent(selected)}/publish`, { reason }), setResult)}>Publish domain pack</button></div>}
    <JsonOutput value={result} />
  </section></div>;
}

function Evidence({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [kind, setKind] = useState('sources');
  return <><Header eyebrow="CORRECTABLE MEMORY" title="The evidence behind the answer." text="Review source rights and scope, publish approved revisions, and inspect claims with temporal provenance." /><Tabs options={['sources', 'claims', 'entities', 'relations', 'conflicts']} value={kind} setValue={setKind} /><ResourceWorkbench api={api} run={run} busy={busy} kind={kind} initial={kind === 'sources' ? sourceExample : kind === 'claims' ? claimExample : genericExamples[kind] ?? { id: `${kind}-reviewed` }} sourceActions={kind === 'sources'} /></>;
}

const genericExamples: Record<string, RecordValue> = {
  entities: { id: 'asset-unit-42', entity_type: 'asset', serial_number: 'UNIT-42', product: 'industrial-pump' },
  relations: { id: 'unit-model', subject: 'asset:unit-42', predicate: 'model', object: 'product:industrial-pump', depends_on: [], status: 'candidate' },
  conflicts: { id: 'conflict-review', claim_ids: [], status: 'open', reason: 'Requires authorized source review' },
  workflows: { id: 'service-review', version: '1.0.0', initial: 'identify', status: 'approved', states: { identify: { action: 'ask_clarification', transitions: { confirmed: 'review' } }, review: { action: 'escalate', terminal: true } } },
  decision_schemas: { id: 'routing-review', type: 'choice', choices: ['technical', 'warranty', 'review'], required_fields: ['serial_number'], evidence_requirements: [], loss_matrix: {}, calibration_ref: null },
  sessions: { id: 'session-review', state: {}, status: 'open' },
  evaluations: { id: 'evaluation-review', dataset_id: 'synthetic-demo', synthetic: true, task_id: 'service_triage', result: {}, status: 'recorded' },
};

function Authoring({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [kind, setKind] = useState('packs');
  return <><Header eyebrow="VERSIONED BEHAVIOR" title="Make the rules explicit." text="Domain packs define supported tasks, rules, wording and workflow behavior. Test candidates before using them in a case." /><div className="notice"><span>i</span><p>Storing a standalone rule is a proposal. Add reviewed rules to the active domain pack explicitly; text content never grants control-plane authority.</p></div><Tabs options={['packs', 'rules', 'workflows', 'decision_schemas', 'sessions']} value={kind} setValue={setKind} /><ResourceWorkbench api={api} run={run} busy={busy} kind={kind} initial={kind === 'packs' ? packExample : kind === 'rules' ? ruleExample : genericExamples[kind]} /></>;
}

function Header({ eyebrow, title, text }: { eyebrow: string; title: string; text: string }) {
  return <div className="page-intro"><div><span className="eyebrow">{eyebrow}</span><h1>{title}</h1><p>{text}</p></div></div>;
}

function Tabs({ options, value, setValue }: { options: string[]; value: string; setValue: (value: string) => void }) {
  return <div className="tabs" aria-label="Resource categories">{options.map(option => <button key={option} aria-pressed={value === option} className={value === option ? 'active' : ''} onClick={() => setValue(option)}>{option.replaceAll('_', ' ')}</button>)}</div>;
}

function Decisions({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [input, setInput] = useState(pretty({ task_id: 'service_triage', pack_id: 'industrial-service', input: { message: '', serial_number: '' } }));
  const [id, setId] = useState(''); const [result, setResult] = useState<unknown>(); const [history, setHistory] = useState<RecordValue[]>([]);
  return <><Header eyebrow="ACCOUNTABLE DECISIONS" title="Inspect. Reconstruct. Compare." text="Run registered tasks and inspect historical receipts separately from current eligibility. Replay never sends external effects." /><div className="split"><section className="panel"><h2>Typed decision</h2><JsonEditor label="Decision request" value={input} setValue={setInput} /><button className="primary" disabled={busy} onClick={() => void run('Run typed decision', () => api.post('/decisions', parseJson(input)), r => { setResult(r); setId(String(objectValue(r).receipt_id ?? '')); })}>Run decision</button><div className="divider" /><h2>Evidence receipt</h2><Field label="Receipt ID" value={id} setValue={setId} /><div className="button-row"><button disabled={busy || !id} onClick={() => void run('Read evidence receipt', () => api.get(`/receipts/${encodeURIComponent(id)}`), setResult)}>Read receipt</button><button disabled={busy || !id} onClick={() => void run('Replay pinned receipt', () => api.post(`/receipts/${encodeURIComponent(id)}/replay`, {}), setResult)}>Replay snapshot</button></div><details className="details"><summary>Browse decision records</summary><button disabled={busy} onClick={() => void run('Load decisions', () => api.list('decisions'), r => setHistory(records(r)))}>Load history</button>{history.map((row, i) => <button className="record-row" key={String(row.id ?? i)} onClick={() => { setResult(row); setId(String(row.receipt_id ?? '')); }}><strong>{String(row.id ?? row.decision_id ?? i)}</strong><span>{String(row.status ?? '')}</span></button>)}</details></section><section className="panel"><JsonOutput value={result} title="Decision / receipt output" /></section></div></>;
}

function Actions({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [payload, setPayload] = useState(pretty({ decision_id: '', tool_id: 'internal.ticket', arguments: { title: '', description: '', queue: 'review' } }));
  const [id, setId] = useState(''); const [reason, setReason] = useState(''); const [key, setKey] = useState('');
  const [result, setResult] = useState<unknown>(); const [rows, setRows] = useState<RecordValue[]>([]);
  return <><Header eyebrow="CONTROLLED EFFECTS" title="Approval belongs to a specific action." text="Review the target, arguments, evidence and expiry. The runtime rechecks authority at execution and reconciles uncertain completion." /><div className="split"><section className="panel"><h2>Propose an administrative action</h2><JsonEditor label="Action proposal" value={payload} setValue={setPayload} /><button className="primary" disabled={busy} onClick={() => void run('Propose action', () => api.post('/actions/propose', parseJson(payload)), r => { setResult(r); setId(String(objectValue(r).id ?? objectValue(r).action_id ?? '')); setKey(crypto.randomUUID()); })}>Validate proposal</button><div className="divider" /><h2>Review and execute</h2><Field label="Action ID" value={id} setValue={setId} /><Field label="Approval reason" value={reason} setValue={setReason} /><Field label="Execution idempotency key" value={key} setValue={setKey} hint="Keep this key for the same action. After a timeout, inspect or reconcile before retrying." /><div className="button-row"><button disabled={busy || !id} onClick={() => void run('Read current action', () => api.get(`/admin/actions/${encodeURIComponent(id)}`), setResult)}>Read action</button><button disabled={busy || !id || !reason} onClick={() => void run('Approve action for 5 minutes', () => api.post(`/actions/${encodeURIComponent(id)}/approve`, { reason, expires_in_seconds: 300 }), setResult)}>Approve for 5 minutes</button><button className="primary" disabled={busy || !id || !key} onClick={() => void run('Execute approved action', () => api.request('POST', `/actions/${encodeURIComponent(id)}/execute`, {}, 30_000, key), setResult)}>Execute approved action</button><button disabled={busy || !id} onClick={() => void run('Reconcile action outcome', () => api.post(`/actions/${encodeURIComponent(id)}/reconcile`, {}), setResult)}>Reconcile outcome</button></div><p className="muted">Approval alone does not create authority. The server rejects expired, changed, denied or invalidated plans.</p></section><section className="panel"><JsonOutput value={result} title="Action state / connector outcome" /><details className="details"><summary>Action queue</summary><button disabled={busy} onClick={() => void run('Load action queue', () => api.list('actions'), r => setRows(records(r)))}>Refresh queue</button>{rows.map((row, i) => <button className="record-row" key={String(row.id ?? i)} onClick={() => { setId(String(row.id ?? '')); setResult(row); setKey(''); }}><strong>{String(row.id ?? i)}</strong><span>{String(row.status ?? row.state ?? '')}</span></button>)}</details></section></div></>;
}

const researchExamples: Record<string, RecordValue> = Object.fromEntries(
  Object.entries(researchInputExamples)
    .filter(([name, value]) => name.includes('.') && value !== null && typeof value === 'object' && !Array.isArray(value))
    .map(([name, value]) => [name, objectValue(value)]),
);

function Research({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [operation, setOperation] = useState('calibration.fit'); const [custom, setCustom] = useState('');
  const [payload, setPayload] = useState(pretty(researchExamples['calibration.fit'])); const [result, setResult] = useState<unknown>();
  const [feedback, setFeedback] = useState(pretty({ decision_id: '', type: 'human_override', label: '', reason: '' }));
  return <><Header eyebrow="BOUNDED EXPERIMENTS" title="Measure capability before claiming it." text="Research endpoints execute bounded native routines. Small generated examples demonstrate code paths; they do not establish production accuracy or a breakthrough." /><div className="notice"><span>i</span><p>Demo inputs are synthetic. Use locked, consented held-out datasets for release evaluation. New behavior stays quarantined until reviewed promotion.</p></div><div className="split"><section className="panel"><h2>Native research operation</h2><label className="field"><span>Operation</span><select value={operation} onChange={e => { setOperation(e.target.value); setPayload(pretty(researchExamples[e.target.value] ?? {})); }}>{Object.keys(researchExamples).map(name => <option key={name} value={name}>{name.replaceAll('_', ' ')}</option>)}</select></label><Field label="Override operation name (optional)" value={custom} setValue={setCustom} hint="Use the operation names declared by your runtime version. Unsupported operations return a typed server error." /><JsonEditor label="Experiment input" value={payload} setValue={setPayload} /><button className="primary" disabled={busy} onClick={() => void run('Run bounded research operation', () => api.post(`/research/${encodeURIComponent(custom || operation)}`, parseJson(payload)), setResult)}>Run operation</button><div className="divider" /><h2>Attributable feedback</h2><JsonEditor label="Feedback event" value={feedback} setValue={setFeedback} /><button disabled={busy} onClick={() => void run('Record quarantined feedback', () => api.post('/feedback', parseJson(feedback)), setResult)}>Submit feedback</button></section><section className="panel"><JsonOutput value={result} title="Research / feedback result" /></section></div><details className="details"><summary>Recorded evaluation artifacts</summary><ResourceWorkbench api={api} run={run} busy={busy} kind="evaluations" initial={genericExamples.evaluations} /></details></>;
}

function Governance({ api, run, busy }: { api: Api; run: Run; busy: boolean }) {
  const [keyRequest, setKeyRequest] = useState(pretty({ principal: 'service-reviewer', scopes: ['decision.invoke', 'knowledge.search', 'receipt.read', 'session.read', 'session.write'], groups: ['service.read'] }));
  const [keyId, setKeyId] = useState(''); const [result, setResult] = useState<unknown>();
  const [controls, setControls] = useState(pretty({ id: 'runtime', effects_enabled: false, learning_enabled: false, sources_enabled: true }));
  const [audit, setAudit] = useState<RecordValue[]>([]);
  return <><Header eyebrow="GOVERNANCE & OPERATIONS" title="Know who can act. Know what changed." text="Credentials and runtime controls are enforced by the backend. Treat a newly created key as a secret; it is returned only at creation." /><div className="split"><section className="panel"><h2>Runtime health</h2><div className="button-row"><button disabled={busy} onClick={() => void run('Check service health', () => new Api('', '').get('/health'), setResult)}>Health</button><button disabled={busy} onClick={() => void run('Check readiness', () => new Api('', '').get('/ready'), setResult)}>Readiness</button><button disabled={busy} onClick={() => void run('Load controls', () => api.list('controls'), r => { setResult(r); const rows = records(r); if (rows[0]) setControls(pretty(rows[0])); })}>Read runtime controls</button></div><h2 className="spaced">Kill switches / eligibility</h2><JsonEditor label="Runtime control record" value={controls} setValue={setControls} hint="Read current controls before editing to preserve revision. Disabling effects leaves conversation and inspection available." /><button className="danger" disabled={busy} onClick={() => void run('Update runtime controls', () => api.save('controls', parseJson(controls)), r => { setResult(r); setControls(pretty(r)); })}>Apply controls</button><div className="divider" /><h2>API credential management</h2><JsonEditor label="New key scope" value={keyRequest} setValue={setKeyRequest} /><div className="button-row"><button disabled={busy} onClick={() => void run('Create scoped API key', () => api.post('/keys', parseJson(keyRequest)), setResult)}>Create key</button><button disabled={busy} onClick={() => void run('List key metadata', () => api.get('/keys'), setResult)}>List key metadata</button></div><Field label="Key ID to revoke" value={keyId} setValue={setKeyId} /><button className="danger" disabled={busy || !keyId} onClick={() => void run('Revoke API key', () => api.post(`/keys/${encodeURIComponent(keyId)}/revoke`, {}), setResult)}>Revoke key</button></section><section className="panel"><JsonOutput value={result} title="Administrative result — may contain a new secret" /><div className="divider" /><div className="section-heading"><h2>Audit history</h2><button disabled={busy} onClick={() => void run('Load audit history', () => api.get('/audit'), r => setAudit(records(r)))}>Load audit</button></div><div className="audit-list">{audit.length === 0 && <p className="muted">No audit records loaded.</p>}{audit.map((event, i) => <details key={String(event.id ?? i)}><summary>{String(event.event ?? event.action ?? event.kind ?? 'Audit event')} <span>{String(event.created_at ?? event.timestamp ?? '')}</span></summary><pre>{pretty(event)}</pre></details>)}</div></section></div></>;
}

export function App() {
  const [view, setView] = useState<View>('cases');
  const [credential, setCredential] = useState(() => sessionStorage.getItem('sami-token') ?? '');
  const [token, setToken] = useState(() => sessionStorage.getItem('sami-token') ?? '');
  const [remember, setRemember] = useState(() => sessionStorage.getItem('sami-token') !== null);
  const [busy, setBusy] = useState(false); const [error, setError] = useState(''); const [notice, setNotice] = useState('');
  const api = useMemo(() => new Api(token), [token]);
  const run: Run = async (label, operation, onSuccess) => {
    setBusy(true); setError(''); setNotice(`${label}…`);
    try { const result = await operation(); onSuccess?.(result); setNotice(`${label} completed.`); }
    catch (e) { setError(e instanceof Error ? e.message : 'The operation failed. Inspect server logs with the request ID.'); setNotice(''); }
    finally { setBusy(false); }
  };
  return <div className="app"><a href="#main" className="skip-link">Skip to workbench</a><aside className="sidebar"><a className="brand" href="#main"><span className="brand-symbol">S</span><span>SAMI<small>OPERATIONAL INTELLIGENCE</small></span></a><div className="sidebar-rule" /><span className="nav-caption">WORKSPACE</span><nav aria-label="Main workbenches">{views.map(item => <button key={item.id} className={view === item.id ? 'active' : ''} aria-current={view === item.id ? 'page' : undefined} onClick={() => { setView(item.id); setError(''); setNotice(''); }}><span>{item.mark}</span>{item.label}</button>)}</nav><div className="sidebar-footer"><span className="dot" />Native · CPU-first<small>v0.2.0 / proposed product scope</small><p>Correctable evidence.<br />Accountable decisions.</p></div></aside><div className="main-shell"><header className="topbar"><span className="breadcrumb">Workspace <span>/</span> {views.find(item => item.id === view)?.label}</span><details className="credentials"><summary><span className={token ? 'dot' : 'dot off'} />{token ? 'Credential connected' : 'Connect credential'}</summary><div className="credential-popover"><h2>Connect your workspace</h2><p>Paste a scoped SAMI API key or a bearer token obtained from your configured OIDC provider. The console does not create a local password account.</p><Field label="Bearer credential" type="password" value={credential} setValue={setCredential} /><label className="check"><input type="checkbox" checked={remember} onChange={e => setRemember(e.target.checked)} />Keep in this tab's session storage</label><small>Default storage is memory. Session storage remains readable by scripts on this origin; use a trusted host.</small><div className="button-row"><button className="primary" disabled={!credential.trim()} onClick={() => { setToken(credential.trim()); if (remember) sessionStorage.setItem('sami-token', credential.trim()); else sessionStorage.removeItem('sami-token'); setNotice('Credential set. Server authorization is checked with each operation.'); }}>Connect</button><button onClick={() => { setToken(''); setCredential(''); sessionStorage.removeItem('sami-token'); setNotice('Credential cleared.'); }}>Disconnect</button></div></div></details></header><main id="main" tabIndex={-1}>
    {!token && <div className="connection-hint">Connect a credential above to access tenant records. Data appears only after a real API operation.</div>}
    <div aria-live="polite" className={`operation-status ${busy ? 'working' : ''}`}>{notice}</div>{error && <div role="alert" className="error"><strong>Operation blocked</strong><p>{error}</p><button className="quiet" onClick={() => setError('')}>Dismiss</button></div>}
    {view === 'cases' && <CaseWorkbench api={api} run={run} busy={busy} />}
    {view === 'evidence' && <Evidence api={api} run={run} busy={busy} />}
    {view === 'authoring' && <Authoring api={api} run={run} busy={busy} />}
    {view === 'decisions' && <Decisions api={api} run={run} busy={busy} />}
    {view === 'actions' && <Actions api={api} run={run} busy={busy} />}
    {view === 'research' && <Research api={api} run={run} busy={busy} />}
    {view === 'governance' && <Governance api={api} run={run} busy={busy} />}
  </main><footer className="main-footer"><span>SAMI — memory, evidence, execution.</span><span>Deployment and release status must be verified against the PRD gates.</span></footer></div></div>;
}
