# Linux Powerhouse Localization Architecture

**Document:** Localization Architecture and Implementation Specification  
**Version:** 1.0  
**Status:** Architecture and planning approved; implementation not started  
**Initial languages:** English (`en`) and Kannada (`kn`)  
**Frontend:** React, TypeScript, and Vite  
**Desktop framework:** Tauri 2  
**Backend:** Rust

## 1. Purpose and scope

This document defines the approved localization architecture and the planned implementation sequence for Linux Powerhouse. The objective is to support multiple display languages without compromising system-monitoring accuracy, safe-action boundaries, audit integrity, accessibility, or application reliability.

The initial languages are English and Kannada. The architecture should permit additional languages, regional variants, and future right-to-left (RTL) support without requiring a redesign.

This is an architecture and planning document. It does not authorize or describe completed implementation. No localization code or dependencies are implied to exist by this specification.

## 2. Architectural principles

1. Keep application and system logic independent of display language.
2. Separate translation resources from application code.
3. Use stable identifiers for internal states, actions, and backend messages.
4. Centralize locale detection, normalization, persistence, and formatting.
5. Preserve existing persisted data and historical audit records.
6. Introduce localization incrementally, with English functional throughout migration.
7. Use English as the permanent fallback language.
8. Prepare layout and text handling for RTL languages.
9. Preserve accessibility and technical accuracy.
10. Require feature-level regression testing and existing CI validation throughout migration.

## 3. Technology and locale model

The approved technology direction is:

- `i18next` for translation resource management.
- `react-i18next` for React integration.
- JSON translation resources.
- BCP 47 locale identifiers.
- JavaScript `Intl` APIs for locale-aware formatting.
- Typed Rust enums for known backend message codes, serialized to stable string identifiers.

English (`en`) is the default and fallback language. Kannada (`kn`) is the first additional language. Regional identifiers such as `kn-IN` may be used for locale selection and formatting while resolving translation resources to the supported language resource.

Pluralization should use i18next's native plural handling and locale plural rules. Kannada plural behavior must be explicitly tested. An ICU message-format plugin is not part of the initial approved dependency plan.

## 4. Translation resource architecture

Use feature-based namespaces to keep resource ownership and review manageable.

| Namespace | Responsibility |
| --- | --- |
| `common` | Shared controls, labels, and accessibility text |
| `app` | Application shell, navigation, and global messages |
| `dashboard` | System overview and resource information |
| `health` | System health, diagnostics, and health states |
| `actions` | Safe actions, confirmations, and verification |
| `alerts` | Alert controls, decisions, and preferences |
| `alertHistory` | Alert event history, filters, and event details |
| `insights` | Trends, summaries, and comparisons |
| `performance` | Performance monitoring and anomalies |
| `audit` | Action audit history and outcomes |
| `errors` | Known application-facing error messages |

Translation keys must be semantic, descriptive, and use `camelCase`. Keys must not encode visual position or implementation details, and must remain stable when visible wording changes.

Example key: `health.subsystems.cpu`.

Dynamic text must use named interpolation parameters. Pluralized messages must use i18next plural forms and receive a numeric `count` parameter. Missing keys should fall back to English; development builds should make missing keys diagnosable without exposing raw translation keys to production users.

## 5. Proposed resource layout

The initial resource layout is:

```text
apps/desktop/src/localization/
├── index.ts
├── i18n.ts
├── locale.ts
├── formatting.ts
└── resources/
    ├── en/
    │   ├── common.json
    │   ├── app.json
    │   ├── dashboard.json
    │   ├── health.json
    │   ├── actions.json
    │   ├── alerts.json
    │   ├── alertHistory.json
    │   ├── insights.json
    │   ├── performance.json
    │   ├── audit.json
    │   └── errors.json
    └── kn/
        └── ...
```

This is a proposed organization, not a requirement to create every file at once. The implementation may consolidate small helper modules where that better fits the existing application structure, while preserving the namespace boundaries.

## 6. Backend message contract

Rust business logic must remain language-neutral. Known application-generated messages should be represented by typed Rust enums and serialized into stable string codes that the frontend can resolve through translation resources.

