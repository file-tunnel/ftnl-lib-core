# Security policy

Report vulnerabilities privately through GitHub Security Advisories for this
repository. Do not open a public issue containing database URLs, credentials,
private schemas, customer records, File Tunnel capabilities, pairing fragments,
event tickets, filenames, local paths, or file contents.

The core library treats schema text as untrusted input. SQL identifiers are
strictly validated and record values are returned separately from parameterized
SQL. The DPM adapter never invokes a shell, never exposes `apply`, bounds time
and output, and redacts database arguments from `Debug` output.
