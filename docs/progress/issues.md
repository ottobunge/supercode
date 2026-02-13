# Open Issues

## Technical Questions

- [ ] **ISSUE-001:** How to handle OpenCode API authentication? Currently no auth, is this sufficient for local use?
- [ ] **ISSUE-002:** Session output streaming - OpenCode API vs Claude Code CLI streaming different approaches
- [ ] **ISSUE-003:** Claude Code license verification - how to confirm valid commercial license programmatically?

## Design Decisions Needed

- [x] **DECISION-001:** MCP server runs standalone (TCP socket) ✅
- [ ] **DECISION-002:** How to handle agent handoffs? (e.g., Manager → Developer → Reviewer)
- [x] **DECISION-003:** Full stdout/stderr captured ✅
- [ ] **DECISION-004:** How to handle session timeouts?

## Critical Issues (From Reviews)

### Architecture
- [ ] **ISSUE-010:** Synchronous SQLite (rusqlite) blocks async runtime - need spawn_blocking or sqlx
- [ ] **ISSUE-011:** r2d2 connection pooling unused - implement or remove
- [ ] **ISSUE-012:** No kill_session at provider level - sessions leak

### Code Quality
- [ ] **ISSUE-013:** Command injection vulnerability in gates.rs - project_dir not validated
- [ ] **ISSUE-014:** No connection limits in MCP server - DoS vulnerability
- [ ] **ISSUE-015:** Process leak in Claude client - orphaned processes on error

### Implementation
- [ ] **ISSUE-016:** REST API not implemented (only MCP available)
- [ ] **ISSUE-017:** Zero test coverage
- [ ] **ISSUE-018:** Claude response handling is placeholder text only
- [ ] **ISSUE-019:** Claude fork creates new session instead of true fork

## High Priority Issues

- [ ] **ISSUE-020:** Input validation uses silent defaults in MCP server
- [ ] **ISSUE-021:** Database unwraps will panic on corrupt data
- [ ] **ISSUE-022:** Single-threaded CLI runtime - poor performance
- [ ] **ISSUE-023:** Blocking calls in QualityGates block async runtime
- [ ] **ISSUE-024:** HTTP client has no timeouts
- [ ] **ISSUE-025:** Quality gates not integrated (dead code)
- [ ] **ISSUE-026:** No message persistence (messages table unused)

## Medium Priority Issues

- [ ] **ISSUE-027:** Hardcoded provider URLs (no config)
- [ ] **ISSUE-028:** No rate limiting on MCP server
- [ ] **ISSUE-029:** Project progress tracking not implemented
- [ ] **ISSUE-030:** Context isolation per project not enforced

## Blockers

- [ ] **BLOCKER-001:** None - research phase complete
- [ ] **BLOCKER-002:** DB layer blocking - blocks scalability to 10+ sessions

## Known Limitations

- [x] **LIMIT-001:** Claude Code automation requires commercial license ✅ (user confirmed)
- [x] **LIMIT-002:** OpenCode HTTP API is localhost only by default ✅
- [x] **LIMIT-003:** SQLite not suitable for distributed systems ✅ (not applicable)

---

## Review Documents

See `docs/progress/reviews/` for detailed reviews:
- `architecture-review.md` - System design analysis
- `code-quality-review.md` - Code-level issues
- `implementation-gaps-review.md` - Feature gaps
- `SUMMARY.md` - Consolidated findings