The message contract should support:

- A stable message code.
- Message-specific structured interpolation parameters.
- Explicit serialization behavior for stable code values.
- Safe handling of unknown or unsupported codes.
- A localized generic fallback for unknown user-facing messages.
- Technical diagnostic details kept separate from localized presentation text.

Where practical, message parameters should have typed Rust representations rather than relying on unvalidated free-form strings. The frontend should not infer message meaning from arbitrary English prose.

This contract is intended for application-generated guidance, action previews, remediation reasons, confirmation messages, success messages, and verification messages. Arbitrary operating-system or library errors must not be blindly translated. Preserve useful technical diagnostics separately.

Stable action IDs, enum values, verification states, process identifiers, PIDs, paths, interface names, and other machine-facing values must not be translated.

### Historical audit integrity

Existing persisted audit entries include human-readable messages. Historical records must remain unchanged when the active locale changes. Localize known labels and status descriptions around those records, but do not rewrite previously recorded message text or alter existing persisted formats as part of localization.

## 7. Locale detection, normalization, and persistence

### 7.1 Selection precedence

The approved locale precedence is:

1. Saved Linux Powerhouse user preference.
2. Linux `LC_ALL`.
3. Linux `LC_MESSAGES`.
4. Linux `LANG`.
5. English (`en`) fallback.

An explicit supported user preference takes precedence over the system locale. An invalid, malformed, or unsupported saved preference is ignored so detection can continue through the system locale sources.

### 7.2 Controlled system locale retrieval

The React frontend must not assume direct access to Linux process environment variables. System locale values should be obtained through a controlled Rust/Tauri interface and passed to the frontend through a defined application boundary.

The implementation must handle missing and invalid environment values safely and must not modify the operating system's locale settings.

### 7.3 Normalization

Normalize POSIX locale forms into BCP 47 identifiers. Examples:

| Linux locale | Normalized locale |
| --- | --- |
| `en_US.UTF-8` | `en-US` |
| `en_GB.UTF-8` | `en-GB` |
| `kn_IN.UTF-8` | `kn-IN` |

Handle encoding suffixes, separators, malformed identifiers, and the special `C` and `POSIX` locales. A syntactically valid but unsupported language must fall back to English unless a supported language resource can be selected.

### 7.4 Preference persistence

Use the existing approved key:

`linux-powerhouse.locale.v1`

The selected locale value must be validated before use. Missing, malformed, or unsupported stored values must not prevent application startup.

If preference persistence fails, the current session should continue using its active locale. Runtime language switching must update the interface without requiring an application restart and must persist the preference when storage is available.

## 8. Formatting architecture

Use centralized formatting functions backed by JavaScript `Intl` APIs. Feature components should not independently implement locale-sensitive formatting.

Proposed public formatting operations:

- `formatDate()`
- `formatDateTime()`
- `formatTime()`
- `formatRelativeTime()`
- `formatNumber()`
- `formatPercent()`
- `formatBytes()`
- `formatDuration()`
- `formatCurrency()`

Logical responsibilities may be separated into date/time, numbers, measurements, and durations modules, or consolidated if the implementation remains clear and testable.

### 8.1 Formatting conventions

| Data type | Convention |
| --- | --- |
| Dates and times | `Intl.DateTimeFormat` |
| Relative time | `Intl.RelativeTimeFormat` |
| Numbers | `Intl.NumberFormat` |
| Percentages | `Intl.NumberFormat` with percent style |
| Byte sizes | IEC units |
| Durations | Centralized localized duration formatter |
| Currency | `Intl.NumberFormat` with an explicit currency code |

Use IEC byte units: `B`, `KiB`, `MiB`, `GiB`, and `TiB`, with binary scaling (base 1024). Number portions should follow the active locale while the unit symbols remain technically correct.

Percentage formatter input uses percentage-point semantics: the numeric input `37.5` represents `37.5%`. Any ratio-based internal value must be converted explicitly at the presentation boundary.

Currency formatting must require an explicit currency code, such as `INR`; do not infer currency solely from the selected language.

