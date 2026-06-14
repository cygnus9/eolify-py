from io import BytesIO, StringIO

import pytest

import eolify


class ChunkedReader:
    def __init__(self, data: bytes, chunk_size: int) -> None:
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
    def __init__(self) -> None:
        super().__init__()
        self.flush_count = 0

    def flush(self) -> None:
        self.flush_count += 1
        super().flush()


def test_normalize_stream_to_lf() -> None:
    source = BytesIO(b"one\r\ntwo\rthree\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\ntwo\nthree\n"


def test_normalize_stream_to_crlf() -> None:
    source = BytesIO(b"one\ntwo\rthree\r\n")
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.CRLF)

    assert destination.getvalue() == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_stream_handles_line_endings_across_read_boundaries() -> None:
    source = ChunkedReader(b"one\r\ntwo\rthree\n", chunk_size=1)
    destination = BytesIO()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\ntwo\nthree\n"


def test_normalize_stream_flushes_output() -> None:
    source = BytesIO(b"one\r\n")
    destination = FlushingWriter()

    eolify.normalize_stream(source, destination, eolify.Mode.LF)

    assert destination.getvalue() == b"one\n"
    assert destination.flush_count == 1


def test_normalize_stream_requires_binary_input() -> None:
    source = StringIO("one\r\n")
    destination = BytesIO()

    with pytest.raises(OSError):
        eolify.normalize_stream(source, destination, eolify.Mode.LF)
