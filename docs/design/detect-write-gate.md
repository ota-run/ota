# Detect Write Gate

`ota detect` write mode is trust-sensitive.

The rule is simple: dry-run quality must be credible before write behavior expands.

## Current policy

Today, write mode is conservative by design:

- `ota detect --dry-run` prints the full detected candidate
- each inferred field includes provenance
- each inferred field includes confidence
- `ota detect` writes only `high` confidence fields
- write mode validates before writing
- write mode refuses to overwrite an existing `ota.yaml`

## Write gate

Detection should only write fields automatically when these conditions hold:

1. `--dry-run` output is backed by fixture coverage for that repo shape.
2. Every written field has explicit provenance.
3. Every written field has explicit confidence.
4. `high` confidence fields are consistently correct on the supported fixtures.
5. The written projection passes `ota validate`.
6. A human can review the dry-run output and understand where each field came from.

## Confidence policy

- `high`: eligible for automatic write
- `medium`: show in dry-run, do not write automatically
- `low`: show in dry-run, do not write automatically

This keeps write mode conservative and review mode more informative.

## Practical acceptance bar

Before expanding write behavior for a new source or repo shape:

- add or extend fixture coverage
- verify precedence rules when multiple signals disagree
- verify the projected contract still validates
- prefer omission over ambiguous writes

Current precedence rule:

- higher confidence wins
- on equal confidence, more repo-specific runtime declarations should beat generic version-manager aggregation

## Why this exists

ota wins trust when detection is reviewable, explicit, and unsurprising.

It loses trust when detection writes configuration that looks confident but is actually guessed.
