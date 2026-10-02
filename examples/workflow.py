#!/usr/bin/env python3
import os, sys, json
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'sdk' / 'python'))
from sami_client import SamiClient
api=SamiClient(os.environ.get('SAMI_API_URL','http://127.0.0.1:8080/v1'),os.environ['SAMI_API_KEY'])
run=api.request('POST','/workflows/service_case/run',{})
print('Plan:',json.dumps(run['plan'],indent=2))
# These are generated process steps with no registered external tools. This example
# confirms demonstration process state only; never use it to fabricate evidence.
for step in run['plan']['steps']:
    run=api.request('POST',f"/workflow-runs/{run['id']}/advance",{'revision':run['revision'],'confirm_step':step['id'],'reason':'Confirmed generated example step; no customer evidence asserted'})
print(json.dumps({'run_id':run['id'],'status':run['status'],'external_effects':run['external_effects']},indent=2))
