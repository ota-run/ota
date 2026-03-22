# Fixture Repo Plan

Ota needs real repo shapes to avoid designing in fantasy.

## Current fixture coverage

- Node
- Python
- Go
- mixed Node/Python
- mixed Node/Go
- Java
- Java Maven repo
- Java Gradle multi-module repo
- Docker-heavy repo
- unsupported Docker-only repo
- Node conflict monorepo
- ugly real-world repo
- polyglot ops repo

These now exist as canonical real-shape fixtures under `tests/fixtures/real`.

## Next fixture pressure

- deeper `doctor` assertions against service and lifecycle behavior
- more precedence/conflict assertions on mixed-reality repos
- targeted follow-up fixtures only when they expose a real product gap

## Purpose

Fixture repos should drive:

- `doctor` trust
- `init` usefulness
- `detect` precedence and coverage
- output stability across real shapes
