#!/usr/bin/env python3
"""Fake shell that mimics nushell's terminal capability probing behavior.

On startup, nushell's line editor (reedline) sends a kitty keyboard protocol
query to the terminal and blocks until it gets a response:

  \\x1b[?u  →  expects  \\x1b[?{mode}u  back

Without a response, reedline blocks and the terminal appears frozen.

This script replicates that behavior: it switches stdin to raw mode (as
nushell does via reedline/crossterm), sends the kitty keyboard query, and
waits for the response. If no response arrives within the timeout, it prints
FAKE_SHELL_STUCK_NO_RESPONSE and blocks — exactly like the bug in issue #884.

Used by the e2e test for #884 to verify fresh's PTY responds to \\x1b[?u.
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

        # Send the kitty keyboard protocol query — the specific query that
        # was broken before the fix (alacritty_terminal only responds when
        # its config has kitty_keyboard=true).
        os.write(1, b"\x1b[?u")

        # Wait for the response: \x1b[?{mode}u
        # nushell blocks here until it gets the response.
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

    # Got response — terminal supports kitty keyboard protocol query.
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
