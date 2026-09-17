#!/usr/bin/env python3
"""Validate bounded DPM integration and canonical OTel/lifecycle Zed dependencies."""

from pathlib import Path
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
ORES_OTEL_COMMIT = "4f9e7021b81339e44681632b4d86308cf113e54f"
ORES_OTEL_MANIFEST = (
    "https://raw.githubusercontent.com/ores-otel/ores.otel.log/"
    f"{ORES_OTEL_COMMIT}/.zpkg.toml"
)
NEXT_LOGGERS_COMMIT = "e0eaa55fd964827bb3a9ef580f076cce23000cdb"
NEXT_LOGGERS_MANIFEST = (
    "https://raw.githubusercontent.com/ORESoftware/next-loggers.ts/"
    f"{NEXT_LOGGERS_COMMIT}/.zpkg.toml"
)

manifest = tomllib.loads((ROOT / ".zpkg.toml").read_text(encoding="utf-8"))
dependencies = manifest.get("dependencies", {})
expected_dependencies = {
    "declarative-migrations/declarative-postgres-migrate": "^0.3.2",
    "oresoftware/next-loggers": "^0.1.0",
    "oresoftware/next-loggers-rust": "^0.1.0",
}

errors: list[str] = []
if dependencies != expected_dependencies:
    errors.append(
        "Zed dependencies must match the bounded DPM and canonical OTel/lifecycle dependency set: "
        f"{expected_dependencies!r}"
    )

try:
    with urllib.request.urlopen(ORES_OTEL_MANIFEST, timeout=15) as response:
        upstream_otel = tomllib.loads(response.read().decode("utf-8"))
except Exception as exc:
    errors.append(f"failed to read immutable OTel package manifest {ORES_OTEL_COMMIT}: {exc}")
else:
    expected_upstream = {
        "org": "oresoftware",
        "name": "next-loggers",
        "version": "0.1.0",
        "repository": "https://github.com/ores-otel/ores.otel.log",
        "interfaces": "^0.1.0",
    }
    actual_upstream = {
        "org": upstream_otel.get("package", {}).get("org"),
        "name": upstream_otel.get("package", {}).get("name"),
        "version": upstream_otel.get("package", {}).get("version"),
        "repository": upstream_otel.get("package", {}).get("repository", {}).get("url"),
        "interfaces": upstream_otel.get("dependencies", {}).get("ores-otel/ores-interfaces"),
    }
    if actual_upstream != expected_upstream:
        errors.append(
            "immutable OTel package provenance does not match expected next-loggers identity: "
            f"{actual_upstream!r}"
        )

try:
    with urllib.request.urlopen(NEXT_LOGGERS_MANIFEST, timeout=15) as response:
        lifecycle_manifest = tomllib.loads(response.read().decode("utf-8"))
except Exception as exc:
    errors.append(
        f"failed to read immutable next-loggers lifecycle manifest {NEXT_LOGGERS_COMMIT}: {exc}"
    )
else:
    rust_target = lifecycle_manifest.get("targets", {}).get("rust", {})
    expected_rust_target = {
        "dir": "sdk/rust",
        "name": "next-loggers-rust",
        "adapter": "rust",
    }
    actual_rust_target = {
        "dir": rust_target.get("dir"),
        "name": rust_target.get("name"),
        "adapter": rust_target.get("adapter"),
    }
    if actual_rust_target != expected_rust_target:
        errors.append(
            "immutable next-loggers manifest does not expose the canonical Rust lifecycle target: "
            f"{actual_rust_target!r}"
        )

adapter = (ROOT / "src/dpm.rs").read_text(encoding="utf-8")
for token in ["Command::new", "DpmOperation::Diff", "DpmOperation::Verify", "DpmOperation::Bootstrap"]:
    if token not in adapter:
        errors.append(f"DPM adapter is missing {token}")
if "DpmOperation::Apply" in adapter or '"apply"' in adapter:
    errors.append("DPM apply must not be representable in ftnl-lib-core")

if errors:
    print("ftnl-lib-core dependency validation failed:")
    for error in errors:
        print(f" - {error}")
    raise SystemExit(1)

print(
    "validated bounded DPM CLI integration, canonical OTel dependency, and shared Rust lifecycle "
    f"target at ORESoftware/next-loggers.ts@{NEXT_LOGGERS_COMMIT}"
)
