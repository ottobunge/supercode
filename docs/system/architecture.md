# Supercode Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        MCP Client                                │
│  (OpenCode, Claude Code, or external tool)                       │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MCP Server                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Tool Layer                               ││
│  │  - spawn_session    - list_sessions    - send_message     ││
│  │  - kill_session     - get_session      - fork_session      ││
│  └─────────────────────────────────────────────────────────────┘│
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Orchestrator Core                            │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────────┐  │
│  │ Session Mgmt  │  │ Project Mgmt  │  │ Agent Dispatch   │  │
│  └───────┬────────┘  └───────┬────────┘  └────────┬────────┘  │
└──────────┼───────────────────┼────────────────────┼────────────┘
           │                   │                    │
           ▼                   ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Session Manager                              │
│  ┌─────────────────────┐    ┌─────────────────────┐             │
│  │  OpenCode Sessions  │    │ Claude Code Sessions│             │
│  │  (via HTTP API)     │    │  (via CLI Wrapper) │             │
│  └─────────────────────┘    └─────────────────────┘             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SQLite DB                                  │
│  - sessions     - projects    - messages   - agent_config      │
└─────────────────────────────────────────────────────────────────┘
```

## Component Layers

### 1. MCP Server Layer
- Exposes tools to external clients
- JSON-RPC 2.0 protocol
- Authentication (future)
- Request validation

### 2. Orchestrator Core
- **Session Management**: Create, list, kill, fork sessions
- **Project Management**: Create projects, assign sessions
- **Agent Dispatch**: Route tasks to appropriate agent types

### 3. Session Manager
- Manages OpenCode sessions via HTTP API
- Manages Claude Code sessions via CLI
- Handles subprocess lifecycle
- Streams output back to orchestrator

### 4. Persistence Layer
- SQLite for all state
- Sessions, projects, messages
- Agent configurations

## Data Flow

1. **Spawn Session**
   ```
   MCP Request → MCP Server → Orchestrator → Session Manager
                 → OpenCode/Claude Code CLI → Session Created
                 → SQLite (persist) → Return session ID
   ```

2. **Send Message**
   ```
   MCP Request → MCP Server → Orchestrator → Find Session
                 → Send to OpenCode API or Claude CLI
                 → Stream response → Return to client
   ```

3. **Delegate Task**
   ```
   Manager Agent → Orchestrator → Spawn Developer Agent Session
                 → Developer works → Report back
                 → Spawn Reviewer Agent Session
                 → Reviewer reviews → Report back
                 → Manager aggregates
   ```

## Agent Interaction Patterns

### Manager → Developer → Reviewer
```
Manager: "Implement feature X"
  │
  ├──▶ Developer: "Here's the implementation..."
  │      │
  │      └──▶ [runs tools, writes code, tests]
  │           │
  │           └──▶ "Implementation complete"
  │
  └──▶ Reviewer: "Code review findings..."
       │
       └──▶ [reviews code]
            │
            └──▶ "LGTM" or "Changes needed"
```
