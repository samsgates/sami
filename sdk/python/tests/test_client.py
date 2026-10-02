import json
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from sami_client import SamiClient, SamiError


class Handler(BaseHTTPRequestHandler):
    seen = []

    def log_message(self, *args):
        pass

    def do_GET(self):
        self.seen.append((self.path, self.headers.get("Authorization"), None, {}))
        self.send_response(302)
        self.send_header("Location", "/sink")
        self.end_headers()

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))) or b"{}")
        self.seen.append((self.path, self.headers.get("Authorization"), self.headers.get("Idempotency-Key"), body))
        denied = self.path.endswith("/approve")
        self.send_response(409 if denied else 200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps({"error": {"message": "Changed policy"}} if denied else {"id": "action-1"}).encode())


class ClientTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    def setUp(self):
        Handler.seen.clear()
        self.client = SamiClient(f"http://127.0.0.1:{self.server.server_port}/v1", "supplied-token")

    def test_authenticated_payload_and_encoded_id(self):
        self.client.execute_action("a/b", "stable-key")
        path, auth, key, body = Handler.seen[0]
        self.assertEqual(path, "/v1/actions/a%2Fb/execute")
        self.assertEqual(auth, "Bearer supplied-token")
        self.assertEqual(key, "stable-key")
        self.assertEqual(body, {})

    def test_rejection_is_not_retried(self):
        with self.assertRaises(SamiError) as error:
            self.client.approve_action("action-1", "reviewed")
        self.assertEqual(error.exception.status, 409)
        self.assertEqual(len(Handler.seen), 1)

    def test_redirect_is_not_followed(self):
        with self.assertRaises(SamiError) as error:
            self.client.request("GET", "/redirect")
        self.assertEqual(error.exception.status, 302)
        self.assertEqual(len(Handler.seen), 1)
        with self.assertRaises(ValueError):
            SamiClient("http://remote.example/v1", "generated-test-only")

    def test_key_and_url_validation(self):
        with self.assertRaises(ValueError):
            self.client.execute_action("a", "")
        with self.assertRaises(ValueError):
            SamiClient("https://secret@example.com/v1")


if __name__ == "__main__":
    unittest.main()
