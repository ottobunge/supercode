# Supercode Roadmap

## Phase 1: Core Infrastructure
**Goal:** Basic Rust project with SQLite and session management.

- [x] 1.1 Initialize Rust project with Cargo
- [x] 1.2 Set up SQLite with rusqlite/sqlx
- [x] 1.3 Create database schema
- [x] 1.4 Implement basic Session struct and CRUD
- [x] 1.5 Add logging with tracing

**Deliverable:** CLI that can create/list/kill basic sessions in SQLite ✅

---

## Phase 2: OpenCode Integration
**Goal:** Manage OpenCode sessions via HTTP API.

- [x] 2.1 Add HTTP client (reqwest)
- [x] 2.2 Implement OpenCode session spawn
- [x] 2.3 Implement message sending via API
- [x] 2.4 Handle session output streaming
- [x] 2.5 Add session health monitoring

**Deliverable:** Can spawn OpenCode sessions and send messages ✅

---

## Phase 3: Claude Code Integration
**Goal:** Manage Claude Code sessions via CLI wrapper.

- [x] 3.1 Implement CLI wrapper for Claude Code
- [x] 3.2 Handle non-interactive mode (-p flag)
- [x] 3.3 Implement session resumption (--resume)
- [x] 3.4 Handle JSON output parsing
- [x] 3.5 Add process lifecycle management

**Deliverable:** Can spawn Claude Code sessions and send messages ✅

---

## Phase 4: MCP Server
**Goal:** Expose session management as MCP tools.

- [x] 4.1 Implement MCP server foundation
- [x] 4.2 Define tool schemas (spawn, list, send, kill, fork)
- [x] 4.3 Implement JSON-RPC handlers
- [x] 4.4 Add request validation
- [x] 4.5 Test with OpenCode MCP client

**Deliverable:** MCP server running with working tools ✅

---

## Phase 5: Agent System
**Goal:** Implement manager/developer/reviewer agents.

- [x] 5.1 Create agent prompt templates
- [x] 5.2 Implement manager orchestration logic
- [x] 5.3 Implement developer agent workflow
- [x] 5.4 Implement reviewer agent workflow
- [x] 5.5 Add quality gates (lint, test, typecheck)

**Deliverable:** Agent system that can coordinate work ✅

---

## Phase 6: Polish & Release
**Goal:** Production-ready release.

- [ ] 6.1 Error handling and recovery
- [ ] 6.2 Configuration management
- [ ] 6.3 Documentation
- [ ] 6.4 Testing (unit + integration)
- [ ] 6.5 Performance optimization

**Deliverable:** v1.0 release

---

## Milestone Checklist

| Milestone | Status | Date |
|-----------|--------|------|
| Phase 1: Core | ✅ | 2026-02-13 |
| Phase 2: OpenCode | ✅ | 2026-02-13 |
| Phase 3: Claude Code | ✅ | 2026-02-13 |
| Phase 4: MCP Server | ✅ | 2026-02-13 |
| Phase 5: Agents | ✅ | 2026-02-13 |
| Phase 6: Release | ⬜ | - |
