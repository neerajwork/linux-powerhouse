# Architecture

Linux Powerhouse is structured as a set of small, testable boundaries rather than one privileged application process.

## Current bootstrap layers

```text
GUI / future Tauri app
        │
        ▼
AI provider abstraction
        │
        ▼
Tool Registry
        │
        ▼
Policy Engine
        │
        ▼
Execution Engine
        │
        ▼
Linux backends
```

### `powerhouse-core`
Shared domain primitives such as execution identifiers and operation status.

### `tool-registry`
Defines user-oriented capabilities. A tool describes what Powerhouse can do, its risk, required permissions, and whether AI may invoke it autonomously.

### `policy-engine`
Provides deterministic authorization decisions independent of any AI model.

### `execution-engine`
The only layer that will eventually be allowed to cross from validated intent into operating-system side effects.

## Design rule

AI output is untrusted input. It must never directly become a shell command or privileged operation.

The execution path is:

```text
Natural language
    ↓
AI structured intent
    ↓
Tool Registry validation
    ↓
Policy evaluation
    ↓
Permission / confirmation
    ↓
Execution
    ↓
Verification
    ↓
Audit record
```
