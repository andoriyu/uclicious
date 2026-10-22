# FD-1017 Uclicious extraction verification

Source: Foundry f3499aa4ed0113aa7efa48b8dfd462c84a63cab5.
All 45 selected source/publisher files matched byte-for-byte before adaptation.
The filtered history retains 19 commits; the map and manifest are adjacent.
Main/derive/sys representative source histories originate at Foundry import
4bc63094; no earlier project paths were found through follow history.

Preserved crate manifests, names, versions and publishing intent:
uclicious 0.10.1, uclicious_derive 0.10.1, uclicious-libucl-sys 0.9.2.
No runtime source or published crate manifests changed.

Validation in standalone colocated jj checkout, without Foundry/sibling inputs:
- Shared CI profile formatting and Clippy: passed.
- Nextest with coverage: 49 tests passed.
- Cargo doctests: seven passed.
- Nix x86_64-linux default workspace package: passed with libucl.
- No-default-features library compilation: passed.
- Read-only publishing plan: all three versions already exist; nothing to publish.
- Actionlint with explicit beefy runner label: passed.
- Alejandra flake/devenv checks: passed.

Additional no-default-features integration test attempt fails because the
existing derive integration suite imports feature-gated derive APIs. This is
unchanged source behavior; the default-feature suite and no-default library
compilation pass. Feature-gating that test suite is outside this extraction.
The pinned nixpkgs rejects x86_64-darwin (support removed upstream), so
the flake exposes x86_64-linux, aarch64-linux, and aarch64-darwin. Other
architectures are evaluated but not locally built. A separate clean checkout
also passed workspace formatting, all-target Clippy, tests and doctests.

Workflow/input pins use verified Molds bd0ccfff73dbc6845f3263b9e722870bf8be4424.
PR publishing plans are read-only. Actual publishing runs only on master after
Rust, Nix and plan checks; preserve the registry-aware topological publisher.
All FDs remain in Coldsteel/foundry. The owner merges PRs.
Foundry source/automation cleanup stays on the existing local extraction branch;
no Foundry push or PR is authorized.
