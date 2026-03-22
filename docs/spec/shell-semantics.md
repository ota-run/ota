# Shell Semantics

This document defines the current V1 shell execution contract.

## Current execution model

Ota is shell-native in V1.

Current command execution uses:

- Unix-like systems: `sh -lc`
- Windows: `cmd /C`

This applies to:

- task `run`
- task `script`
- service `start`
- service `stop`
- service `healthcheck`
- configured checks

## Working directory

Commands run in the contract directory:

- if a direct `ota.yaml` path is given, its parent directory is used
- if a directory is given, that directory is used

## Environment behavior

Current V1 behavior:

- configured env values are applied to task execution
- required env values must be present or resolvable from defaults where allowed
- Ota does not translate shell syntax across platforms

That means shell-specific constructs remain platform-sensitive.

Examples:

- `export FOO=bar && cmd` is Unix-style
- `set FOO=bar && cmd` is Windows `cmd` style

## `run` vs `script`

Current V1 distinction:

- `run`: single command string
- `script`: multiline shell body

Both still execute through the same platform shell model above.

## Non-goals in V1

- no per-task shell selection
- no PowerShell-specific execution mode
- no Bash-only contract mode
- no shell portability translation
- no isolated shell environment for `ephemeral`

## Practical implication

Repos should write task and service commands with their supported platforms in mind.

Ota guarantees the invocation model. It does not guarantee that shell-specific commands become portable automatically.
