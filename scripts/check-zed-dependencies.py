#!/usr/bin/env python3
"""Validate that the DPM package edge corresponds to the bounded CLI adapter."""

from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((ROOT / ".zpkg.toml").read_text(encoding="utf-8"))
dependencies = manifest.get("dependencies", {})
expected = "declarative-migrations/declarative-postgres-migrate.rs"

errors: list[str] = []
if dependencies != {expected: "^0.3.2"}:
    errors.append(f"Zed dependencies must contain only {expected} at ^0.3.2")

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

print("validated bounded DPM CLI integration")
