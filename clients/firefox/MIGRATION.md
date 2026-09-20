# Firefox Client Repository Migration

## Canonical ownership

GoreeCloud Download Manager Extension is an application-owned Firefox client of GoreeCloud Advanced Download Manager.

Canonical source after this migration:

- Repository: `GoreeCloud/advanced-download-manager`
- Path: `clients/firefox/`
- Firefox add-on ID: `download-manager@goreecloud.com`
- Stable Firefox version: `0.2.12`
- Accepted native helper: `0.2.11` / protocol `2`

The client must not return to a separate or shared repository as its permanent source authority.

## Source provenance

The migrated source was copied byte-for-byte from:

- Repository: `GoreeCloud/goreecloud-firefox-extensions`
- Path: `extensions/download-manager/`
- Reviewed source revision: `e7f502b15b706864268ddec18e327f12840f3b82`

Packaged runtime files were not rewritten during the repository move. Source-only documentation, validation paths, and workflow references were then reconciled from `extensions/download-manager/` to `clients/firefox/`.

The shared Firefox repository remains historical provenance after the source-authority transition and must not be treated as the canonical Download Manager Firefox source after migration acceptance.

## Release preservation gate

The accepted 0.2.12 deterministic unsigned candidate SHA-256 is:

`779425b150921c1969462066a3e79cb345d976d11369a6891b5611c63a3d5537`

The accepted Mozilla-signed XPI SHA-256 is:

`4c02a152a258c4f8e76581ece2cb2a41f088463a4464354da0c374dfb2957f25`

Application-repository CI must reproduce the accepted unsigned candidate hash from the migrated 0.2.12 source before the source-authority migration is accepted.

Stable status remains version-specific. Moving the repository location does not create a new Firefox release and does not promote the Development Rust application core.

## Signing workflow

The governed Mozilla signing/restart workflow is migrated to:

`.github/workflows/download-manager-mozilla-signing.yml`

It reads lifecycle authority from `clients/firefox/release-state.json` and retains the existing branch-binding, unlisted Mozilla signing, payload-verification, persistent-install, full-Firefox-restart, same-job native-recovery, and evidence-retention gates.

Repository secret availability is not inferred from workflow source. `AMO_JWT_ISSUER` and `AMO_JWT_SECRET` must be authorized for this repository before a future signing execution can succeed.

## Shared-repository cleanup — Complete

The application-owned Firefox migration completed on 2026-09-18.

- Owning application migration: Advanced Download Manager PR #14 merged to authoritative `main` at `fbde048fccd8154e462793134984e9d587d31ecd`.
- Owning application post-merge validation: `Download Manager Firefox Client` run #4 and `Core Foundation` run #55 both passed.
- Shared Firefox cleanup: `GoreeCloud/goreecloud-firefox-extensions` PR #104 removed the transitional `extensions/download-manager/` copy, shared Download Manager signing workflow, and shared validation/package ownership.
- Shared repository post-merge validation: Firefox Repository run #398 passed.
- Current canonical source: `GoreeCloud/advanced-download-manager/clients/firefox/`.
- Firefox add-on ID remains `download-manager@goreecloud.com`.
- Accepted Stable Firefox version remains `0.2.12`, with accepted native helper `0.2.11` / protocol `2` and the preserved accepted candidate/signed XPI hashes recorded above.

Historical source provenance remains available through Git history in the shared Firefox repository. The shared repository is no longer an active source authority or signing authority for Download Manager Firefox development.
