<!--
Sync Impact Report
==================
Version Change: 0.0.0 → 1.0.0
Modified Principles: N/A (initial constitution)
Added Sections: All core principles and governance
Removed Sections: None
Templates Status:
  ✅ plan-template.md - Constitution Check section aligns with principles
  ✅ spec-template.md - Requirements structure supports principle-driven development
  ✅ tasks-template.md - Task organization reflects testing and modularity principles
  ✅ checklist-template.md - Generic structure compatible with all principles
Follow-up TODOs: None
-->

# realtime-svg Constitution

## Core Principles

### I. Workspace Modularity

The project MUST maintain a clear multi-crate workspace structure with explicit
boundaries between `common` (domain logic), `backend` (HTTP/streaming server),
and `frontend` (WASM UI). Each crate MUST be independently testable with minimal
cross-dependencies. Shared types and business logic MUST reside in `common`.

**Rationale**: Rust workspace architecture enforces compile-time boundaries,
prevents circular dependencies, and enables parallel development of concerns.

### II. Contract-First API Design

All HTTP endpoints and streaming protocols MUST be documented with explicit
contracts before implementation. API contracts MUST specify request/response
schemas, error codes, content types, and boundary conditions. The
`multipart/x-mixed-replace` streaming protocol MUST maintain backward
compatibility once established.

**Rationale**: Real-time streaming systems require precise protocol guarantees
to prevent client breaking changes and ensure interoperability.

### III. Type-Safe Domain Models

Domain entities (`SvgTemplate`, `UpdateRequest`, `SvgFrame`) MUST be defined as
strongly-typed structs in `common` with `serde` serialization. Runtime string
manipulation MUST be minimized in favor of structured parsing. XML operations
MUST use validated libraries (`xmltree`, `roxmltree`) rather than regex or
ad-hoc string replacements.

**Rationale**: Rust's type system prevents entire classes of runtime errors;
leveraging it fully reduces debugging overhead and increases reliability.

### IV. Testing Discipline

- **Unit tests**: All `common` crate logic MUST have unit tests covering
  success paths, edge cases, and error conditions.
- **Integration tests**: Backend routes (`POST /api/update`, `GET /stream.svg`)
  MUST have integration tests validating end-to-end request/response cycles.
- **Contract tests**: Streaming boundary format and header compliance MUST be
  verified via contract tests.

Tests MUST be written before or alongside implementation (TDD encouraged).

**Rationale**: Real-time systems have complex failure modes; comprehensive
testing catches issues before production deployment.

### V. Observability & Debugging

All crates MUST use structured logging via `tracing` with appropriate log
levels. Backend MUST log:
- Incoming API requests with sanitized payloads
- SVG rendering operations and timing
- Stream client connections and disconnections
- Error conditions with context

Frontend MUST provide user-visible error messages for API failures.

**Rationale**: Distributed real-time systems require rich telemetry to diagnose
latency issues, connection problems, and rendering failures.

### VI. Simplicity & Incremental Delivery

Features MUST start with the simplest viable implementation. Avoid premature
optimization, complex abstractions, or speculative features. Deliver
functionality in small, testable increments prioritized by user value.

**Rationale**: Rust's compiler enforces safety; focus developer effort on
delivering working features rather than over-engineering.

## Technology Standards

### Language & Tooling

- **Rust Edition**: 2021 or later
- **Workspace Structure**: Cargo workspace with explicit crate boundaries
- **Build Tool**: `cargo` for backend/common, `trunk` for frontend WASM builds
- **Linting**: `clippy` with default lints; warnings MUST not be suppressed
  without documented justification
- **Formatting**: `rustfmt` with default settings; all code MUST be formatted
  before commit

### Dependencies

- Backend: `axum` (HTTP), `tokio` (async runtime), `tokio-stream` (streaming),
  `tower-http` (middleware)
- Common: `serde`/`serde_json` (serialization), `thiserror` (error handling),
  `chrono` (timestamps), `xmltree` or `roxmltree` (XML parsing)
- Frontend: `yew` (WASM framework), browser-native fetch APIs

New dependencies MUST be justified by necessity and evaluated for maintenance
status, security, and bundle size impact.

## Development Workflow

### Feature Development

1. **Specification**: Create feature spec in `specs/###-feature-name/spec.md`
   with user scenarios and acceptance criteria.
2. **Planning**: Generate implementation plan with `/speckit.plan` command,
   including technical context and constitution compliance check.
3. **Task Breakdown**: Generate task list with `/speckit.tasks` organized by
   user story priority.
4. **Implementation**: Execute tasks incrementally, writing tests alongside code.
5. **Validation**: Verify all acceptance criteria, run integration tests, check
   streaming protocol compliance.

### Quality Gates

Before merging:
- [ ] All tests pass (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] Frontend builds successfully (`trunk build`)
- [ ] Integration tests validate API contracts
- [ ] Streaming protocol headers and boundaries verified
- [ ] Observability logs reviewed for completeness

## Governance

### Amendment Process

1. Propose changes via pull request with rationale and impact analysis.
2. Identify affected templates (plan, spec, tasks, checklist) and update them.
3. Increment constitution version according to semantic versioning:
   - **MAJOR**: Principle removal or backward-incompatible governance changes
   - **MINOR**: New principle additions or material expansions
   - **PATCH**: Clarifications, wording improvements, typo fixes
4. Document changes in Sync Impact Report (HTML comment at file top).
5. Update `LAST_AMENDED_DATE` to amendment date.

### Compliance Review

All pull requests MUST verify adherence to:
- Workspace modularity (no invalid cross-crate dependencies)
- Type safety (no unsafe code without justification)
- Testing discipline (tests for new functionality)
- Observability (structured logging present)

Constitution violations MUST be documented in plan.md Complexity Tracking table
with justification and rejected simpler alternatives.

### Runtime Guidance

See `AGENTS.md` for agent-specific development guidance and Korean language
design notes.

**Version**: 1.0.0 | **Ratified**: 2025-10-17 | **Last Amended**: 2025-10-17
