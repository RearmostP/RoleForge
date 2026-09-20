"""Minimal Python interface to the Rust Core; Role execution is not yet implemented."""
from ._native import Project, RoleInfo, load

__all__ = ["load", "Project", "RoleInfo"]
