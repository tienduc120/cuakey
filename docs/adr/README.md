# Architecture decisions

One file per decision, numbered, never edited after acceptance (supersede it with a new record instead).

| # | Title | Status |
| --- | --- | --- |
| [0001](0001-foundations.md) | Foundations | accepted; decision 4 in part superseded by 0014 |
| [0002](0002-gates-proven-by-fixtures.md) | Gates, proven by fixtures | accepted |
| [0003](0003-git-hooks.md) | Git hooks | accepted |
| [0004](0004-lint-policy.md) | Lint policy: deny everything, suppress one item with a reason | accepted; decision 1 in part superseded by 0014 |
| [0005](0005-agent-guard.md) | Agent guard | accepted; decision 4 superseded by 0012 |
| [0006](0006-the-domain-is-no-std.md) | The domain is no_std | accepted |
| [0007](0007-ports-use-cases-and-aggregates.md) | Ports, use cases, and aggregates | accepted |
| [0008](0008-supply-chain.md) | Supply chain | accepted |
| [0009](0009-supervised-tasks-clock-and-http.md) | Supervised tasks, the clock port, and the HTTP server | accepted; decision 3 superseded by 0014 |
| [0010](0010-the-service-generator-copies-the-witness.md) | The service generator copies the witness | accepted |
| [0011](0011-errors-are-values-owned-by-the-layer-that-raises-them.md) | Errors are values owned by the layer that raises them | accepted; decision 4 in part superseded by 0014 |
| [0012](0012-protected-changes-are-approved-in-the-permission-prompt.md) | Protected changes are approved in the permission prompt | accepted |
| [0013](0013-rules-from-published-rust-guides.md) | Rules from published Rust guides | accepted; decision 6 in part superseded by 0014 |
| [0014](0014-desktop-app.md) | A Windows desktop app, not a server | accepted |
| [0015](0015-dependencies.md) | Dependencies of the desktop app | accepted |

Needs an ADR first: a new external dependency, a hosted service, the toolchain pin or a pinned tool moving, a change to a published contract, or a gate added, loosened, or removed.
