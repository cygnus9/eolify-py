use std::io;

use pyo3::{
    Py, PyAny, Python, pymodule,
    types::{PyAnyMethods, PyBytes, PyBytesMethods},
};

struct PyReader {
    read: Py<PyAny>,
}

impl io::Read for PyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let result = self
                .read
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

            let written = self
                .write
                .bind(py)
                .call1((data,))
                .and_then(|r| r.extract::<usize>())
                .map_err(|e| io::Error::other(e.to_string()))?;

            if written > buf.len() {
                return Err(io::Error::other(
                    "Python stream wrote more bytes than requested",
                ));
            }

            Ok(written)
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
    use pyo3::{
        Bound, FromPyObject, PyAny, PyResult, Python, exceptions::PyValueError, pyclass,
        pyfunction, types::PyAnyMethods,
    };

    use crate::{PyReader, PyWriter};

    #[pyclass(eq, eq_int, from_py_object)]
    #[derive(Clone, Copy, PartialEq)]
    #[allow(clippy::upper_case_acronyms)]
    /// Line endings normalization mode.
    enum Mode {
        LF,
        CRLF,
    }

    #[derive(FromPyObject)]
    enum ModeArg {
        Mode(Mode),
        Str(String),
    }

    impl ModeArg {
        fn into_mode(self) -> PyResult<Mode> {
            match self {
                Self::Mode(mode) => Ok(mode),
                Self::Str(s) => match s.as_str() {
                    "\n" => Ok(Mode::LF),
                    "\r\n" => Ok(Mode::CRLF),
                    _ => Err(PyValueError::new_err(
                        "mode must be Mode.LF, Mode.CRLF, '\\n', or '\\r\\n'",
                    )),
                },
            }
        }
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
    fn normalize_text(text: &str, mode: ModeArg) -> PyResult<String> {
        match mode.into_mode()? {
            Mode::LF => Ok(eolify::LF::normalize_str(text)),
            Mode::CRLF => Ok(eolify::CRLF::normalize_str(text)),
        }
    }

    #[pyfunction]
    #[pyo3(signature = (source, destination, mode, overwrite = false))]
    /// Normalize line endings from a file
    ///
    /// Args:
    ///     source: Path to source file
    ///     destination: Path destination file
    ///     mode: Desired line ending style
    ///     overwrite: Allow existing output files to be overwritten (default = False)
    fn normalize_file(
        py: Python<'_>,
        source: path::PathBuf,
        destination: path::PathBuf,
        mode: ModeArg,
        overwrite: bool,
    ) -> PyResult<()> {
        let mode = mode.into_mode()?;

        py.detach(|| {
            let input = fs::File::open(source)?;
            let mut output = if overwrite {
                fs::File::create(destination)?
            } else {
                fs::File::create_new(destination)?
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
    #[pyo3(signature = (source, destination, mode))]
    fn normalize_stream(
        py: Python<'_>,
        source: Bound<PyAny>,
        destination: Bound<PyAny>,
        mode: ModeArg,
    ) -> PyResult<()> {
        let mode = mode.into_mode()?;

        let reader = if source.is_callable() {
            PyReader {
                read: source.unbind(),
            }
        } else {
            PyReader {
                read: source.getattr("read")?.unbind(),
            }
        };

        let mut writer = if destination.is_callable() {
            PyWriter {
                write: destination.unbind(),
                flush: None,
            }
        } else {
            PyWriter {
                write: destination.getattr("write")?.unbind(),
                flush: destination.getattr_opt("flush")?.map(Bound::unbind),
            }
        };

        py.detach(|| {
            let mut reader: Box<dyn io::Read> = match mode {
                Mode::LF => Box::new(eolify::LF::wrap_reader(reader)),
                Mode::CRLF => Box::new(eolify::CRLF::wrap_reader(reader)),
            };

            io::copy(&mut reader, &mut writer)?;
            writer.flush()?;
            Ok(())
        })
    }
}
