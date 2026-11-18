#!/usr/bin/env python3
"""Static dev server for the game, with caching disabled.

`python3 -m http.server` answers conditional requests from its Last-Modified
headers, so the browser happily serves stale textures after a rebuild - which
makes visual QA report the previous build's art. This serves the same tree but
tells the browser never to store anything, so a reload always shows what is
actually on disk.

    python3 assets/tools/devserver.py [port]     # default 5014
"""
import functools
import http.server
import os
import socketserver
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


class NoCacheHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")
        super().end_headers()

    def log_message(self, fmt, *args):
        pass  # quiet: this runs alongside the game, not as a service


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 5014
    handler = functools.partial(NoCacheHandler, directory=ROOT)
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.ThreadingTCPServer(("127.0.0.1", port), handler) as httpd:
        print(f"serving {ROOT} on http://127.0.0.1:{port}/play.html (no cache)")
        httpd.serve_forever()


if __name__ == "__main__":
    main()
