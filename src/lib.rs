use pyo3::pymodule;

#[pymodule(name = "eolify")]
mod eolify_py {
    use std::{fs, io, path};

    use eolify::{IoExt, Normalize};
    use pyo3::{PyResult, Python, pyclass, pyfunction};

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
}
