# Performance Budget

These are rough V1 targets, not hard guarantees yet.

## Command targets

- CLI startup should feel immediate on a normal developer machine
- `ota doctor` on a medium repo should complete in low single-digit seconds when checks are reasonable
- detection should stay source-targeted rather than broad-crawling the repo
- memory usage should stay modest and proportional to the small contract and detection surface

## Resource rules

- commands should remain primarily stateless
- no long-lived background daemons in V1
- concurrency, when introduced, should be bounded
- large repositories should degrade gracefully rather than explode in file scanning or process count
