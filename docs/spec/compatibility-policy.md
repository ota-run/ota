# Compatibility Policy

This document defines the current V1 contract stability policy for `ota.yaml`.

## Versioning

- `version: 1` is the current contract version.
- breaking schema changes require a version bump.
- additive fields may be introduced within V1 when they do not break existing valid contracts.

## Unknown keys

Current V1 policy is strict:

- unknown keys fail parsing
- there is no warning-only unknown-key mode in the current implementation
- top-level `extensions` is a known contract field and is parsed as inert contract data
- later-spec fields such as `readiness_gate` are not part of the current shipped parser unless
  the implementation explicitly adds them

This favors contract clarity over permissive parsing.

## Deprecation

When a field needs to be replaced:

- the replacement should be documented first
- the old field should remain valid for a warning period when feasible
- removal should not happen silently

## Migration

If V1 contract evolution requires migration:

- the migration should be documented
- the target schema should be explicit
- breaking transitions should not rely on guesswork
