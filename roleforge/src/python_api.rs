// The public Python boundary adapts Core results; receiving calls belong to Bridges.
use std::{io, path::PathBuf};

use pyo3::{
    exceptions::{PyAttributeError, PyException},
    prelude::*,
    types::{PyList, PyTuple},
};

use crate::core::{
    bridges::Bridges,
    errors::PackagePathResolutionFailed as PackagePathFailure,
    pipeline::{dispatcher::DispatchResult, runtime::run_file_with_bridges},
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

// Preserve compatibility with callers that previously caught AttributeError.
pyo3::create_exception!(roleforge, PackagePathResolutionFailed, PyAttributeError);

/// This boundary owns load-request context; Core owns the failure explanation.
fn package_path_failure(py: Python<'_>, requested_file: &std::path::Path) -> PyResult<PyErr> {
    let diagnostic = PackagePathFailure;
    let error = PackagePathResolutionFailed::new_err(format!(
        "{}: {}\n{}\nRequested file: {}",
        PackagePathFailure::CATEGORY,
        PackagePathFailure::NAME,
        diagnostic.message(),
        requested_file.display(),
    ));
    let value = error.value(py);
    value.setattr("requested_file", requested_file.as_os_str())?;
    // Do not render an unrelated active Python exception as implicit context.
    value.setattr("__suppress_context__", true)?;
    Ok(error)
}

/// All fallible package discovery belongs inside this bounded operation.
fn discover_package_root(py: Python<'_>) -> PyResult<PathBuf> {
    let package_file = py.import("roleforge")?.getattr("__file__")?;
    py.import("pathlib")?
        .getattr("Path")?
        .call1((package_file,))?
        .call_method0("resolve")?
        .getattr("parent")?
        .extract()
}

/// Discover Roles and deliver resolved instances; never automatically call start().
#[pyfunction]
fn load(py: Python<'_>, path: PathBuf) -> PyResult<Project> {
    let package_root = match discover_package_root(py) {
        Ok(root) => root,
        Err(error) if error.is_instance_of::<PyException>(py) => {
            // Discard the internal exception before creating the public diagnostic.
            drop(error);
            return Err(package_path_failure(py, &path)?);
        }
        // BaseException subclasses (including process-control exceptions) propagate.
        Err(error) => return Err(error),
    };
    let registry = match Registry::load(&package_root) {
        Ok(registry) => registry,
        Err(error) => return Err(crate::python_errors::registry(py, error)?),
    };
    let mut output = io::stdout().lock();
    let results =
        match run_file_with_bridges(&path, &registry, &Bridges::with_builtins(), &mut |event| {
            crate::core::output::report(event, &mut output)
        }) {
            Ok(results) => results,
            Err(error) => return Err(crate::python_errors::runtime(py, error, &path)?),
        };
    drop(output);
    build_project(py, &path, results)
}

fn build_project(
    py: Python<'_>,
    path: &std::path::Path,
    results: crate::core::pipeline::runtime::LoadResult<Py<PyAny>>,
) -> PyResult<Project> {
    use crate::python_errors::project;
    let adapter = project(py, py.import("roleforge._live"), path)?;
    let delivered = project(
        py,
        PyList::new(py, results.delivered.into_iter().map(|(_, live)| live)),
        path,
    )?;
    // Grouping accesses live Role attributes after successful receipt. A Role
    // may override those properties: its exceptions must propagate unchanged.
    let live = adapter
        .call_method1("_ProjectRoles", (delivered,))?
        .unbind();
    let roles = results
        .roles
        .into_iter()
        .map(|result| Py::new(py, RoleInfo::from(result)))
        .collect::<PyResult<Vec<_>>>();
    let roles = project(py, roles, path)?;
    Ok(Project {
        path: path.into(),
        roles: project(py, PyTuple::new(py, roles), path)?.unbind(),
        live,
    })
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let error_type = module.py().get_type::<PackagePathResolutionFailed>();
    error_type.setattr("category", PackagePathFailure::CATEGORY)?;
    module.add("PackagePathResolutionFailed", error_type)?;
    module.add_function(wrap_pyfunction!(load, module)?)?;
    module.add_class::<Project>()?;
    module.add_class::<RoleInfo>()?;
    Ok(())
}
