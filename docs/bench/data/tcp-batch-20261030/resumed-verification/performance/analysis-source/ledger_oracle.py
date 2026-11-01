"""Expected terminal body from bytes actually produced, including skipped IDs."""
import base64


class LedgerSource:
    def __init__(self, frames, cols, rows):
        import pyte
        screen = pyte.Screen(cols, rows)
        stream = pyte.ByteStream(screen)
        stream.feed(b"\x1b[?7l")
        self.frames, self.missing = {}, set()
        for frame in frames:
            stream.feed(base64.b64decode(frame["payload_b64"], validate=True))
            self.frames[frame["n"]] = tuple(screen.display[1:])

    def body(self, n):
        if n not in self.frames:
            self.missing.add(n)
        return self.frames.get(n)
