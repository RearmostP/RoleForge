"""Run after installing the repository with this environment's `python -m pip install -e .`."""
from pathlib import Path

from roleforge import load


if __name__ == "__main__":
    project = load(Path(__file__).with_name("test_role.rfg"))
    print(f"Loaded {len(project.roles)} Roles from {project.path}")
