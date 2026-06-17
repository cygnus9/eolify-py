from io import BytesIO, StringIO

import eolify
import pytest


class ChunkedReader:
    def __init__(self, data: bytes, chunk_size: int):
        self._data = data
        self._chunk_size = chunk_size
        self._position = 0

    def read(self, size: int = -1) -> bytes:
        if self._position >= len(self._data):
            return b""

        if size < 0:
            size = len(self._data) - self._position

        size = min(size, self._chunk_size)
        start = self._position
        self._position += size
        return self._data[start : self._position]


class FlushingWriter(BytesIO):
    def __init__(self):
        super().__init__()
        self.flush_count = 0

    def flush(self):
        self.flush_count += 1
        super().flush()


def test_normalize_stream_to_lf():
    source = BytesIO(b"one\r\ntwo\rthree\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\ntwo\nthree\n"


def test_normalize_stream_to_lf_by_string():
    source = BytesIO(b"one\r\ntwo\rthree\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, "\n")

    assert destination.getvalue() == b"one\ntwo\nthree\n"


def test_normalize_stream_to_crlf():
    source = BytesIO(b"one\ntwo\rthree\r\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.CRLF)

    assert destination.getvalue() == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_stream_to_crlf_by_string():
    source = BytesIO(b"one\ntwo\rthree\r\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, "\r\n")

    assert destination.getvalue() == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_stream_handles_line_endings_across_read_boundaries():
    source = ChunkedReader(b"one\r\ntwo\rthree\n", chunk_size=1)
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\ntwo\nthree\n"


def test_normalize_stream_flushes_output():
    source = BytesIO(b"one\r\n")
    destination = FlushingWriter()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\n"
    assert destination.flush_count == 1


def test_normalize_stream_accepts_read_callback():
    source = ChunkedReader(b"one\r\ntwo\rthree\n", chunk_size=1)
    destination = BytesIO()
    requested_sizes: list[int] = []

    def read(size: int) -> bytes:
        requested_sizes.append(size)
        return source.read(size)

    eolify.normalize_stream(read, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\ntwo\nthree\n"
    assert requested_sizes
    assert all(size > 0 for size in requested_sizes)


def test_normalize_stream_accepts_write_callback():
    source = BytesIO(b"one\ntwo\rthree\r\n")
    chunks: list[bytes] = []

    def write(data: bytes) -> int:
        chunks.append(data)
        return len(data)

    eolify.normalize_stream(source, write, eolify.Mode.CRLF)

    assert b"".join(chunks) == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_stream_accepts_read_and_write_callbacks():
    source = ChunkedReader(b"one\r\ntwo\rthree\n", chunk_size=2)
    chunks: list[bytes] = []

    def read(size: int) -> bytes:
        return source.read(size)

    def write(data: bytes) -> int:
        chunks.append(data)
        return len(data)

    eolify.normalize_stream(read, write, eolify.Mode.LF)

    assert b"".join(chunks) == b"one\ntwo\nthree\n"


def test_normalize_stream_rejects_write_callback_that_returns_too_many_bytes():
    def write(data: bytes) -> int:
        return len(data) + 1

    with pytest.raises(OSError, match="wrote more bytes"):
        eolify.normalize_stream(BytesIO(b"one\r\n"), write, eolify.Mode.LF)


def test_normalize_stream_requires_binary_input():
    source = StringIO("one\r\n")
    destination = BytesIO()

    with pytest.raises(OSError):
        eolify.normalize_stream(source, destination, eolify.Mode.LF)  # pyright: ignore[reportArgumentType]


def test_normalize_stream_rejects_invalid_mode_string():
    source = BytesIO(b"one\r\n")
    destination = BytesIO()

    with pytest.raises(ValueError):
        eolify.normalize_stream(source, destination, "\r")  # pyright: ignore[reportArgumentType]
