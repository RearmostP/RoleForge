// Stable, environment-neutral Core-to-Role handoff contract.
use crate::core::pipeline::models::CleanRole;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RoleInput {
    pub(crate) name: String,
    pub(crate) index: usize,
    pub(crate) role_index: usize,
    pub(crate) body: String,
    pub(crate) source: RoleSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RoleSource {
    pub(crate) declaration_line: usize,
}

impl From<&CleanRole> for RoleInput {
    fn from(role: &CleanRole) -> Self {
        Self {
            name: role.name.clone(),
            index: role.index,
            role_index: role.role_index,
            body: role.body.clone(),
            source: RoleSource {
                declaration_line: role.source.declaration_line,
            },
        }
    }
}
