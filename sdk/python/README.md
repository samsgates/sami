# SAMI Python client

Requires Python 3.10+. From the SAMI repository root run `python3 -m pip install ./sdk/python`. The client uses the standard library and has no runtime dependencies.

```python
import os
from sami_client import SamiClient

client = SamiClient("https://your-reviewed-deployment.example/v1", os.environ["SAMI_API_KEY"])
result = client.decide("service_triage", {"message": "Your case description", "asset_id": "Your confirmed asset"})
print(result["status"])
```

Configure a real trusted deployment origin and scoped secret. The client rejects cleartext remote origins and redirects. It never automatically retries a mutation. Save action IDs and stable execution keys; reconcile ambiguous effects. `SamiError` exposes status and error details/request ID. See the repository's `docs/sdk.md`, `docs/api.md` and runnable `examples/complete_case.py`.

Run package tests with `python3 -m unittest discover -s tests -v` from this directory. Apache-2.0; source/data scope follows the parent repository policy.
