"""Dependency-free HTTP client for the versioned SAMI native API."""
from __future__ import annotations

import json
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlparse
from urllib.request import Request, HTTPRedirectHandler, build_opener


class SamiError(Exception):
    def __init__(self, message: str, status: int | None = None, details: Any = None):
        super().__init__(message)
        self.status = status
        self.details = details


class _NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class SamiClient:
    """Use an externally supplied scoped key/OIDC bearer token.

    The default timeout is 30 seconds. A timed-out mutation may have completed;
    inspect its resource/action state and reconcile before a deliberate retry.
    """

    def __init__(self, base_url: str = "http://127.0.0.1:8080/v1", token: str = "", timeout: float = 30):
        parsed = urlparse(base_url)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc:
            raise ValueError("base_url must be an absolute HTTP(S) API URL")
        if parsed.username or parsed.password or parsed.query or parsed.fragment:
            raise ValueError("base_url must not contain credentials, query or fragment")
        if parsed.scheme == "http" and parsed.hostname not in {"localhost", "127.0.0.1", "::1"}:
            raise ValueError("Use HTTPS for a remote API credential")
        if timeout <= 0:
            raise ValueError("timeout must be positive")
        self.base_url = base_url.rstrip("/")
        self._token = token
        self.timeout = timeout
        self._opener = build_opener(_NoRedirect)

    def request(self, method: str, path: str, body: Any = None, *, idempotency_key: str | None = None) -> Any:
        if not path.startswith("/") or "?" in path or "#" in path:
            raise ValueError("path must be a relative API path without query or fragment")
        headers = {"Accept": "application/json"}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        data = None
        if body is not None:
            data = json.dumps(body, allow_nan=False).encode("utf-8")
            headers["Content-Type"] = "application/json"
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        request = Request(self.base_url + path, data=data, headers=headers, method=method)
        try:
            with self._opener.open(request, timeout=self.timeout) as response:
                raw = response.read()
                return json.loads(raw) if raw else None
        except HTTPError as error:
            raw = error.read()
            try:
                details = json.loads(raw)
            except (ValueError, UnicodeDecodeError):
                details = {"message": raw.decode("utf-8", errors="replace")[:500]}
            nested = details.get("error", {}) if isinstance(details, dict) else {}
            message = nested.get("message") if isinstance(nested, dict) else None
            if not message and isinstance(details, dict):
                message = details.get("message")
            raise SamiError(message or f"SAMI request failed ({error.code})", error.code, details) from error
        except (URLError, TimeoutError, OSError) as error:
            raise SamiError("Transport failed. A mutation may have completed; inspect/reconcile before retrying.") from error

    @staticmethod
    def _id(value: str) -> str:
        if not value:
            raise ValueError("identifier must not be empty")
        return quote(value, safe="")

    def decide(self, task_id: str, input: dict[str, Any], pack_id: str = "industrial-service") -> Any:
        return self.request("POST", "/decisions", {"task_id": task_id, "input": input, "pack_id": pack_id})

    def create_session(self, state: dict[str, Any] | None = None) -> Any:
        return self.request("POST", "/sessions", {"state": state or {}})

    def respond(self, session_id: str, message: str, revision: int | None = None) -> Any:
        body: dict[str, Any] = {"session_id": session_id, "message": message}
        if revision is not None:
            body["revision"] = revision
        return self.request("POST", "/respond", body)

    def list_resources(self, kind: str) -> Any:
        return self.request("GET", f"/admin/{self._id(kind)}")

    def get_resource(self, kind: str, resource_id: str) -> Any:
        return self.request("GET", f"/admin/{self._id(kind)}/{self._id(resource_id)}")

    def save_resource(self, kind: str, record: dict[str, Any]) -> Any:
        return self.request("POST", f"/admin/{self._id(kind)}", record)

    def publish_source(self, source_id: str, reason: str) -> Any:
        return self.request("POST", f"/sources/{self._id(source_id)}/publish", {"reason": reason})

    def retract_source(self, source_id: str, reason: str) -> Any:
        return self.request("POST", f"/sources/{self._id(source_id)}/retract", {"reason": reason})

    def delete_source(self, source_id: str) -> Any:
        return self.request("DELETE", f"/sources/{self._id(source_id)}")

    def receipt(self, receipt_id: str) -> Any:
        return self.request("GET", f"/receipts/{self._id(receipt_id)}")

    def replay(self, receipt_id: str) -> Any:
        return self.request("POST", f"/receipts/{self._id(receipt_id)}/replay", {})

    def propose_action(self, decision_id: str, tool_id: str, arguments: dict[str, Any]) -> Any:
        return self.request("POST", "/actions/propose", {"decision_id": decision_id, "tool_id": tool_id, "arguments": arguments})

    def approve_action(self, action_id: str, reason: str, expires_in_seconds: int = 300) -> Any:
        return self.request("POST", f"/actions/{self._id(action_id)}/approve", {"reason": reason, "expires_in_seconds": expires_in_seconds})

    def execute_action(self, action_id: str, idempotency_key: str) -> Any:
        if not idempotency_key:
            raise ValueError("execution requires an idempotency key")
        return self.request("POST", f"/actions/{self._id(action_id)}/execute", {}, idempotency_key=idempotency_key)

    def reconcile_action(self, action_id: str) -> Any:
        return self.request("POST", f"/actions/{self._id(action_id)}/reconcile", {})

    def feedback(self, event: dict[str, Any]) -> Any:
        return self.request("POST", "/feedback", event)

    def research(self, operation: str, input: dict[str, Any]) -> Any:
        return self.request("POST", f"/research/{self._id(operation)}", input)

    def create_key(self, principal: str, scopes: list[str], groups: list[str] | None = None) -> Any:
        return self.request("POST", "/keys", {"principal": principal, "scopes": scopes, "groups": groups or []})

    def revoke_key(self, key_id: str) -> Any:
        return self.request("POST", f"/keys/{self._id(key_id)}/revoke", {})
