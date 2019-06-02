use rusty_usn::record::{UsnRecord, UsnEntry};
use rusty_usn::usn::{UsnParser, IntoIterFileChunks};

// Python Libs
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyIterProtocol;
use pyo3::types::PyString;
use pyo3::exceptions::RuntimeError;

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
/// PyUsnParser(self, path_or_file_like)
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
    fn new(obj: &PyRawObject, source_description: String, path_or_file_like: PyObject) -> PyResult<()> {
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
            source_description,
            boxed_read_seek
        )?;

        obj.init({
            PyUsnParser {
                inner: Some(usn_parser),
            }
        });

        Ok(())
    }

    /// records(self)
    /// --
    ///
    /// Returns PyRecordsIterator that yields records.
    fn records(&mut self) -> PyResult<PyRecordsIterator> {
        self.records_iterator()
    }
}

impl PyUsnParser {
    fn records_iterator(&mut self) -> PyResult<PyRecordsIterator> {
        let inner = match self.inner.take() {
            Some(inner) => inner,
            None => {
                return Err(PyErr::new::<RuntimeError, _>(
                    "PyUsnParser can only be used once",
                ));
            }
        };

        Ok(
            PyRecordsIterator {
                inner: inner.into_chunk_iterator(),
                records: None
            }
        )
    }
}


fn record_to_pydict(entry: UsnEntry, py: Python) -> PyResult<&PyDict> {
    let pyrecord = PyDict::new(py);

    pyrecord.set_item("_source", entry.source)?;
    pyrecord.set_item("_offset", entry.offset)?;
    match entry.record {
        UsnRecord::V2(usn_v2) => {
            pyrecord.set_item("record_length", usn_v2.record_length)?;
            pyrecord.set_item("major_version", usn_v2.major_version)?;
            pyrecord.set_item("minor_version", usn_v2.minor_version)?;

            let file_reference= PyDict::new(py);
            file_reference.set_item("entry", usn_v2.file_reference.entry)?;
            file_reference.set_item("sequence", usn_v2.file_reference.sequence)?;
            pyrecord.set_item("file_reference", file_reference)?;

            let parent_reference= PyDict::new(py);
            parent_reference.set_item("entry", usn_v2.parent_reference.entry)?;
            parent_reference.set_item("sequence", usn_v2.parent_reference.sequence)?;
            pyrecord.set_item("parent_reference", parent_reference)?;

            pyrecord.set_item("usn", usn_v2.usn)?;
            pyrecord.set_item("timestamp", format!("{}", usn_v2.timestamp))?;
            pyrecord.set_item("reason", format!("{}", usn_v2.reason))?;
            pyrecord.set_item("source_info", format!("{}", usn_v2.source_info))?;
            pyrecord.set_item("security_id", usn_v2.security_id)?;
            pyrecord.set_item("file_attributes", usn_v2.file_attributes)?;
            pyrecord.set_item("file_name_length", usn_v2.file_name_length)?;
            pyrecord.set_item("file_name_offset", usn_v2.file_name_offset)?;
            pyrecord.set_item("file_name", usn_v2.file_name)?;
        }
    }

    Ok(pyrecord)
}

fn record_to_pyobject(
    r: UsnEntry,
    py: Python,
) -> PyResult<PyObject> {
    match record_to_pydict(r, py) {
        Ok(dict) => Ok(dict.to_object(py)),
        Err(e) => Ok(e.to_object(py)),
    }
}


#[pyclass]
/// PyRecordsIterator(self)
/// --
///
/// An iterator that yields USN dictionaries.
pub struct PyRecordsIterator {
    inner: IntoIterFileChunks<Box<dyn ReadSeek>>,
    records: Option<Vec<UsnEntry>>
}

impl PyRecordsIterator {
    fn next(&mut self) -> PyResult<Option<PyObject>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        loop {
            if let Some(record) = self.records.as_mut().and_then(Vec::pop) {
                return record_to_pyobject(record, py).map(Some);
            }

            let data_chunk = self.inner.next();

            match data_chunk {
                None => return Ok(None),
                Some(data_chunk) => {
                    let record_iterator = data_chunk.get_record_iterator();
                    self.records = Some(record_iterator.collect());
                }
            }
        }
    }
}

#[pyproto]
impl PyIterProtocol for PyRecordsIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<PyRecordsIterator>> {
        Ok(slf.into())
    }
    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        slf.next()
    }
}


#[pymodule]
/// pyrustyusn
/// --
/// CLASSES
/// 
/// help(pyrustyusn.PyUsnParser)
/// help(pyrustyusn.PyRecordsIterator)
/// 
fn pyrustyusn(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUsnParser>()?;
    m.add_class::<PyRecordsIterator>()?;

    Ok(())
}
