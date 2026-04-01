

# Shell Semantics

ota is shell-native in V1. It does not invent a new shell language and it does not translate shell
syntax between platforms.

Use this page when you need to know:

- which shell ota uses on each platform
- where commands run
- how `run` and `script` differ
- what env handling does and does not do
- what to expect when a command is platform-specific

## What ota does today

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

That means ota runs shell text through the platform shell, not through a portability layer.

## Working directory

Commands run in the contract directory:

- if a direct `ota.yaml` path is given, ota uses its parent directory
- if a directory is given, ota uses that directory
- if no path is given, ota discovers the contract upward from the current directory

That matters because shell commands often assume relative paths, build outputs, or service files
inside the repo root.

## Environment behavior

Current V1 behavior:

- configured env values are applied to task execution
- required env values must be present or resolvable from defaults where allowed
- ota does not translate shell syntax across platforms

That means shell-specific constructs remain platform-sensitive.

Examples:

- `export FOO=bar && cmd` is Unix-style
- `set FOO=bar && cmd` is Windows `cmd` style

## `run` vs `script`

Current V1 distinction:

- `run`: single command string
- `script`: multiline shell body

Both still execute through the same platform shell model above.

Use `run` for one command you would type directly in a shell.
Use `script` when the command needs setup, multiple lines, or shell flow control.

## Use cases

- a repo works on Linux and macOS, but Windows needs different shell syntax
- a task needs a multiline setup script instead of one shell string
- a service healthcheck depends on a command that only exists in the repo shell environment
- a CI job needs to know whether ota will call `sh -lc` or `cmd /C`

## Practical example

If a repo needs a task that behaves differently by platform, use task variants instead of hoping ota
will translate the shell for you:

```yaml
tasks:
  test:
    variants:
      - when:
          os: linux
        run: pnpm test -- --runInBand
      - when:
          os: macos
        run: pnpm test -- --runInBand
      - when:
          os: windows
        script: |
          pnpm test -- --runInBand
```

## What ota does not do

- no per-task shell selection
- no PowerShell-specific execution mode in V1
- no Bash-only contract mode
- no shell portability translation
- no isolated shell environment for `ephemeral`

## Practical implication

Repos should write task and service commands with their supported platforms in mind.

ota guarantees the invocation model. It does not guarantee that shell-specific commands become
portable automatically.
