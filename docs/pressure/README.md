# Pressure Evidence Registry

[`evidence-manifest.json`](evidence-manifest.json) is the canonical registry for retained Ota
pressure evidence. It binds each case to its repository revision, Ota revision, hosted matrix,
exercised surfaces, proven facts, explicit limits, and engineering-note status.

The manifest is not a product scorecard. A case never means that a repository "passed Ota," is
endorsed, is governed repository-wide, or has proved behavior outside its declared surfaces.

## Statuses

- `immutable_pressure`: a retained hosted matrix binds the listed revision and controls.
- `pre_release_design_partner`: bounded fork evidence before release and maintainer review.

The Site renders a generated discovery projection of this manifest. The Core manifest remains the
technical source of truth; the projection may summarize but must not add evidence claims.

## Scope

The registry starts with the retained, revision-bound pressure corpus used by current product
work. Older engineering notes remain useful narrative history, but they do not become canonical
registry entries until their exact source, matrix, proven controls, and limits are reconciled here.
This prevents an old post from being silently upgraded into current evidence.

## Evidence Rule

Every case must state both `proven_facts` and `not_proved`. A green workflow, passing command, or
published engineering note is not a substitute for that boundary.

Each evidence entry always binds its platform set and hosted run. When a case has separately
retained scenarios, it may also bind the scenario name, exact job URL, and artifact name plus
lowercase SHA-256 digest. Those fields identify retained evidence; they do not broaden the listed
`proven_facts`.
