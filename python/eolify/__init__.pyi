from enum import Enum
from os import PathLike

class Mode(Enum):
    LF: Mode
    CRLF: Mode

def normalize_text(
    text: str,
    mode: Mode,
) -> str: ...
def normalize_file(
    input: str | PathLike[str],
    output: str | PathLike[str],
    mode: Mode,
    overwrite: bool = False,
) -> None: ...
