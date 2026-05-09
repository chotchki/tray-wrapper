# PLAN.md

Completed phases: see [`PLAN_ARCHIVE.md`](./PLAN_ARCHIVE.md).

## Phase 6 — Address Dependabot alerts

GitHub flagged 3 alerts on `main` after the v0.4.0 push: `bytes` (medium), `rand` (low), and `glib` (medium). The first two are easy patch bumps on transitive deps. The glib alert is blocked upstream — see 6.3.

- [x] 6.1 `cargo update -p bytes` to pull `1.11.1+` (fixes GHSA-434x-w66g-qw3r — integer overflow in `BytesMut::reserve`). Transitive dep, no API change for us.
- [x] 6.2 `cargo update -p rand` to pull `0.8.6+` (fixes GHSA-cq8v-f236-94qc — unsoundness with custom logger). Transitive dep, no API change for us.
- [x] 6.3 Document why the `glib` alert (GHSA-wrw7-89jp-8q8g, fixed in `0.20.0`) is not patched in this phase. Vulnerable `glib 0.18.5` enters the tree via `gtk 0.18.2` (the final, **unmaintained** version of the gtk3 binding — crates.io says "use gtk4 instead"). Fixing requires migrating from `gtk = "0.18"` to `gtk4`, which is breaking and substantial. Accept the alert for now; track follow-up in a future phase. The vulnerability is in `glib::VariantStrIter`'s `Iterator`/`DoubleEndedIterator` impls and only affects the Linux build, while the primary target is macOS.

**Phase exit:** `cargo update` completes, `Cargo.lock` shows `bytes >= 1.11.1` and `rand >= 0.8.6`, full test suite green, two of three GitHub alerts auto-close on next push, and the `glib` alert has a documented disposition.

