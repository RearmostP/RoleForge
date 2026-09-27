"""Focused Core API errors; preserve built-in exception classes and messages.

Each constructor owns its case-specific data. Native Python argument/protocol
errors and failures in Role methods after receipt propagate unchanged.
"""


def live_role_not_found(name):
    error = KeyError(f"No successfully delivered Role named {name!r}")
    error.case = "LiveRoleNotFound"
    error.role_name = name
    return error


def role_occurrence_not_found(index):
    error = IndexError(f"No live Role occurrence at role_index {index}")
    error.case = "RoleOccurrenceNotFound"
    error.role_index = index
    return error


def live_role_attribute_not_found(attribute):
    error = AttributeError(f"No live Role attribute {attribute!r}")
    error.case = "LiveRoleAttributeNotFound"
    error.attribute = attribute
    return error


def ambiguous_role_attribute(attribute, names):
    error = AttributeError(f"Ambiguous Role attribute {attribute!r}; use get_role(exact_name)")
    error.case = "AmbiguousRoleAttribute"
    error.attribute = attribute
    error.role_names = tuple(names)
    return error


def invalid_role_class():
    error = TypeError("Target Role must subclass roleforge.Role")
    error.case = "InvalidRoleClass"
    return error
