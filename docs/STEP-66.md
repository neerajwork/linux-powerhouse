# Step 66 — Scoped Action Authorization

## Goal

Establish a deterministic authorization contract between explicit action confirmation and any future execution, without granting authorization implicitly and without executing an alert action in this step.

## Safety boundary

- Confirmation records user intent only.
- Authorization must be explicit and tied to the exact action identifier.
- Authorization does not execute an action.
- Authorization must not broaden the action scope or privilege requirements.
- Unknown action identifiers are rejected.
- Privileged actions remain ineligible on the current non-privileged path.
- No shell execution, process termination, file deletion, service restart, network mutation, or privilege escalation is introduced.

## Acceptance goals

1. Represent an explicit authorization result separately from confirmation.
2. Bind authorization to the exact confirmed action identifier.
3. Require a matching confirmation before authorization can succeed.
4. Preserve the preview's safety and privilege metadata.
5. Keep authorization local, deterministic, serializable, and non-mutating.
6. Keep actual execution as a separate later capability.

## Rationale

Step 65 established whether a confirmed action satisfies execution eligibility checks. Step 66 makes the authorization boundary explicit so that a later execution engine cannot infer permission merely from a confirmation record or from a generic boolean.
