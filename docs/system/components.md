# Supercode Components

## Core Components

### 1. supercode-core

**Purpose:** Business logic and orchestration.

**Responsibilities:**
- Session lifecycle management
- Project management
- Agent dispatch and coordination
- State management

**Location:** `src/core/`

### 2. session-manager

**Purpose:** Handles individual agent sessions.

**Responsibilities:**
- Spawn new sessions
- Stream I/O
- Handle process lifecycle
- Monitor health

**Location:** `src/session/`

**Sub-modules:**
- `opencode_session.rs` - OpenCode HTTP API integration
- `claude_session.rs` - Claude Code CLI wrapper
- `session.rs` - Common session interface

### 3. mcp-server

**Purpose:** MCP protocol implementation.

**Responsibilities:**
- JSON-RPC 2.0 handling
- Tool definitions
- Request/response serialization
- Authentication

**Location:** `src/mcp/`

### 4. cli-wrappers

**Purpose:** Subprocess management for Claude Code.

**Responsibilities:**
- CLI process spawning
- Output streaming
- Exit code handling
- Signal management

**Location:** `src/wrappers/`

### 5. persistence

**Purpose:** SQLite database operations.

**Responsibilities:**
- Schema management
- CRUD operations
- Migrations

**Location:** `src/db/`

---

## Agent Types

### Manager Agent

**File:** `.opencode/agent/manager.md`

**Role:** Primary orchestrator
- Coordinates development work
- Delegates to specialist agents
- Maintains high-level context
- Ensures quality gates

**Mode:** Primary

### Developer Agent

**File:** `.opencode/agent/developer.md`

**Role:** Implementation specialist
- Implements features
- Fixes bugs
- Writes tests
- Runs linters/typecheckers

**Mode:** Subagent

### Reviewer Agent

**File:** `.opencode/agent/reviewer.md`

**Role:** Quality assurance
- Code reviews
- Architecture reviews
- Security reviews
- Performance reviews

**Mode:** Subagent

---

## Database Schema

### sessions
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    agent_type TEXT NOT NULL, -- 'manager', 'developer', 'reviewer'
    session_type TEXT NOT NULL, -- 'opencode', 'claude'
    status TEXT NOT NULL, -- 'running', 'completed', 'failed', 'terminated'
    working_dir TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    metadata TEXT -- JSON
);
```

### projects
```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    metadata TEXT -- JSON
);
```

### messages
```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL, -- 'user', 'assistant'
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);
```

### agent_configs
```sql
CREATE TABLE agent_configs (
    id TEXT PRIMARY KEY,
    agent_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    model TEXT NOT NULL,
    system_prompt TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```
