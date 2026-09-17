#!/usr/bin/env python3
"""Dependency-free repository baseline validation for CI."""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_FILES = {
    "README.md",
    "SPECIFICATIONS.md",
    "FEATURES.md",
    "FEATURE-ROADMAP.md",
    "BENEFITS.md",
    "COMPETITIVE-OBJECTIVES.md",
    "BRANDING.md",
    "USER-MANUAL.md",
    "PRIVACY POLICY.md",
    "NOTES.md",
    "SECURITY.md",
    ".gitignore",
    ".editorconfig",
    "goreecloud.platform.yaml",
    "ARCHITECTURE.md",
    "CHANGELOG.md",
    "RIGHTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
}

EXPECTED_PLATFORM_SYSTEMS = {
    "manager",
    "privacy_shield",
    "wardveil_security",
    "everkeep",
    "glaze_ui",
    "mesh",
    "identity",
    "policy",
    "observability",
}


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_required_files() -> None:
    missing = sorted(name for name in REQUIRED_FILES if not (ROOT / name).is_file())
    if missing:
        fail(f"missing required repository files: {', '.join(missing)}")


def validate_platform_contract() -> None:
    path = ROOT / "goreecloud.platform.yaml"
    lines = path.read_text(encoding="utf-8").splitlines()
    text = "\n".join(lines)

    required_fragments = (
        'schema_version: "0.2"',
        "lifecycle: development",
        'version: "0.1.0"',
        'platform_contract: "0.2"',
        'glaze_ui_required: "1.5.1"',
    )
    for fragment in required_fragments:
        if fragment not in text:
            fail(f"platform contract is missing required fragment: {fragment}")

    try:
        start = lines.index("platform_systems:") + 1
    except ValueError as exc:
        raise SystemExit("ERROR: platform_systems section is missing") from exc

    found: set[str] = set()
    key_pattern = re.compile(r"^  ([a-z_]+):\s*$")
    for line in lines[start:]:
        if line and not line.startswith(" "):
            break
        match = key_pattern.match(line)
        if match:
            found.add(match.group(1))

    if found != EXPECTED_PLATFORM_SYSTEMS:
        fail(
            "platform_systems keys must be exactly: "
            + ", ".join(sorted(EXPECTED_PLATFORM_SYSTEMS))
            + f"; found: {', '.join(sorted(found))}"
        )

    if "result: applicable-blocked" not in text:
        fail("platform contract must keep unaccepted integrations blocked")


def validate_rust_workspace() -> None:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    if 'version = "0.1.0"' not in cargo:
        fail("workspace product version must remain 0.1.0")
    if 'rust-version = "1.98"' not in cargo:
        fail("workspace minimum Rust version must remain explicit")

    toolchain = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
    if 'channel = "1.98.1"' not in toolchain:
        fail("Rust toolchain must be pinned to the accepted 1.98.1 baseline")


def main() -> int:
    validate_required_files()
    validate_platform_contract()
    validate_rust_workspace()
    print("Repository baseline validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
