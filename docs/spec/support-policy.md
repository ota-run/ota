# Platform support

This document explains which platforms ota supports best today and where the boundary is
narrower.

## Current stance

- Linux: first-class target
- macOS: first-class target
- Windows: partial support in V1

## Shell semantics

Current task execution is shell-compatible:

- Unix-like systems: `sh -lc`
- Windows: `cmd /C`

This is explicit by design. V1 does not provide per-task shell selection.

The full shell contract is documented in [shell-semantics.md](shell-semantics.md).

## Practical implication

Repos should expect the best behavior today on Linux and macOS.

Windows support exists, but shell behavior and script portability are more constrained in V1.

## Lifecycle semantics

Current lifecycle support is intentionally limited:

- `persistent` maps to the current shell-native execution model
- `ephemeral` is accepted, but advisory only in V1

ota does not yet create isolated temporary environments, temporary workspaces, or automatic cleanup flows for `ephemeral`.
