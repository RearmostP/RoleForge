// The public Python boundary adapts Core results; receiving calls belong to Bridges.
use std::{io, path::PathBuf};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyTuple,
};

use crate::core::{
    pipeline::{
        dispatcher::DispatchResult,
        runtime::{RuntimeError, run_file},
        tokenizer::TokenizeError,
    },
    registry::Registry,
};

/// Neutral discovery metadata, not a Role implementation or executable API.
#[pyclass(frozen, get_all, module = "roleforge")]
struct RoleInfo {
    name: String,
    index: usize,
    role_index: usize,
    body: String,
    declaration_line: usize,
    status: &'static str,
    entry: Option<PathBuf>,
    builtin_entry: Option<PathBuf>,
    dynamic_entry: Option<PathBuf>,
}

impl From<DispatchResult> for RoleInfo {
    fn from(result: DispatchResult) -> Self {
        let (role, status, entry, builtin_entry, dynamic_entry) = match result {
            DispatchResult::Resolved { role, entry } => {
                (role, "resolved", Some(entry.target), None, None)
            }
            DispatchResult::Unknown { role } => (role, "unknown", None, None, None),
            DispatchResult::Conflict {
                role,
                builtin_entry,
                dynamic_entry,
            } => (
                role,
                "conflict",
                None,
                Some(builtin_entry.target),
                Some(dynamic_entry.target),
            ),
        };
        Self {
            name: role.name,
            index: role.index,
            role_index: role.role_index,
            body: role.body,
            declaration_line: role.source.declaration_line,
            status,
            entry,
            builtin_entry,
            dynamic_entry,
        }
    }
}

/// A loaded source and its neutral Role metadata in global source order.
/// Dynamic Role attributes and start() belong to the future Role API.
#[pyclass(frozen, get_all, module = "roleforge")]
struct Project {
    path: PathBuf,
    roles: Py<PyTuple>,
}

/// Discover Roles and deliver resolved instances; never automatically call start().
#[pyfunction]
fn load(py: Python<'_>, path: PathBuf) -> PyResult<Project> {
    let registry = Registry::load()?;
    let results =
        run_file(&path, &registry, &mut io::stdout().lock()).map_err(|error| match error {
            RuntimeError::Load(error) | RuntimeError::DebugOutput(error) => PyErr::from(error),
            RuntimeError::Handoff {
                name,
                index,
                target,
                error,
            } => {
                use crate::core::pipeline::handoff::HandoffError;
                let detail = match error {
                    HandoffError::UnknownBridge(via) => format!("UnknownBridge: {via}"),
                    HandoffError::Delivery(error) => format!("{error:?}"),
                };
                PyRuntimeError::new_err(format!(
                    "Role {name} (index {index}), {}: {detail}",
                    target.display()
                ))
            }
            RuntimeError::Tokenize(error) => {
                let (line, message) = match error {
                    TokenizeError::MissingRoleName { line } => (line, "missing Role name"),
                    TokenizeError::ContentBeforeRole { line } => {
                        (line, "content before first Role")
                    }
                };
                PyValueError::new_err(format!("{}:{line}: {message}", path.display()))
            }
        })?;
    let roles = results
        .into_iter()
        .map(|result| Py::new(py, RoleInfo::from(result)))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(Project {
        path,
        roles: PyTuple::new(py, roles)?.unbind(),
    })
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(load, module)?)?;
    module.add_class::<Project>()?;
    module.add_class::<RoleInfo>()?;
    Ok(())
}
