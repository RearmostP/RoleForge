// The public Python boundary adapts Core results; receiving calls belong to Bridges.
use std::{io, path::PathBuf};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::{PyList, PyTuple},
};

use crate::core::{
    bridges::Bridges,
    pipeline::{
        dispatcher::DispatchResult,
        runtime::{RuntimeError, run_file_with_bridges},
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

/// A loaded source, discovery metadata, and successfully delivered native Roles.
#[pyclass(frozen, module = "roleforge")]
struct Project {
    #[pyo3(get)]
    path: PathBuf,
    #[pyo3(get)]
    roles: Py<PyTuple>,
    live: Py<PyAny>,
}

#[pymethods]
impl Project {
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        self.live.call_method1(py, "by_attribute", (name,))
    }

    #[pyo3(signature = (name, role_index=0))]
    fn get_role(&self, py: Python<'_>, name: &str, role_index: isize) -> PyResult<Py<PyAny>> {
        self.live.call_method1(py, "by_name", (name, role_index))
    }
}

/// Discover Roles and deliver resolved instances; never automatically call start().
#[pyfunction]
fn load(py: Python<'_>, path: PathBuf) -> PyResult<Project> {
    // Resolve resources from the imported package, independently of the build tree.
    let package_file = py.import("roleforge")?.getattr("__file__")?;
    let package_root: PathBuf = py
        .import("pathlib")?
        .getattr("Path")?
        .call1((package_file,))?
        .call_method0("resolve")?
        .getattr("parent")?
        .extract()?;
    let registry = Registry::load(&package_root)?;
    let results = run_file_with_bridges(
        &path,
        &registry,
        &Bridges::with_builtins(),
        &mut io::stdout().lock(),
    )
    .map_err(|error| match error {
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
                TokenizeError::ContentBeforeRole { line } => (line, "content before first Role"),
            };
            PyValueError::new_err(format!("{}:{line}: {message}", path.display()))
        }
    })?;
    let live = py
        .import("roleforge._live")?
        .call_method1(
            "_ProjectRoles",
            (PyList::new(
                py,
                results.delivered.into_iter().map(|(_, live)| live),
            )?,),
        )?
        .unbind();
    let roles = results
        .roles
        .into_iter()
        .map(|result| Py::new(py, RoleInfo::from(result)))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(Project {
        path,
        roles: PyTuple::new(py, roles)?.unbind(),
        live,
    })
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(load, module)?)?;
    module.add_class::<Project>()?;
    module.add_class::<RoleInfo>()?;
    Ok(())
}
