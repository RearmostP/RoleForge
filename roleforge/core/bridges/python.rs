use super::{Bridge, BridgeError};
use crate::core::role_input::{RoleInput, RoleSource};
use pyo3::{exceptions::PyAttributeError, prelude::*, types::PyDict};
use std::path::Path;

pub(super) struct PythonBridge;

#[pyclass(frozen, get_all, name = "RoleSource", module = "roleforge")]
#[derive(Clone)]
struct PythonSource {
    declaration_line: usize,
}

impl From<RoleSource> for PythonSource {
    fn from(source: RoleSource) -> Self {
        Self {
            declaration_line: source.declaration_line,
        }
    }
}

#[pyclass(frozen, get_all, name = "RoleInput", module = "roleforge")]
struct PythonInput {
    name: String,
    index: usize,
    role_index: usize,
    body: String,
    source: PythonSource,
}

impl From<RoleInput> for PythonInput {
    fn from(role: RoleInput) -> Self {
        Self {
            name: role.name,
            index: role.index,
            role_index: role.role_index,
            body: role.body,
            source: role.source.into(),
        }
    }
}

impl Bridge for PythonBridge {
    fn deliver(&self, target: &Path, role: RoleInput) -> Result<(), BridgeError> {
        Python::initialize();
        Python::attach(|py| {
            let load_error = |error: PyErr| BridgeError::PythonTargetLoadFailure(error.to_string());
            let target = target
                .canonicalize()
                .map_err(|error| BridgeError::PythonTargetLoadFailure(error.to_string()))?;
            let filename = target.to_str().ok_or_else(|| {
                BridgeError::PythonTargetLoadFailure("target is not Unicode".into())
            })?;
            // Encode the full canonical path, not just the basename. No collisions,
            // target search, sys.path changes, or persistent module cache.
            use std::fmt::Write;
            let mut name = String::from("_roleforge_target_");
            for byte in filename.as_bytes() {
                write!(&mut name, "{byte:02x}").unwrap();
            }
            let machinery = py.import("importlib.machinery").map_err(load_error)?;
            let util = py.import("importlib.util").map_err(load_error)?;
            let loader = machinery
                .getattr("SourceFileLoader")
                .and_then(|c| c.call1((&name, filename)))
                .map_err(load_error)?;
            let spec = util
                .call_method1("spec_from_loader", (&name, &loader))
                .map_err(load_error)?;
            let module = util
                .call_method1("module_from_spec", (&spec,))
                .map_err(load_error)?;
            let modules = py
                .import("sys")
                .and_then(|sys| sys.getattr("modules"))
                .map_err(load_error)?
                .cast_into::<PyDict>()
                .map_err(|e| load_error(e.into()))?;
            let previous = modules.get_item(&name).map_err(load_error)?;
            modules.set_item(&name, &module).map_err(load_error)?;
            let result = (|| {
                // Compile source directly to avoid stale timestamp-based bytecode.
                let source = loader
                    .call_method1("get_source", (&name,))
                    .map_err(load_error)?;
                let code = loader
                    .call_method1("source_to_code", (source, filename))
                    .map_err(load_error)?;
                let globals = module.getattr("__dict__").map_err(load_error)?;
                py.import("builtins")
                    .and_then(|b| b.call_method1("exec", (code, globals)))
                    .map_err(load_error)?;
                let receiver = module.getattr("roleforge_receive").map_err(|error| {
                    if error.is_instance_of::<PyAttributeError>(py) {
                        BridgeError::MissingReceiver
                    } else {
                        load_error(error)
                    }
                })?;
                if !receiver.is_callable() {
                    return Err(BridgeError::ReceiverNotCallable);
                }
                let input = Py::new(py, PythonInput::from(role))
                    .map_err(|e| BridgeError::InputConversion(e.to_string()))?;
                receiver
                    .call1((input,))
                    .map_err(|e| BridgeError::ReceiverRaised(e.to_string()))?;
                Ok(())
            })();
            let cleanup = match previous {
                Some(previous) => modules.set_item(&name, previous),
                None => modules.del_item(&name),
            };
            result.and(cleanup.map_err(load_error))
        })
    }
}
