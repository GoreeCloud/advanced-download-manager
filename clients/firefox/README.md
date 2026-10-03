# GoreeCloud Download Manager Extension

**Status:** **0.2.17 source candidate** — compact Settings hierarchy and contextual native-permission UX under validation; accepted Stable remains 0.2.12

GoreeCloud Download Manager Extension is GoreeCloud's first-party Firefox Manifest V3 download manager. It provides managed queueing, pause/resume, retries, batch input, telemetry, and an optional separately installed Linux Native Messaging helper for segmented HTTP range transfers and durable same-job recovery.

Firefox add-on ID: `download-manager@goreecloud.com`  
Native Messaging host: `goreecloud_download_manager`  
Current source version: `0.2.17`  
Accepted Stable extension version: `0.2.12`  
Accepted native helper: `0.2.11` / protocol `2`

## 0.2.17 source-candidate polish

0.2.17 refines the Settings hierarchy based on representative 0.2.16 UI review. When the Firefox download engine is selected, the native-acceleration section now collapses to a compact summary instead of rendering a large disabled form. Selecting the native segmented helper expands the acceleration, retry, destination, authenticated-cookie, and helper-test controls.

Optional cookie permission actions are now contextual rather than permanently visible. **Allow cookie access…** appears only when native mode and cookie forwarding are both selected and permission is absent. **Revoke cookie access** appears only when permission is currently granted. This removes disabled destructive-looking controls from the ordinary Firefox-mode view while preserving the 0.2.16 least-privilege permission gate.

The per-download destination explanation is shorter and keeps the same contract: GoreeCloud-started Firefox-engine transfers always open Save As, while Firefox-started downloads use Firefox's own **Always ask you where to save files** preference. The primary action is renamed **Save changes**.

0.2.17 changes Settings presentation only. It carries forward 0.2.16 permission gating/revocation, 0.2.15 mandatory Save As behavior, and 0.2.14 non-destructive automatic Firefox-download adoption. Native helper 0.2.11 / protocol 2 is unchanged.

0.2.17 is a source candidate only. Accepted Stable remains 0.2.12 until applicable Mozilla signing, persistent-install/runtime validation, and explicit lifecycle promotion succeed.

## 0.2.16 source-candidate fix

0.2.16 hardens the Settings privacy and information architecture exposed during representative 0.2.15 testing. General Firefox-download controls and native-acceleration controls are now separated into distinct Settings sections. When **Firefox downloads — maximum compatibility** is selected, native-only controls are visibly inactive and disabled.

The optional Cookies + All Sites permission can no longer be requested merely because the permission button is visible. **Allow cookie access…** is enabled only when the native segmented helper is selected **and** target-site cookie forwarding is deliberately turned on. The Settings page discloses Firefox's broad permission wording before the request and includes **Revoke cookie access** after a grant.

Cookie permission acquisition remains directly bound to the explicit user click required by Firefox. Saving settings never attempts to manufacture a permission request after asynchronous work, and cookie forwarding remains off by default. A browser-engine configuration can be saved without cookie permission because the native-only forwarding path is inactive.

0.2.16 carries forward 0.2.15 mandatory per-download Save As behavior for GoreeCloud-started Firefox-engine transfers and 0.2.14 non-destructive automatic adoption. No new native-helper protocol or download transport behavior is claimed in 0.2.16.

0.2.16 is a source candidate only. Accepted Stable remains 0.2.12 until applicable Mozilla signing, persistent-install/runtime validation, and explicit lifecycle promotion succeed.

## 0.2.15 source-candidate fix

0.2.15 makes per-download destination choice mandatory for Firefox-engine downloads that GoreeCloud itself starts through the popup, Manager, context-menu action, batch queue, retry path, or browser fallback. Those launches call Firefox's downloads API with `saveAs: true`, which opens Firefox's Save As dialog for each newly started transfer instead of silently using one automatic location.

Automatically captured Firefox-started downloads remain deliberately non-destructive. By the time the WebExtension receives Firefox's `downloads.onCreated` event, the original request has already begun; cancelling and replaying it just to force a dialog could change POST bodies, short-lived authorization state, container/session context, signed URLs, service-worker behavior, or other request semantics. For those captured downloads, Firefox's own **Always ask you where to save files** preference controls the destination dialog before GoreeCloud adopts the original download ID.

