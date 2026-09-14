#!/usr/bin/env python3
"""Validate bounded DPM integration and the canonical OTel Zed dependency."""

from pathlib import Path
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
ORES_OTEL_COMMIT = "4f9e7021b81339e44681632b4d86308cf113e54f"
ORES_OTEL_MANIFEST = (
    "https://raw.githubusercontent.com/ores-otel/ores.otel.log/"
    f"{ORES_OTEL_COMMIT}/.zpkg.toml"
)

manifest = tomllib.loads((ROOT / ".zpkg.toml").read_text(encoding="utf-8"))
dependencies = manifest.get("dependencies", {})
expected_dependencies = {
    "declarative-migrations/declarative-postgres-migrate": "^0.3.2",
    "oresoftware/next-loggers": "^0.1.0",
}

errors: list[str] = []
if dependencies != expected_dependencies:
    errors.append(
        "Zed dependencies must match the bounded DPM and canonical OTel dependency set: "
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
    "validated bounded DPM CLI integration and canonical OTel Zed dependency "
    f"at ores-otel/ores.otel.log@{ORES_OTEL_COMMIT}"
)
