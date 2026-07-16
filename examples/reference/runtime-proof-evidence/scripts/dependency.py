# Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
# Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.

from http.server import BaseHTTPRequestHandler, HTTPServer

marker = b""


class Handler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:
        global marker
        marker = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.send_response(204)
        self.end_headers()

    def do_GET(self) -> None:
        self.send_response(200 if marker else 404)
        self.end_headers()
        self.wfile.write(marker)

    def log_message(self, _format: str, *_args: object) -> None:
        pass


HTTPServer(("0.0.0.0", 8081), Handler).serve_forever()
