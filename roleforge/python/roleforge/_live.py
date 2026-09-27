"""Python representation used by the Python Bridge, independent of Role semantics."""
import keyword
import operator

from . import _errors


def _select(instances, index):
    index = operator.index(index)
    try:
        return instances[index]
    except KeyError:
        raise _errors.role_occurrence_not_found(index) from None


class Role:
    """Base for an optional target-module Role class; initialize state in the receiver."""

    def __init__(self, role_input):
        self._input = role_input
        self._instances = {role_input.role_index: self}

    @property
    def name(self):
        return self._input.name

    @property
    def index(self):
        return self._input.index

    @property
    def role_index(self):
        return self._input.role_index

    @property
    def body(self):
        return self._input.body

    @property
    def source(self):
        return self._input.source

    @property
    def role_input(self):
        return self._input

    def __getitem__(self, index):
        return _select(self._instances, index)


def _create_role(module, role_input):
    role_type = vars(module).get("Role", Role)
    if not isinstance(role_type, type) or not issubclass(role_type, Role):
        raise _errors.invalid_role_class()
    return role_type(role_input)


class _ProjectRoles:
    def __init__(self, delivered):
        self.groups = {}
        self.attributes = {}
        for role in delivered:
            self.groups.setdefault(role.name, {})[role.role_index] = role
        for name, instances in self.groups.items():
            for role in instances.values():
                role._instances = instances
            attribute = name.lower()
            if attribute.isidentifier() and not keyword.iskeyword(attribute):
                self.attributes.setdefault(attribute, []).append(name)

    def by_name(self, name, index):
        try:
            instances = self.groups[name]
        except KeyError:
            raise _errors.live_role_not_found(name) from None
        return _select(instances, index)

    def by_attribute(self, attribute):
        names = self.attributes.get(attribute, ())
        if not names:
            raise _errors.live_role_attribute_not_found(attribute)
        if len(names) != 1:
            raise _errors.ambiguous_role_attribute(attribute, names)
        return self.by_name(names[0], 0)