Existing Firefox downloads that are merely adopted are never replayed. The Settings page explains the distinction between GoreeCloud-started Save As prompting and Firefox's own prompt preference for automatically captured downloads. Native segmented downloads continue to use the native destination-directory contract; a cross-platform native system-save-dialog contract is not claimed in 0.2.15.

0.2.15 is a source candidate only. Accepted Stable remains 0.2.12 until applicable Mozilla signing, persistent-install/runtime validation, and explicit lifecycle promotion succeed.

## 0.2.14 source-candidate fix

0.2.14 adds default-on automatic adoption of ordinary HTTP/HTTPS downloads that Firefox starts outside GoreeCloud controls. The extension listens for new Firefox downloads, records them as managed Firefox-engine jobs, and retains the original Firefox download ID so pause, resume, cancel, history, telemetry, and terminal notifications can operate without requiring the user to paste the URL manually.

Automatic adoption intentionally does not cancel or replay the originating request. This preserves Firefox's original request semantics and avoids converting authenticated, POST-backed, signed-URL, service-worker-influenced, or otherwise browser-context-sensitive transfers into a new request. Downloads started by this extension are ignored by the adoption listener to prevent duplicate jobs, unsupported non-HTTP(S) schemes remain outside capture, and Settings includes a default-on **Automatically manage Firefox downloads** control.

When native segmented mode is selected, an already-started Firefox download that is automatically adopted remains on the Firefox engine. Native acceleration continues to apply only when GoreeCloud itself initiates the download through its popup, Manager, context-menu action, or another explicitly managed start path. 0.2.14 is a source candidate only; accepted Stable remains 0.2.12 until Mozilla signing, persistent-install/runtime validation, and explicit promotion succeed.

## 0.2.13 source-candidate fix

0.2.13 corrects Firefox context-menu URL selection for media wrapped in links. When Firefox supplies both `srcUrl` and `linkUrl` for an image, video, or audio element, GoreeCloud Download Manager now prefers the media source URL. Plain link context actions continue to use `linkUrl` when no media source exists.

The regression is covered by the Firefox scheduler harness using an image-context case that includes both URLs and a plain-link fallback case. This source change does not alter the accepted native helper, Native Messaging protocol, cookie-forwarding model, or the accepted Stable 0.2.12 release. 0.2.13 must complete its own validation, Mozilla signing, persistent-install/runtime acceptance, and explicit lifecycle promotion before it can replace 0.2.12 as Stable.

## Stable 0.2.12 release evidence

GoreeCloud Download Manager Extension 0.2.12 is the accepted Stable Firefox release for Mozilla unlisted/self-distribution.

Governed signing/restart run `34176105690` validated exact source revision `2cc6d3bbe6ec2c63d49bec338bd68f154747be70`, built the deterministic 0.2.12 candidate, submitted a new unlisted version to Mozilla, verified the returned signed XPI against the candidate, installed the signed extension persistently, and exercised the full browser restart/native recovery gate on Firefox 155.0.1.

Release hashes and retained evidence:

- deterministic candidate SHA-256: `779425b150921c1969462066a3e79cb345d976d11369a6891b5611c63a3d5537`;
- Mozilla-signed XPI SHA-256: `4c02a152a258c4f8e76581ece2cb2a41f088463a4464354da0c374dfb2957f25`;
- signing source: `new-submission`;
- retained artifact: `goreecloud-download-manager-0.2.12-mozilla-signed`, artifact ID `10037385022`;
- artifact ZIP SHA-256: `17ba5f469b04979f5405f618abae5fc461e4e5c583f10e2d6484fe9706b6e747`.

The signed non-manifest runtime payload matched the candidate byte-for-byte. Mozilla's manifest change was accepted only as governed JSON-serialization normalization.

The full restart test started a native eight-segment transfer, captured the exact GoreeCloud job identity, confirmed validated partial staging, exited the complete Firefox process, and reopened the same profile without reinstalling the add-on. The signed extension survived restart and automatically resumed the same job through HTTP Range requests beginning inside preserved segments. The transfer completed at 67,108,864 bytes, required **zero manual Resume actions**, removed the original staging directory, and reconnected to the native helper. Source and recovered output SHA-256 both equaled `a4a99d83daaac4823006cd3b14df26d1a256042591ad7d2f83e7ecbb203c342f`.

## Why 0.2.12 followed 0.2.11

