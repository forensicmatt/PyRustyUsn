use rusty_usn::usn::UsnParser;

// Python Libs
use pyo3::prelude::*;
use pyo3::types::PyString;

// Python Filelike
use pyo3_file::PyFileLikeObject;

// Standard Libs
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

// ReadSeak Trait Def
pub trait ReadSeek: Read + Seek {
    fn tell(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}
impl<T: Read + Seek> ReadSeek for T {}


// FileOrFileLike allows us to use a value of file path or file like object
#[derive(Debug)]
enum FileOrFileLike {
    File(String),
    FileLike(PyFileLikeObject),
}
impl FileOrFileLike {
    pub fn from_pyobject(path_or_file_like: PyObject) -> PyResult<FileOrFileLike> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // is a path
        if let Ok(string_ref) = path_or_file_like.cast_as::<PyString>(py) {
            return Ok(FileOrFileLike::File(
                string_ref.to_string_lossy().to_string(),
            ));
        }

        // We only need read + seek
        match PyFileLikeObject::with_requirements(path_or_file_like, true, false, true) {
            Ok(f) => Ok(FileOrFileLike::FileLike(f)),
            Err(e) => Err(e),
        }
    }
}


#[pyclass]
/// PyUsnParser(self, path_or_file_like, /)
/// --
///
/// Returns an instance of the parser.
/// Works on both a path (string), or a file-like object.
pub struct PyUsnParser {
    inner: Option<UsnParser<Box<dyn ReadSeek>>>,
}

#[pymethods]
impl PyUsnParser {
    #[new]
    fn new(obj: &PyRawObject, path_or_file_like: PyObject) -> PyResult<()> {
        let file_or_file_like = FileOrFileLike::from_pyobject(path_or_file_like)?;

        let boxed_read_seek = match file_or_file_like {
            FileOrFileLike::File(s) => {
                let file = File::open(s)?;
                Box::new(file) as Box<dyn ReadSeek>
            }
            FileOrFileLike::FileLike(f) => Box::new(f) as Box<dyn ReadSeek>,
        };

        // Create our usn parser
        let usn_parser = UsnParser::from_read_seek(
            boxed_read_seek
        )?;

        obj.init({
            PyUsnParser {
                inner: Some(usn_parser),
            }
        });

        Ok(())
    }
}


#[pymodule]
fn evtx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUsnParser>()?;

    Ok(())
}
