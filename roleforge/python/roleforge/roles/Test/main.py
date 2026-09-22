from roleforge import Role as BaseRole


class Role(BaseRole):
    def hello(self):
        self.calls += 1
        print(f"Hello from {self.name}[{self.role_index}]!")
        return self.body


def roleforge_receive(role):
    role.calls = 0
    print("=== ROLEFORGE TEST ROLE ===")
    print("Role received successfully!")
    print(f"name: {role.name}")
    print(f"global index: {role.index}")
    print(f"role index: {role.role_index}")
    print(f"body: {role.body!r}")
    print(f"declaration line: {role.source.declaration_line}")
