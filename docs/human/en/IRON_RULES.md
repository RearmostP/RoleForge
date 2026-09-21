# RoleForge Iron Rules for contributors

[Documentation](README.md) · [עברית](../he/IRON_RULES.md) · [Creating Roles](CREATING_ROLES.md)

These principles describe what should remain true as RoleForge evolves. They are a short architectural guide, not an inventory of current filenames. The [detailed AI Iron Rules](../../ai/CORE_IRON_RULES.md) provide the deeper development constraints.

1. **Core knows the protocol, never the roles.** Core recognizes RoleForge declarations, preserves data, and routes it. It must not learn what a command inside a Role's body means.

2. **The source determines what the Role is. The Role determines what API and behavior it exposes.** `@role Test` establishes the discovered identity. Registration provides its destination; an implementation class or an import does not replace source discovery.

3. **Role discovery must happen before Role-specific behavior is exposed.** First establish which instances exist and where they occur. A convenient API must not assume a Role identity before inspecting the source. Dynamic Role convenience APIs are still future work.

4. **Components process. Runtime orchestrates.** Loader reads; Main Tokenizer discovers; Registry resolves; Dispatcher produces routing results; Handoff resolves a Bridge for delivery. Runtime controls the order. A major component should not take over driving the next major component.

5. **The Core resolves destinations, not implementations.** Routing needs a registered destination and Bridge identifier. It should not branch on a particular Role's internal types, grammar, or behavior.

6. **The Core resolves and delivers. The Role decides what happens next.** Successful receiving handoff is the boundary. The Role can parse, validate, store the instance, prepare an API, or do nothing; Core does not manage its internal lifecycle.

7. **Every Role receives the same logical RoleInput contract.** Core owns neutral data: name, both indexes, body, and source metadata. Bridges adapt its representation to their environments without inventing new indexes or removing information.

8. **Bridge identifiers select Bridges; they do not declare programming languages.** `via: "python"` selects the Bridge registered under that exact identifier. Neither the Role name nor a `.py` extension is a substitute for explicit selection.

9. **roleforge_receive is handoff, not start.** The required Python receiver accepts the delivered input. Receipt does not authorize Core to call `start()` or any other user-facing operation. There is no finalized Core-managed start lifecycle today.

10. **A Role owns its own language.** Any Role-specific tokenizer, parser, validation, and semantics belong to the Role. A Role only implements the pieces it needs; there is no mandatory parser pipeline and no Main Parser in Core.

11. **A new Role should not require teaching the Core what that Role means.** Adding a Python implementation and a Registry entry should fit the existing handoff contract. If adding a Role requires a name-specific Core branch, reconsider the boundary.

12. **Optimize obvious waste immediately. Postpone architectural optimization.** Remove simple unnecessary copies or repeated work when safe. Do not introduce caches, concurrency, or a larger architecture solely for hypothetical performance.

13. **An undecided detail is not a decision.** An example, plausible design, or old prompt does not establish a future contract. Rust delivery, final Role APIs, aliases, packaging, and the final Error/Console Managers remain open where no explicit decision exists.

Two invariants support these principles: preserve global order, per-name indexes, and source information throughout delivery; never silently prefer built-in or dynamic registration when both contain the same name. Current conflict preflight prevents every delivery in that load. See the [overview](README.md) for the distinction between established principles and temporary reporting behavior.
