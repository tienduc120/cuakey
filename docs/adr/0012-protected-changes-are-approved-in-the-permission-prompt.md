# 0012 - Protected changes are approved in the permission prompt

- **Status**: accepted
- **Date**: 2026-11-14

## Context

ADR-0005 made the agent guard refuse any change to a committed rule file, and gave the user one way to approve: start the session with `XTASK_ALLOW_PROTECTED=1`. In VS Code that means closing every window, because the extension inherits its environment from the running VS Code process. While the rules are still being shaped this was needed often, and the cost fell on the person, every time.

Claude Code lets a PreToolUse hook answer "ask": the user sees a permission prompt with the hook's reason and approves or denies that one tool call. The agent cannot answer the prompt itself, so it is still a human decision. The hooks reference documents that in auto mode a hook's "ask" goes to the prompt rather than the classifier; it does not say what `acceptEdits` or `bypassPermissions` do with it, and `dontAsk` denies every prompt. The hook input carries the mode as `permission_mode`, but not on every call.

## Decision

1. **A change to a committed rule file asks instead of blocking**, by an edit or through the shell: the hook prints a PreToolUse `ask` decision whose reason names the file and says to approve only a change the user wants.
2. **Only in the modes where the prompt is documented to reach the user**: `default`, `plan`, and `auto`. In `acceptEdits`, `bypassPermissions`, `dontAsk`, or when the mode is absent, the change is refused as before, so a prompt can never be approved without the user seeing it.
3. **Skipping the git hooks and commands that never return stay hard blocks in every mode**, with or without approval. The hook tells them apart by asking the guard twice: what it refuses even with approval is one of these.
4. **`XTASK_ALLOW_PROTECTED=1` stays**, for the modes that refuse and for a session of many rule changes.
5. **An ask that cannot be written becomes a block**, never a silent allow.
6. ADR-0005 decision 4 is superseded by this record; the rest of ADR-0005 stands.

## Consequences

- Positive: a rule change costs one click, in the session that needs it, and the user still reads what is being changed.
- Negative: the guard now depends on Claude Code's prompt behaving as documented. If a future version auto-approves a hook's "ask" in `default` or `auto`, protected changes would go through unseen; CODEOWNERS and CI remain the boundary either way.
- Neutral: the code lives in `xtask/src/hook.rs`; the guard's rules in `xtask/src/gates/guard.rs` did not change.

## Alternatives considered

- **Keep the restart**: safest, and the cost that prompted this record.
- **Run the `claude` CLI from a terminal with the variable**: no VS Code restart, but still a separate session per rule change.
- **An approval file or a settings `env` entry**: the agent can write files; Claude Code treats `.claude/settings*.json` as protected paths, but the docs do not say a settings `env` change reaches the hook mid-session.
