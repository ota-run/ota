# Lessons

## 2026-03-21

- Pattern: Source files need valid repository license notes, but non-language comment formats or decorative Unicode headers can break compilation.
- Correction: Use a short ASCII language-valid license header in Rust source files and verify the compiler still accepts the file afterward.
- Rule: When adding or restoring license notes in code, use the target language's native comment syntax only and avoid decorative Unicode banners.

## 2026-03-24

- Pattern: Docs pages that only list commands are low adoption value and create user confusion.
- Correction: Rewrite command/docs pages to include when-to-use, why, and concrete use-cases with practical command examples.
- Rule: For OSS-facing docs updates, require adoption-first guidance (when/why/use-case), not enumeration-only content.
