//! Python exception adaptation only. Neutral definitions live in core::errors.
use crate::core::errors::{RegistryError, RuntimeError};
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use std::{io, path::Path};

fn identify(py: Python<'_>, error: PyErr, case: &str) -> PyResult<PyErr> {
    error.value(py).setattr("case", case)?;
    error.value(py).setattr("__suppress_context__", true)?;
    Ok(error)
}

pub(crate) fn registry(py: Python<'_>, error: RegistryError) -> PyResult<PyErr> {
    let exception = identify(
        py,
        io::Error::new(error.kind(), error.to_string()).into(),
        error.name(),
    )?;
    exception
        .value(py)
        .setattr("path", error.path().as_os_str())?;
    Ok(exception)
}

pub(crate) fn runtime(py: Python<'_>, error: RuntimeError, path: &Path) -> PyResult<PyErr> {
    match error {
        RuntimeError::Load(error) => {
            let exception = identify(
                py,
                io::Error::new(error.kind(), error.to_string()).into(),
                error.name(),
            )?;
            exception
                .value(py)
                .setattr("path", error.path().as_os_str())?;
            Ok(exception)
        }
        RuntimeError::Output(error) => identify(
            py,
            io::Error::new(error.kind, error.to_string()).into(),
            "OutputWriteFailed",
        ),
        RuntimeError::Tokenize(error) => {
            let exception = identify(
                py,
                PyValueError::new_err(format!("{}:{}: {error}", path.display(), error.line())),
                error.name(),
            )?;
            exception.value(py).setattr("path", path.as_os_str())?;
            exception.value(py).setattr("line", error.line())?;
            Ok(exception)
        }
        RuntimeError::Handoff(failure) => {
            let exception = identify(
                py,
                PyRuntimeError::new_err(failure.to_string()),
                failure.error.name(),
            )?;
            let value = exception.value(py);
            value.setattr("role_name", failure.name)?;
            value.setattr("index", failure.index)?;
            value.setattr("role_index", failure.role_index)?;
            value.setattr("declaration_line", failure.declaration_line)?;
            value.setattr("target", failure.target.as_os_str())?;
            if let crate::core::errors::HandoffError::UnknownBridge(identifier) = failure.error {
                value.setattr("identifier", identifier)?;
            }
            Ok(exception)
        }
    }
}

/// Only Core-owned preparation/allocation, never calls into received Role objects.
pub(crate) fn project<T>(py: Python<'_>, result: PyResult<T>, path: &Path) -> PyResult<T> {
    let original = match result {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    // Process-control exceptions remain Python control flow.
    if !original.is_instance_of::<pyo3::exceptions::PyException>(py) {
        return Err(original);
    }
    drop(original);
    let failure = crate::core::errors::ProjectConstructionFailed;
    let error = identify(
        py,
        PyRuntimeError::new_err(failure.to_string()),
        "ProjectConstructionFailed",
    )?;
    error
        .value(py)
        .setattr("requested_file", path.as_os_str())?;
    Err(error)
}
