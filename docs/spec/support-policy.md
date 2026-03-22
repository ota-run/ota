# Support Policy

This document defines the current platform support stance for Ota V1.

## Current stance

- Linux: first-class target
- macOS: first-class target
- Windows: partial support in V1

## Shell semantics

Current task execution is shell-compatible:

- Unix-like systems: `sh -lc`
- Windows: `cmd /C`

This is explicit by design. V1 does not provide per-task shell selection.

## Practical implication

Repos should expect the best behavior today on Linux and macOS.

Windows support exists, but shell behavior and script portability are more constrained in V1.
