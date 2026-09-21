"""Python representation used by the Python Bridge, independent of Role semantics."""
import keyword
import operator


def _select(instances, index):
    index = operator.index(index)
    try:
        return instances[index]
    except KeyError:
        raise IndexError(f"No live Role occurrence at role_index {index}") from None


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
        raise TypeError("Target Role must subclass roleforge.Role")
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
            raise KeyError(f"No successfully delivered Role named {name!r}") from None
        return _select(instances, index)

    def by_attribute(self, attribute):
        names = self.attributes.get(attribute, ())
        if not names:
            raise AttributeError(f"No live Role attribute {attribute!r}")
        if len(names) != 1:
            raise AttributeError(
                f"Ambiguous Role attribute {attribute!r}; use get_role(exact_name)"
            )
        return self.by_name(names[0], 0)
