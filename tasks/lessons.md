# Lessons

## 2026-03-21

- Pattern: Source files need valid repository license notes, but non-language comment formats or decorative Unicode headers can break compilation.
- Correction: Use a short ASCII language-valid license header in Rust source files and verify the compiler still accepts the file afterward.
- Rule: When adding or restoring license notes in code, use the target language's native comment syntax only and avoid decorative Unicode banners.

## 2026-03-24

- Pattern: Docs pages that only list commands are low adoption value and create user confusion.
- Correction: Rewrite command/docs pages to include when-to-use, why, and concrete use-cases with practical command examples.
- Rule: For OSS-facing docs updates, require adoption-first guidance (when/why/use-case), not enumeration-only content.
- Pattern: A schema-valid contract can still be operationally incomplete (for example no runnable tasks), which can produce misleading `doctor` readiness.
- Correction: Add explicit readiness checks for operational surface (at minimum one task for repo execution workflows).
- Rule: Treat schema validity and operational readiness as separate checks; `doctor` must fail on missing core execution entrypoints.

## 2026-03-25

- Pattern: A user asking for status or clarification may not be asking for implementation.
- Correction: Answer the status directly first, and only change code when the user explicitly asks to do so.
- Rule: Do not turn a confirmation question into a code change without a clear request to implement.

## 2026-04-07

- Pattern: Doctor output can be technically correct but still too repetitive for humans when every finding repeats the same `Next:` action.
- Correction: Group repeated doctor findings by shared remediation, keep the primary blocker explicit, and keep JSON unchanged.
- Rule: In human doctor output, group by action/family, not severity alone; collapse identical or equivalent `Next:` lines into one shared remediation block.
- Pattern: Grouping by literal `Next:` prose is too weak because equivalent operator actions still fan out once tool names or versions change.
- Correction: Derive grouping from normalized remediation/action classes, then render one shared remediation block with the existing Ota visual language.
- Rule: For doctor-style grouped output, normalize by operator action first and only fall back to exact remediation text when no stable action class exists.
- Pattern: CLI-level fallback guidance can reintroduce blank-line noise even when lower-level renderers are already structured correctly.
- Correction: Trim the boundary between structured error bodies and injected footer guidance before inserting fallback `Next:` lines.
- Rule: When adding fallback guidance above structured command output, preserve section spacing intentionally and never rely on existing trailing blank lines.
- Pattern: ANSI-styled separator lines can survive later blank-line collapsing and still show up as visible gaps in rendered output.
- Correction: Remove unwanted separators at the formatter source instead of relying on downstream blank-line cleanup.
- Rule: If a rendered spacing bug survives post-processing, tighten the original formatter string first; blank-line collapse is not a reliable fix for ANSI-decorated output.
- Pattern: Detect drift rendered as raw dotted field paths makes task changes look like schema noise instead of actionable repo updates.
- Correction: Present task removals by task name with concrete removal actions, and reserve raw field paths for non-task fallback cases.
- Rule: In human drift output, group by the operator’s unit of change when one exists; prefer `Task <name>` over raw `tasks.<name>.*` paths.
- Pattern: Child action markers that share the same hue family as parent bullets flatten hierarchy and make grouped output harder to scan.
- Correction: Give nested action bullets a clearly distinct accent color from their parent group/task markers while preserving the overall palette.
- Rule: In rich CLI output, use color to reinforce structure; child markers should not visually blend into parent bullets.
- Pattern: A roll-up line that sits visually like a sibling section can compete with the content it is supposed to frame.
- Correction: Render drift impact as a lightweight block directly under the section title, not as a full section headline or inline sentence fighting with the real warning groups.
- Rule: Summary/impact metadata should frame the next block, not visually rival it.
- Pattern: Existing-contract detect previews become less useful when the inferred contract dump appears before the actual comparison and drift review.
- Correction: Lead existing-contract detect text output with comparison/drift, then show the inferred contract and annotations as supporting detail.
- Rule: In review-oriented CLI flows, show the delta before the payload.
- Pattern: Successful orchestration commands can feel noisy and low-quality if captured backend logs are dumped by default.
- Correction: Suppress successful phase output on the happy path and reserve detailed command/service logs for failures or explicit inspection surfaces.
- Rule: Default success output should confirm the outcome and next step, not replay backend chatter.
- Pattern: Default readiness output can still feel internal and heavy even after the most important findings are reordered to the top.
- Correction: Render execution and agent detail as compact summary blocks with strong visual hierarchy instead of YAML-shaped detail dumps.
- Rule: In default human output, prefer compact operator summaries over schema-shaped blocks unless the extra structure is needed for actionability.
- Pattern: Raw task logs that are acceptable in an interactive terminal become noisy and low-trust when the same command runs in a captured or non-interactive context.
- Correction: Keep live streaming for interactive repo task runs, but buffer non-interactive output into a bounded excerpt with an explicit `--stream` escape hatch.
- Rule: When a command can be both interactive and captured, optimize the default presentation for the active context instead of forcing one output mode everywhere.
- Pattern: Premium sibling commands drift apart when some still end at a bare status while others close with obvious next steps.
- Correction: Give successful low-noise commands like `validate` explicit next actions, and reuse the same concise section naming across adjacent commands like `explain`.
- Rule: Core first-contact commands should not end in dead air; successful output should still point to the next useful Ota action.
