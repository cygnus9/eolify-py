from collections.abc import Callable
from os import PathLike
from typing import Final, Protocol, TypeAlias, final

class Source(Protocol):
    def read(self, size: int, /) -> bytes: ...

class Sink(Protocol):
    def write(self, data: bytes, /) -> int: ...

ReadCallback: TypeAlias = Callable[[int], bytes]
WriteCallback: TypeAlias = Callable[[bytes], int]

Input: TypeAlias = Source | ReadCallback
Output: TypeAlias = Sink | WriteCallback

@final
class Mode:
    CRLF: Final[Mode]
    LF: Final[Mode]
    def __eq__(self, /, other: Mode | int) -> bool: ...
    def __int__(self, /) -> int: ...
    def __ne__(self, /, other: Mode | int) -> bool: ...
    def __repr__(self, /) -> str: ...

def normalize_file(
    input: str | PathLike[str],
    output: str | PathLike[str],
    mode: Mode,
    overwrite: bool = False,
) -> None: ...
def normalize_stream(input: Input, output: Output, mode: Mode) -> None: ...
def normalize_text(text: str, mode: Mode) -> str: ...
