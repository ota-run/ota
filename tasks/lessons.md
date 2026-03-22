# Lessons

## 2026-03-21

- Pattern: Source files need valid repository license notes, but non-language comment formats or decorative Unicode headers can break compilation.
- Correction: Use a short ASCII language-valid license header in Rust source files and verify the compiler still accepts the file afterward.
- Rule: When adding or restoring license notes in code, use the target language's native comment syntax only and avoid decorative Unicode banners.
