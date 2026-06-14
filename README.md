# eolify

High-performance line ending normalization for Python, powered by Rust.

`eolify` provides fast and memory-efficient conversion between LF (`\n`) and CRLF (`\r\n`) line endings for text, files, and streams.

## Features

* Fast Rust implementation
* Supports LF and CRLF normalization
* Memory-efficient file processing
* Cross-platform
* Typed API with Python type hints

## Installation

```bash
pip install eolify
```

Pre-built wheels are available for common platforms, so a Rust toolchain is usually not required.

## Quick Start

### Normalize text

```python
import eolify

text = "hello\r\nworld\r\n"

normalized = eolify.normalize_text(
    text,
    eolify.Mode.LF,
)

print(repr(normalized))
# 'hello\nworld\n'
```

### Normalize a file

```python
import eolify

eolify.normalize_file(
    "input.txt",
    "output.txt",
    eolify.Mode.LF,
)
```

By default, the destination file must not already exist.

```python
eolify.normalize_file(
    "input.txt",
    "output.txt",
    eolify.Mode.LF,
    overwrite=True,
)
```

### Normalize a stream

```python
import eolify

with open("input.txt", "rb") as source:
    with open("output.txt", "wb") as destination:
        eolify.normalize_stream(
            source,
            destination,
            eolify.Mode.LF,
        )
```

The source can also be a read callback and the destination can be a write callback.

```python
import eolify

chunks = [b"hello\r\n", b"world\r\n", b""]
output = bytearray()

def read(size: int) -> bytes:
    return chunks.pop(0)

def write(data: bytes) -> int:
    output.extend(data)
    return len(data)

eolify.normalize_stream(read, write, eolify.Mode.LF)

print(bytes(output))
# b'hello\nworld\n'
```

## API

### `normalize_text(text, mode) -> str`

Normalize line endings in a string.

Parameters:

* `text`: Input text.
* `mode`: `Mode.LF` or `Mode.CRLF`.

Returns:

* The normalized string.

### `normalize_file(source, destination, mode, overwrite=False) -> None`

Normalize line endings while copying one file to another.

Parameters:

* `source`: Source file path.
* `destination`: Destination file path.
* `mode`: `Mode.LF` or `Mode.CRLF`.
* `overwrite`: Whether an existing destination file may be replaced.

### `normalize_stream(source, destination, mode) -> None`

Normalize line endings while copying from one binary source to another.

Parameters:

* `source`: Binary file-like object with `read(size) -> bytes`, or a read callback with the same signature.
* `destination`: Binary file-like object with `write(data) -> int`, or a write callback with the same signature.
  If a destination object provides `flush()`, it is called after copying.
* `mode`: `Mode.LF` or `Mode.CRLF`.

## Modes

```python
eolify.Mode.LF
eolify.Mode.CRLF
```

## Why eolify?

Python's built-in newline handling is often sufficient, but there are situations where explicit normalization is desirable:

* Converting files between platforms
* Preparing source archives
* Processing generated files
* Build and CI pipelines
* Working with mixed line endings

`eolify` uses the same Rust implementation as the [eolify crate](https://crates.io/crates/eolify), providing predictable
and efficient normalization behavior.
