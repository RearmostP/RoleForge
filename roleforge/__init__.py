"""Minimal Python interface to the Rust Core; load() delivers resolved Roles through their receiving entry point."""
from ._native import Project, RoleInfo, load
from ._live import Role

__all__ = ["load", "Project", "RoleInfo", "Role"]