Mozilla-signed 0.2.11 had already passed the same full restart/recovery gate after fixing the binary segmented-publication defect found in 0.2.10. A final packaged-runtime audit then found that the 0.2.11 Settings page still displayed `GoreeCloud Download Manager Extension 0.2.11 source candidate`.

That self-description contradicted an eventual Stable lifecycle even though the runtime behavior had passed. GoreeCloud therefore withheld 0.2.11 Stable promotion rather than mutating an already-signed version.

0.2.12 changes the Firefox manifest version and makes the packaged Settings heading lifecycle-neutral: `GoreeCloud Download Manager Extension 0.2.12`. A regression prevents packaged Settings UI from embedding `source candidate`, `not Stable`, or a hard-coded Stable status. Lifecycle authority belongs to `clients/firefox/release-state.json` and retained release evidence.

No native-helper behavior changed in 0.2.12. The accepted helper remains **0.2.11**, which includes the binary segmented-assembly correction and protocol-2 security/recovery contracts.

## Implemented

- Firefox `downloads` API engine for maximum browser compatibility.
- Optional native segmented engine with 1–32 HTTP byte-range workers.
- Shared Firefox/native managed-download concurrency enforcement.
- Pause, resume, cancel, retry, pause-all, resume-all, and clear-completed actions.
- Batch URL queueing and link/media context-menu capture.
- Default-on automatic adoption of ordinary HTTP/HTTPS downloads started by Firefox, with a Settings opt-out and no cancellation/replay of the originating request.
- Live bytes, progress, rolling speed, ETA, queue position, engine state, and effective native segment count.
- Persistent native staging under `.goreecloud-downloads/<job-id>/`.
- Same-job native recovery after helper interruption, Firefox background-context recreation, and accepted full Firefox process restart.
- Fail-closed staged-metadata validation before persisted partial bytes are eligible for reuse.
- URL/size/ETag/Last-Modified source-identity checks before staged partial reuse.
- Strict native HTTP 206 / `Content-Range` validation before resumed or segmented bytes are appended.
- Native staging symlink rejection and no-follow regular-file opens for the Linux helper.
- Collision-safe native destination reservation and no-overwrite final publication from staging.
- Binary-safe segmented assembly before final publication.
- Versioned extension/native protocol negotiation with required capability validation.
- Optional target-site cookie forwarding behind explicit Firefox optional permission.
- Completion/failure notifications.
- Deterministic queue ordering and migration-safe queue-sequence reconciliation.
- Lifecycle-race protection for cancellation, pending Firefox launches, late browser/native events, and removed jobs.
- Ordinary Retry preservation of effective engine/configuration/requested-filename snapshots.
- Cross-platform requested-filename normalization for browser retries.
- GoreeCloud product branding and Glaze-aligned popup, Manager, and Settings interfaces.

## Native publication and staging safety

The 0.2.11 helper fixed a native publication defect exposed by the signed 0.2.10 restart diagnostic. Segmented assembly had opened `assembled.part` with text-exclusive mode (`"x"`) and attempted to write binary byte chunks, causing `TypeError: write() argument must be str, not bytes` after all source bytes had recovered. The helper now uses binary-exclusive mode (`"xb"`), preserving exclusive-create and no-follow protections while allowing exact binary assembly.

The helper validates the `.goreecloud-downloads` staging root and per-job staging path with `lstat` semantics and rejects symbolic links or unexpected non-directories. Reusable metadata, single-part, segment, and assembled staging entries must be regular non-link files. Supported file opens use `O_NOFOLLOW`; metadata replacement is staged through an exclusive job-local temporary file with flush/fsync; invalid link entries are removed as entries rather than followed; and final no-overwrite publication validates the staging source before `os.link(..., follow_symlinks=False)`.

These controls materially reduce link-following and accidental overwrite risk. They do not claim a universal filesystem sandbox against another unrestricted process running as the same OS user.

## Persisted staging trust and recovery

A valid `metadata.json` record is required before persisted partial bytes can be reused. The record must use the supported schema, belong to the exact GoreeCloud job ID, contain a valid canonical HTTP/HTTPS URL and supported source-size representation, and keep bounded filename/destination/ETag/Last-Modified values. Structurally valid metadata must still pass URL, known size, ETag, and Last-Modified identity checks.

Interrupted or errored native jobs that already started are recovered using the **same GoreeCloud job ID**. Recovery preflights a protocol-compatible native helper, preserves the existing job/configuration, and reconstructs missing helper state from validated job-scoped staging.

