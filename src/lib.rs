use std::io;

use pyo3::{
    Py, PyAny, Python, pymodule,
    types::{PyAnyMethods, PyBytes, PyBytesMethods},
};

struct PyReader {
    obj: Py<PyAny>,
}

impl io::Read for PyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let result = self
                .obj
                .bind(py)
                .call1((buf.len(),))
                .map_err(|e| io::Error::other(e.to_string()))?;

            let src = result
                .cast::<PyBytes>()
                .map_err(|e| io::Error::other(e.to_string()))?
                .as_bytes();

            if src.len() > buf.len() {
                return Err(io::Error::other(
                    "Python stream returned more bytes than requested",
                ));
            }

            buf[..src.len()].copy_from_slice(src);

            Ok(src.len())
        })
    }
}

struct PyWriter {
    write: Py<PyAny>,
    flush: Option<Py<PyAny>>,
}

impl io::Write for PyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let data = PyBytes::new(py, buf);

            self.write
                .bind(py)
                .call1((data,))
                .and_then(|r| r.extract::<usize>())
                .map_err(|e| io::Error::other(e.to_string()))
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Python::attach(|py| {
            if let Some(flush) = &self.flush {
                flush
                    .call0(py)
                    .map_err(|e| io::Error::other(e.to_string()))?;
            }

            Ok(())
        })
    }
}

#[pymodule(name = "eolify")]
mod eolify_py {
    use std::{
        fs,
        io::{self, Write},
        path,
    };

    use eolify::{IoExt, Normalize};
    use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pyfunction, types::PyAnyMethods};

    use crate::{PyReader, PyWriter};

    #[pyclass(eq, eq_int, from_py_object)]
    #[derive(Clone, Copy, PartialEq)]
    #[allow(clippy::upper_case_acronyms)]
    /// Line endings normalization mode.
    enum Mode {
        LF,
        CRLF,
    }

    #[pyfunction]
    #[pyo3(signature = (text, mode))]
    /// Normalize line endings in text
    ///
    /// Args:
    ///     text: Input text
    ///     mode: Desired line ending style
    ///
    /// Returns:
    ///     Normalized text
    fn normalize_text(text: &str, mode: Mode) -> String {
        match mode {
            Mode::LF => eolify::LF::normalize_str(text),
            Mode::CRLF => eolify::CRLF::normalize_str(text),
        }
    }

    #[pyfunction]
    #[pyo3(signature = (input, output, mode, overwrite = false))]
    /// Normalize line endings from a file
    ///
    /// Args:
    ///     input: Path to source file
    ///     output: Path destination file
    ///     mode: Desired line ending style
    ///     overwrite: Allow existing output files to be overwritten (default = False)
    fn normalize_file(
        py: Python<'_>,
        input: path::PathBuf,
        output: path::PathBuf,
        mode: Mode,
        overwrite: bool,
    ) -> PyResult<()> {
        py.detach(|| {
            let input = fs::File::open(input)?;
            let mut output = if overwrite {
                fs::File::create(output)?
            } else {
                fs::File::create_new(output)?
            };

            let mut input: Box<dyn io::Read> = match mode {
                Mode::LF => Box::new(eolify::LF::wrap_reader(input)),
                Mode::CRLF => Box::new(eolify::CRLF::wrap_reader(input)),
            };

            io::copy(&mut input, &mut output)?;
            Ok(())
        })
    }

    #[pyfunction]
    #[pyo3(signature = (input, output, mode))]
    fn normalize_stream(
        py: Python<'_>,
        input: &Bound<PyAny>,
        output: &Bound<PyAny>,
        mode: Mode,
    ) -> PyResult<()> {
        let input = PyReader {
            obj: input.getattr("read")?.unbind(),
        };
        let mut output = PyWriter {
            write: output.getattr("write")?.unbind(),
            flush: output.getattr_opt("flush")?.map(Bound::unbind),
        };

        py.detach(|| {
            let mut input: Box<dyn io::Read> = match mode {
                Mode::LF => Box::new(eolify::LF::wrap_reader(input)),
                Mode::CRLF => Box::new(eolify::CRLF::wrap_reader(input)),
            };

            io::copy(&mut input, &mut output)?;
            output.flush()?;
            Ok(())
        })
    }
}
