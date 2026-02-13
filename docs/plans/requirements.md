# Supercode Requirements

## Functional Requirements

### Session Management
- FR-001: Spawn new coding sessions (OpenCode and Claude Code)
- FR-002: List all active sessions with status
- FR-003: Kill/terminate sessions
- FR-004: Resume existing sessions
- FR-005: Fork sessions for parallel work

### Agent Types
- FR-010: Manager agent - orchestrates work, delegates tasks
- FR-011: Developer agent - implements features, runs tests
- FR-012: Reviewer agent - code and architecture reviews

### Project Management
- FR-020: Create projects with metadata
- FR-021: Assign sessions to projects
- FR-022: Track project progress
- FR-023: Context isolation per project

### MCP Integration
- FR-030: Expose session management as MCP tools
- FR-031: Tool for spawning sessions
- FR-032: Tool for listing/querying sessions
- FR-033: Tool for sending messages to sessions

### Persistence
- FR-040: Store session state in SQLite
- FR-041: Persist project metadata
- FR-042: Restore sessions on restart

## Non-Functional Requirements

### Technology
- NFR-001: Built in Rust
- NFR-002: SQLite for persistence (simple, no external DB)
- NFR-003: HTTP API for OpenCode integration
- NFR-004: CLI wrapping for Claude Code

### Performance
- NFR-010: Support 10+ concurrent sessions
- NFR-011: Session spawn time < 5 seconds
- NFR-012: Message latency < 100ms

### Reliability
- NFR-020: Graceful session termination
- NFR-021: Error handling and logging
- NFR-022: State recovery on crash

## Technical Constraints

- TC-001: Claude Code requires commercial license for automation
- TC-002: OpenCode HTTP API runs on localhost
- TC-003: MCP server needs proper tool definitions