Ordinary **Retry** is intentionally different: it creates a fresh GoreeCloud job at the queue tail and does not reuse native partial staging, while preserving the source job's effective engine/configuration/requested-filename snapshot.

## Cookie forwarding

Cookie forwarding is off by default. In 0.2.16, Settings enables **Allow cookie access…** only after the native segmented helper and cookie forwarding are both deliberately selected. Firefox's optional Cookies + All Sites permission is still requested directly from that explicit user action. Settings also exposes **Revoke cookie access** after a grant. When forwarding is enabled and permitted, cookies for the target URL are forwarded in memory to the local native helper and are not intentionally written to managed download history or native recovery metadata.

Accepted Firefox 155.0.1 / Flathub Flatpak testing previously demonstrated authenticated HEAD plus eight authenticated HTTP 206 ranges across a controlled 256 MiB source, exact final integrity, staging cleanup, and controlled credential non-persistence in native staging and `browser.storage.local`.

Sites requiring arbitrary request bodies, JavaScript-generated authorization, DRM, anti-bot challenges, service-worker state, or other browser-only request state are not guaranteed to work through the native engine.

## Linux native helper

Install from the repository root:

```bash
./clients/firefox/scripts/install-native-host-linux.sh
```

The helper is copied to:

```text
~/.local/lib/goreecloud-download-manager/goreecloud_download_manager_native.py
```

and registered at:

```text
~/.mozilla/native-messaging-hosts/goreecloud_download_manager.json
```

For Stable extension 0.2.12, the accepted helper remains version `0.2.11`, protocol `2`. The installer compiles the helper and requires startup/ping hello frames with the required protocol capabilities, including `staging-link-rejection`.

For Firefox Flatpak, use the WebExtensions XDG portal path. If discovery fails after a successful helper self-test, open `about:config`, set `widget.use-xdg-desktop-portal.native-messaging` to `1`, restart Firefox, and approve the portal authorization prompt.

## Development packaging

Unsigned development builds can still be loaded temporarily through `about:debugging` → **This Firefox** → **Load Temporary Add-on**. Temporary loading does not replace or supersede Stable release evidence.

Build the deterministic unsigned package with:

```bash
python clients/firefox/scripts/package.py
```

For the Stable source line this produces:

```text
dist/goreecloud-download-manager-0.2.12.xpi
```

The XPI excludes the separately installed native helper and source-only scripts/tests/documentation. An unsigned package is not the accepted Mozilla-signed Stable artifact.

## Validation

```bash
node --check clients/firefox/native_protocol.js
node --check clients/firefox/background.js
node --check clients/firefox/recovery.js
node --check clients/firefox/scheduler_hardening.js
node clients/firefox/tests/test_browser_scheduler.js
node clients/firefox/tests/test_mixed_scheduler.js
node clients/firefox/tests/test_lifecycle_faults.js
node clients/firefox/tests/test_retry_snapshots.js
node clients/firefox/tests/test_auto_capture.js
node --check clients/firefox/ui/popup.js
node --check clients/firefox/ui/manager.js
node --check clients/firefox/ui/options.js
python -m py_compile clients/firefox/scripts/native-host/goreecloud_download_manager_native.py
python -m unittest discover -s clients/firefox/tests -p 'test_*.py'
python -m json.tool clients/firefox/release-state.json >/dev/null
python clients/firefox/scripts/package.py
```

## Current boundaries

Stable 0.2.12 does not establish Windows/macOS native-host support, arbitrary POST/body downloads, complete browser authorization-state reproduction, mirror failover, bandwidth limiting, time scheduling, automatic native takeover/request replay of Firefox-started downloads, or origin/user-supplied cryptographic checksum enforcement. The 0.2.16 source candidate carries forward non-destructive automatic adoption and per-download Save As prompting while adding native-settings/optional-permission UX hardening; these later-version changes are not yet accepted Stable capabilities. The supported Stable scope is the Firefox extension plus the separately installed Linux native helper and the behaviors actually covered by the accepted evidence.

## Release state

**GoreeCloud Download Manager Extension 0.2.12 remains the accepted Stable release.** The application-local Firefox release-state record now identifies 0.2.17 as a source candidate while retaining `accepted_stable_version: 0.2.12`. The 0.2.17 candidate must independently repeat its applicable validation, Mozilla signing, signed-install/runtime acceptance, integrity, review, and promotion gates before replacing 0.2.12 as Stable.
