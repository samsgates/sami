#!/usr/bin/env python3
"""Run a generated supported case, receipt replay, and an approved internal ticket.
Set SAMI_API_KEY to a credential authorized for this demo; never commit it.
"""
import json
import os
from pathlib import Path
import sys
import uuid
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'sdk' / 'python'))
from sami_client import SamiClient

def main():
    token = os.environ['SAMI_API_KEY']
    api = SamiClient(os.environ.get('SAMI_API_URL', 'http://127.0.0.1:8080/v1'), token)
    decision = api.decide('service_triage', {'message': 'Serial 1600 is overheating', 'asset_id': 'asset:unit_84', 'as_of': '2026-10-02'})
    if decision['status'] != 'resolved':
        raise RuntimeError(f"Review evidence before any effect: {decision['status']}")
    replay = api.replay(decision['receipt_id'])
    assert replay['signature_valid'] and replay['structured_match']
    action = api.propose_action(decision['decision_id'], 'internal.ticket', {'title': 'Generated overheating review', 'description': decision['response'], 'queue': 'thermal_service_team'})
    approved = api.approve_action(action['id'], 'Reviewed generated source applicability')
    assert approved['state'] == 'authorized'
    key = str(uuid.uuid4())
    complete = api.execute_action(action['id'], key)
    retry = api.execute_action(action['id'], key)
    assert complete['state'] == retry['state'] == 'succeeded'
    assert complete['result'] == retry['result']
    print(json.dumps({'decision_id': decision['decision_id'], 'receipt_id': decision['receipt_id'], 'replay_match': replay['structured_match'], 'action_id': action['id'], 'ticket': complete['result'], 'classification': 'generated_demo'}, indent=2))
if __name__ == '__main__': main()
