"""Release contract: no stale embedded assets, omitted PRD IDs or missing routes."""
from pathlib import Path
import json
import re

root = Path(__file__).resolve().parents[2]
for a, b in [("packs/industrial-service/pack.json", "console/src/default-pack.json"),
             ("examples/research/operations.json", "console/src/research-examples.json")]:
    assert json.loads((root/a).read_text()) == json.loads((root/b).read_text()), f"Stale embedded asset: {b}"
requirements = json.loads((root/"docs/requirements.json").read_text())["requirements"]
ids = re.findall(r"^\s*(?:- )?\*\*([A-Z]+-\d+)(?:, (?:P\d|R))?:\*\*", (root/"docs/PRD.md").read_text(), re.M)
assert len(ids) == len(set(ids)), "Duplicate PRD IDs"
assert sorted(ids) == sorted(r["id"] for r in requirements), "PRD requirement coverage differs"
for requirement in requirements:
    assert requirement["status"] in {"Functional", "Bounded", "Partial", "External gate"}
    assert requirement["evidence_or_remaining"]
    assert all((root/p).exists() for p in requirement["implementation"])
actual = set()
for path, handlers in re.findall(r'\.route\("([^"\n]+)", ([^\n]+)\)', (root/"src/api.rs").read_text()):
    path = re.sub(r":(\w+)", r"{\1}", path)
    for method in ("get", "post", "delete"):
        if re.search(r"\b"+method+r"\(", handlers):
            actual.add((path, method))
spec = json.loads((root/"docs/openapi.json").read_text())
documented = {(path, method) for path, operations in spec["paths"].items() for method in operations}
assert actual == documented, f"OpenAPI differs: {actual ^ documented}"
print(f"Release contracts passed: {len(ids)} PRD IDs, {len(actual)} route operations, matching embedded assets.")