The default timezone is the operating system's local timezone. Store and exchange timestamps in a consistent machine-readable representation and convert only for display. Audit timestamps and underlying numeric values must remain unchanged.

Precision should be defined centrally by measurement type. Initial guidance is up to one decimal place for CPU and memory percentages, up to two decimal places for byte-size displays, and integer formatting for counts. Feature-specific requirements may justify different precision, but the same measurement must be displayed consistently across the application.

Formatting must never change calculations, stored values, or comparisons.

## 9. Localization coverage

The initial frontend migration covers:

1. Application shell and navigation.
2. Dashboard.
3. System Health.
4. Health Action Workspace.
5. Alert Controls.
6. Alert History.
7. Alert Insights and Summary.
8. Performance.
9. Action Audit.
10. Shared controls, accessibility labels, and formatting.

Backend message localization covers known application-generated guidance, action previews, remediation, confirmations, execution results, and verification messages.

Machine identifiers, technical values, process names, paths, and persisted state values remain language-neutral.

## 10. Incremental migration sequence

Localization must be introduced in independently reviewable phases. English must remain usable throughout.

### Phase 0 — Baseline and safeguards

- Inventory visible strings and backend-generated prose.
- Identify stable identifiers, persisted values, and historical messages.
- Establish localization regression coverage.
- Preserve unrelated working-tree changes and existing project conventions.

### Phase 1 — Localization foundation

- Add and configure i18next and react-i18next when implementation is authorized.
- Establish resource and namespace structure.
- Implement locale normalization and validation.
- Implement controlled Rust/Tauri system locale retrieval.
- Implement saved preference handling.
- Initialize localization before rendering the application.

### Phase 2 — Shared resources and application shell

- Establish English and Kannada resource structures.
- Localize common controls and accessibility labels.
- Localize navigation and application shell.
- Add language selection.
- Verify runtime switching and persistence.

### Phase 3 — Frontend feature migration

Migrate dashboard, health, workspace, alerts, history, insights, performance, and audit features incrementally. Each feature must have targeted translation and functional regression tests before proceeding.

### Phase 4 — Backend message contract

- Introduce typed Rust message enums and stable serialization.
- Add structured, message-specific parameters.
- Migrate known application-generated user-facing messages.
- Preserve technical diagnostics and historical audit records.

### Phase 5 — Centralized formatting migration

- Implement shared formatting operations.
- Migrate existing date and time formatting.
- Migrate numbers, percentages, byte sizes, durations, and applicable currency displays.
- Verify calculations and stored values remain unchanged.

### Phase 6 — Kannada coverage and quality assurance

- Complete Kannada translations after English keys and resource structure are stable.
- Review technical terminology.
- Validate fonts, rendering, text expansion, and layout.
- Test accessibility, fallback behavior, and feature regressions.
- Run full relevant Rust and Desktop validation.

## 11. Testing and acceptance criteria

### 11.1 Localization foundation

Test initialization, resource availability, namespace loading, runtime switching, missing keys, resource failures, and English fallback.

### 11.2 Locale detection

Test saved-preference precedence, `LC_ALL` / `LC_MESSAGES` / `LANG` precedence, malformed values, empty values, whitespace-only values, encoding suffixes, regional identifiers, and `C` / `POSIX` behavior.

### 11.3 Translation integrity

Test missing and duplicate keys, invalid JSON, missing interpolation parameters, pluralization, Kannada plural behavior, namespace failures, and unknown backend message codes.

### 11.4 Backend contract

Test typed enum serialization, stable message codes, parameter serialization, frontend resolution, unknown-code fallback, preservation of diagnostics, and immutability of historical audit messages.

### 11.5 Formatting

Test locale-sensitive date ordering, timezone conversion, relative time, number grouping and decimals, percentage boundaries, IEC byte-unit boundaries, duration boundaries, explicit currency codes, invalid inputs, and large values.

### 11.6 Feature regression

Verify that alert decisions, safe-action confirmation, authorization, diagnostic behavior, performance calculations, filtering, sorting, search, grouping, and audit integrity remain unchanged.

### 11.7 Kannada and accessibility

