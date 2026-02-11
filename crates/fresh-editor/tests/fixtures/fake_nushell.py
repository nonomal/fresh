#!/usr/bin/env python3
"""Fake shell that mimics nushell's terminal capability probing behavior.

On startup, nushell's line editor (reedline) via crossterm sends a DA1
(Primary Device Attributes) query to identify the terminal:

  \\x1b[c  →  expects  \\x1b[?...c  back

Without a response, crossterm blocks and the terminal appears frozen.

Before the fix (commit 62835565), fresh used a NullListener that silently
discarded all PtyWrite events from alacritty_terminal, so NO terminal queries
got responses — causing nushell (and other shells) to freeze on startup.

This script replicates that behavior: it switches stdin to raw mode (as
nushell does via reedline/crossterm), sends a DA1 query, and waits for the
response. If no response arrives within the timeout, it prints
FAKE_SHELL_STUCK_NO_RESPONSE and blocks — exactly like the bug in issue #884.

Used by the e2e test for #884 to verify fresh's PTY responds to \\x1b[c.
"""

import os
import sys
import select
import signal
import termios
import tty

TIMEOUT_SEC = 5


def read_with_timeout(fd, timeout):
    """Read from fd with a timeout. Returns data or empty bytes on timeout."""
    ready, _, _ = select.select([fd], [], [], timeout)
    if ready:
        return os.read(fd, 4096)
    return b""


def main():
    # Switch stdin to raw mode, just like nushell/reedline does via crossterm.
    # This is critical: without raw mode, the PTY buffers input until a
    # newline, so terminal capability responses (which have no newline)
    # never become readable via select().
    old_attrs = termios.tcgetattr(0)
    try:
        tty.setraw(0)

        # Send DA1 (Primary Device Attributes) query — this is one of the
        # queries that crossterm/reedline sends on startup. Before the
        # PtyWriteListener fix, all PtyWrite responses (including DA1) were
        # discarded by NullListener, causing shells to hang.
        os.write(1, b"\x1b[c")

        # Wait for the response: \x1b[?...c
        # nushell/crossterm blocks here until it gets the response.
        response = read_with_timeout(0, TIMEOUT_SEC)
    finally:
        # Restore terminal settings for normal I/O
        termios.tcsetattr(0, termios.TCSADRAIN, old_attrs)

    if not response:
        # No response — stay stuck (mimics the freeze)
        os.write(1, b"FAKE_SHELL_STUCK_NO_RESPONSE\r\n")
        # Block forever (like nushell does)
        signal.pause()
        sys.exit(1)

    # Got response — terminal answered the DA1 query.
    os.write(1, b"FAKE_SHELL_READY\r\n")
    os.write(1, b"$ ")

    # Simple read-eval loop
    buf = b""
    while True:
        data = read_with_timeout(0, 60)
        if not data:
            continue
        buf += data
        while b"\r" in buf or b"\n" in buf:
            idx = min(
                buf.index(b"\r") if b"\r" in buf else len(buf),
                buf.index(b"\n") if b"\n" in buf else len(buf),
            )
            line = buf[:idx].decode("utf-8", errors="replace").strip()
            buf = buf[idx + 1:]
            if line == "exit":
                sys.exit(0)
            os.write(1, f"{line}\r\n".encode())
            os.write(1, b"$ ")


if __name__ == "__main__":
    main()
