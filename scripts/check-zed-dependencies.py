#!/usr/bin/env python3
"""Validate bounded DPM integration and the canonical OTel Zed dependency."""

from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
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

print("validated bounded DPM CLI integration and canonical OTel Zed dependency")