Test Kannada character rendering, font fallback, text wrapping, clipping, expanded controls, dialogs, notifications, keyboard navigation, screen-reader labels, mixed technical terminology, and RTL-readiness of layout decisions.

### 11.8 Reliability and performance

Verify that localization does not introduce avoidable startup delays, repeated initialization, unhandled resource errors, or application instability during language switching.

### 11.9 CI

Use the repository's existing Rust, Desktop, frontend, and preflight checks as applicable. Candidate Rust checks include formatting, workspace checking, Clippy with warnings denied, and workspace tests. Final commands must be reconciled with the actual CI configuration before implementation; do not add redundant CI jobs without need.

## 12. Migration safeguards

- Do not translate internal identifiers or enum values.
- Do not rename existing storage keys or change persisted alert formats as part of localization.
- Do not rewrite historical audit messages.
- Do not alter safe-action eligibility, confirmation, authorization, execution, verification, or audit behavior.
- Do not change calculations to accommodate display formatting.
- Do not blindly translate arbitrary technical errors.
- Keep English functional throughout migration.
- Keep frontend and backend work independently reviewable.
- Do not combine unrelated roadmap work with localization.
- Preserve unrelated local changes, including untracked files, during repository work.

## 13. Git and review workflow

Follow the existing Linux Powerhouse workflow:

1. Use dedicated feature branches for focused documentation or implementation work.
2. Keep changes small and independently reviewable.
3. Run targeted tests after meaningful implementation changes.
4. Run applicable Rust formatting, checking, Clippy, desktop, frontend, and preflight validation.
5. Open focused pull requests.
6. Review and merge only after the applicable checks are green.
7. Confirm post-merge CI before beginning dependent work.

Documentation-only changes should not be presented as implementation completion and should not trigger unnecessary code or dependency changes.

## 14. Implementation milestones

These are planning milestones, not numbered roadmap steps. Step numbers will be assigned only when the project explicitly schedules the implementation sequence.

| Milestone | Scope | Completion condition |
| --- | --- | --- |
| M1 — Localization Foundation | Dependencies, initialization, locale normalization, system locale interface, preference persistence | Locale selection and fallback work reliably |
| M2 — Shared Resources and Shell | Common resources, application shell, language selector | English/Kannada switching works without restart |
| M3 — Frontend Feature Migration | All listed React feature namespaces | Planned frontend coverage and feature regressions pass |
| M4 — Backend Message Contract | Typed Rust message codes, parameters, frontend resolution | Known messages localize and audit history remains immutable |
| M5 — Centralized Formatting | Dates, times, numbers, percentages, bytes, durations, currency | Shared formatters adopted without changing underlying values |
| M6 — Kannada and Final QA | Translation review, accessibility, layout, full regression | Kannada quality and project acceptance criteria pass |

The milestones may be split into multiple small pull requests. Their exact step numbers, dependencies, and sequencing details must be established in the implementation breakdown.

## 15. Definition of done

Localization is complete only when:

- English and Kannada are operational.
- Locale detection, normalization, and persistence work correctly.
- Runtime language switching works without restart.
- All planned frontend features are localized.
- The backend message contract is operational.
- Centralized formatting is adopted.
- Historical audit integrity is preserved.
- Accessibility and Kannada rendering are reviewed.
- Localization-related functional regressions are absent.
- Relevant Rust and Desktop CI are green.
- Documentation reflects the implemented state.
- Final localization review is approved.

## 16. Approval and implementation status

| Planning stage | Status |
| --- | --- |
| L1 — Localization Architecture | Approved |
| L2.1 — UI Translation Inventory | Approved |
| L2.2 — Translation Key Architecture | Approved |
| L2.3 — Locale Detection and Persistence | Approved |
| L2.4 — Formatting Architecture | Approved |
| L2.5 — Migration Sequence | Approved |
| L2.6 — Testing and Acceptance Criteria | Approved |

**Localization architecture and planning are complete. Implementation has not started.**

This specification records approved architectural decisions and the proposed implementation sequence. It does not itself authorize source-code modifications, dependency installation, or implementation work. Those changes require separately reviewed and approved roadmap tasks.
