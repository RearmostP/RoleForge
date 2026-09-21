"""Minimal Python interface to the Rust Core; load() delivers resolved Roles through their receiving entry point."""
from ._native import Project, RoleInfo, load

__all__ = ["load", "Project", "RoleInfo"]
