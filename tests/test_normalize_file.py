from pathlib import Path

import eolify
import pytest


def test_normalize_file_to_lf(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\r\ntwo\rthree\n")

    eolify.normalize_file(source, destination, eolify.Mode.LF)

    assert destination.read_bytes() == b"one\ntwo\nthree\n"


def test_normalize_file_to_lf_by_string(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\r\ntwo\rthree\n")

    eolify.normalize_file(source, destination, "\n")

    assert destination.read_bytes() == b"one\ntwo\nthree\n"


def test_normalize_file_to_crlf(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\ntwo\rthree\r\n")

    eolify.normalize_file(source, destination, eolify.Mode.CRLF)

    assert destination.read_bytes() == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_file_to_crlf_by_string(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\ntwo\rthree\r\n")

    eolify.normalize_file(source, destination, "\r\n")

    assert destination.read_bytes() == b"one\r\ntwo\r\nthree\r\n"


def test_normalize_file_refuses_existing_output_by_default(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\r\n")
    destination.write_bytes(b"existing")

    with pytest.raises(FileExistsError):
        eolify.normalize_file(source, destination, eolify.Mode.LF)

    assert destination.read_bytes() == b"existing"


def test_normalize_file_can_overwrite_existing_output(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\r\n")
    destination.write_bytes(b"existing")

    eolify.normalize_file(source, destination, eolify.Mode.LF, overwrite=True)

    assert destination.read_bytes() == b"one\n"


def test_normalize_file_rejects_invalid_mode_string(tmp_path: Path):
    source = tmp_path / "source.txt"
    destination = tmp_path / "destination.txt"
    source.write_bytes(b"one\r\n")
    destination.write_bytes(b"existing")

    with pytest.raises(ValueError):
        eolify.normalize_file(source, destination, "\r")  # pyright: ignore[reportArgumentType]

    assert destination.read_bytes() == b"existing"
