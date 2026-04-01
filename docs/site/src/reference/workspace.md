# Workspace (`ota.workspace.yaml`)

`ota.workspace.yaml` is the canonical workspace bootstrap contract for multi-repo orchestration.

Use it when one repo is not enough and you need a single contract to describe:

- which repos belong in the workspace
- where they live on disk
- which repos depend on which others
- where missing repos should be acquired from
- how workspace-level `doctor`, `up`, and `run` should move in dependency order

## Source model

This page is the canonical public reference for workspace behavior. It adds
examples, use cases, and operator guidance so the page stands on its own while
staying aligned with shipped behavior.

The key design rule is simple:

- the workspace contract orchestrates repos
- repo contracts remain the source of truth for repo readiness

## What it defines

- workspace identity
- repo paths and dependency graph
- acquisition source for missing repos
- deterministic execution order for workspace commands

Why users need it:

- lets ota know which repos belong in one workspace
- makes missing repos explicit instead of leaving them as broken local paths
- keeps workspace orchestration deterministic when one repo depends on another

## Minimal example

```yaml
version: 1
workspace:
  name: example-workspace
  description: Local fullstack workspace
  git_base: https://github.com/example
repos:
  api:
    path: repos/api
    required: true
    source:
      repo: api
  web:
    path: repos/web
    depends_on:
      - api
    source:
      repo: web
```

This says:

- the workspace is called `example-workspace`
- the `api` repo must exist and can be acquired from the base git host
- the `web` repo depends on `api`
- workspace commands should respect that order

Why that matters:

- a missing repo can be cloned from the declared source instead of blocking on a manual setup step
- dependent repos will not run ahead of their prerequisites
- users can read the workspace contract and understand the boot order immediately

## How it works

- `ota workspace validate` checks workspace contract correctness
- `ota workspace doctor` diagnoses workspace readiness repo by repo
- `ota workspace explain` turns workspace findings into ordered remediation steps
- `ota workspace up` can acquire missing repos from `source`
- workspace orchestration reuses repo-level `ota up` and `ota run` behavior
- dependency order is deterministic

What users should expect:

- `required: true` means the repo is part of the workspace’s ready state and should not be ignored
- optional repos can be reported without blocking the whole workspace
- acquired repos are brought in before workspace prepare continues
- independent repos may run in parallel, but only after their dependencies are already satisfied

## Use cases

- bootstrapping a fullstack system spread across `api`, `web`, and `infra` repos
- running one task, like `test`, across repos in deterministic order
- diagnosing cross-repo readiness failures from one command
- acquiring a missing repo before workspace bootstrap starts
- keeping workspace setup explicit without collapsing repo contracts into one file
- distinguishing required repos from optional ones so the workspace can stay useful while incomplete

## Practical workflow

1. `ota workspace validate`
2. `ota workspace doctor`
3. `ota workspace explain`
4. `ota workspace up`
5. `ota workspace run <task>`

## `workspace`

```yaml
workspace:
  name: ota-dev
  description: Local multi-repo development workspace
  git_base: https://github.com/ota
```

Fields:

- `name`: required, non-empty string
- `description`: optional string
- `git_base`: optional clone base used by `repos.<name>.source.repo`

Why users need it:

- `name` gives the workspace a stable identity for reports and receipts
- `description` tells humans what this workspace is for at a glance
- `git_base` lets shorthand repo slugs resolve without repeating full clone URLs

## `repos`

```yaml
repos:
  web:
    path: apps/web
    source:
      repo: web
  api:
    path: services/api
    contract: services/api/ota.yaml
    required: true
    depends_on:
      - web
    source:
      git: https://github.com/ota/api.git
      ref: main
```

Fields:

- `path`: required path to a repo directory, relative to `ota.workspace.yaml`
- `contract`: optional explicit repo contract path, relative to `ota.workspace.yaml`
- `required`: optional boolean
- `depends_on`: optional list of workspace repo names
- `source`: optional acquisition source for repos that are not present yet

Why users need them:

- `path` tells ota where the repo should live on disk
- `contract` lets users point at a repo contract that is not at the default path
- `required` tells ota whether a missing repo should block the workspace
- `depends_on` keeps boot order explicit
- `source` tells ota how to acquire a missing repo

## `source`

`source` fields:

- `git`: explicit clone URL or git-accepted clone source
- `repo`: repo path or slug resolved against `workspace.git_base`
- `ref`: optional branch, tag, or ref to checkout after clone

Why users need it:

- `git` is the explicit full clone path when you already know the repository URL
- `repo` is the shorthand form when many repos share one base host
- `ref` makes the acquired workspace reproducible without guessing which branch to use

Design intent:

- `source.git` is the canonical acquisition field
- `source.repo` is shorthand for multiple repos sharing the same `workspace.git_base`
- both are generic git concepts and work for GitHub, GitLab, Bitbucket, and internal git hosts

## Validation behavior

- repo names must not be empty
- workspace must declare at least one repo
- repo `path` must be non-empty
- repo `path` must exist and point to a directory unless `source` is declared
- `contract` must be non-empty when present
- if `contract` is omitted, ota expects `<repo path>/ota.yaml`
- `source` must declare exactly one of `git` or `repo`
- `source.repo` requires `workspace.git_base`
- `depends_on` references must resolve to known workspace repos
- workspace repo dependency cycles are rejected
- each present repo contract must load and pass repo-level validation

## `ota workspace doctor`

Current workspace diagnosis behavior:

- validates workspace structure first
- evaluates repos in dependency order
- can diagnose independent repos concurrently when `--jobs` is greater than `1`
- preserves deterministic repo ordering in the final report
- diagnoses each referenced repo through its own `ota.yaml`
- reports missing-but-acquirable repos as not yet acquired instead of treating them as unreadable local paths
- preserves repo-level diagnosis semantics for required repos
- downgrades optional repo errors to warnings at the workspace layer
- rejects required repos that depend on optional repos

Use cases:

- show which repo is blocking the workspace before you start fixing files
- see optional repos without letting them fail the whole workspace
- understand whether the workspace needs acquisition or just diagnosis

This keeps workspace behavior as orchestration over repo readiness, not a parallel readiness system.

## `ota workspace up`

Current workspace prepare behavior:

- validates workspace structure first
- acquires missing repos declared with `source` before repo-level bootstrap
- runs repo-level `up` for each referenced repo
- can prepare independent repos concurrently when `--jobs` is greater than `1`
- respects declared workspace repo dependency order
- blocks downstream repos when a dependency does not become ready
- aggregates repo-level status, phase, findings, and exit details
- captures repo child stdout and stderr per repo so the final report remains deterministic
- emits live repo progress on stderr in text mode so users can see execution moving without losing ordered final output
- optional repo failures do not fail the overall workspace status
- `--stream` opts into raw live child process output instead of buffered per-repo output

Use cases:

- acquire a missing repo before the rest of the workspace prepares
- keep the boot order deterministic even when multiple repos are involved
- run independent repos in parallel without losing ordered final output

Current execution policy:

- workspace repo execution defaults to sequential because `--jobs` defaults to `1`
- ota only parallelizes repos whose dependencies are already satisfied
- final reporting remains in deterministic repo order even when execution is concurrent
- required repos must not depend on optional repos, because required readiness cannot rest on optional guarantees
- `--stream` is currently text-only and requires `--jobs 1` so raw child logs do not interleave

Current non-goals:

- cross-repo dependency scheduling
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning
- a workspace-only bootstrap engine that bypasses repo contracts
- implicit pull, fetch, or update behavior for repos that already exist locally
- GitHub API integration or non-git acquisition modes
