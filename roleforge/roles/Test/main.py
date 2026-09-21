def roleforge_receive(role):
    print("=== ROLEFORGE TEST ROLE ===")
    print("Role received successfully!")
    print(f"name: {role.name}")
    print(f"global index: {role.index}")
    print(f"role index: {role.role_index}")
    print(f"body: {role.body!r}")
    print(f"declaration line: {role.source.declaration_line}")
