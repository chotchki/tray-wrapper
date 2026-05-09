# PLAN.md

Completed phases: see [`PLAN_ARCHIVE.md`](./PLAN_ARCHIVE.md).

## Phase 5 — Coverage report visibility

Coverage and badge are already wired up (`.github/workflows/code_coverage.yml` runs grcov on macOS, uploads to codecov; README has the badge). This phase makes the existing setup more useful as a code-review signal and verifies the Phase 3 unit tests show up in the next report.

- [x] 5.1 Add a `comment:` block to `.codecov.yml` so each PR gets an automatic Codecov comment showing the per-file coverage diff. Use a simple layout (`reach, diff, flags, files`) and `require_changes: false` so the comment posts even when the diff doesn't move coverage (useful for confirming "yes I checked").

**Phase exit:** `.codecov.yml` includes a `comment:` block; after Phase 3 + this lands on main, the next codecov run reflects the new unit tests (badge percentage moves up); the next PR opened against main shows an automatic Codecov comment.
