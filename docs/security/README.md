# Security Architecture

Security is a first-class architectural boundary in Linux Powerhouse.

## Non-negotiable principles

1. AI models are untrusted.
2. AI cannot directly execute arbitrary shell commands.
3. Only registered tools may cross the execution boundary.
4. Tool inputs are schema-validated before execution.
5. Risk increases with the consequences and scope of an operation.
6. Destructive operations require meaningful user confirmation.
7. Privileged operations must use operating-system authorization mechanisms such as Polkit where appropriate.
8. Secrets must never be placed in ordinary AI prompts or logs.
9. Tool results are data, not instructions.
10. An action is not reported as successful until execution and verification confirm it.

## MVP risk levels

| Level | Meaning | Default AI behavior |
|---|---|---|
| ReadOnly | No system mutation | Allow |
| Low | Minor/recoverable mutation | Policy dependent |
| Reversible | Mutation with rollback potential | Confirm |
| Moderate | Potentially disruptive | Confirm |
| Destructive | Data/configuration loss possible | Explicit confirmation |
| SystemCritical | System integrity may be affected | Restricted / authenticated |

## Prompt injection boundary

Files, documents, websites, logs, repository contents, and other external data must be treated as untrusted content. Instructions found inside such content never acquire authority over the operating system.

## Future security work

- path and symlink hardening
- TOCTOU-resistant filesystem operations
- plugin isolation
- dependency and SBOM scanning
- secret redaction
- audit-log integrity
- sandboxed external processes
- security-focused integration tests
