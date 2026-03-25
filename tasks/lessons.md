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
