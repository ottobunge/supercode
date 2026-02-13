# Alternatives Considered

## Option 1: n8n/Zapier Automation

**Description:** Use workflow automation tools to coordinate agents via APIs.

**Pros:**
- No custom development needed
- Visual workflow editor
- Built-in integrations

**Cons:**
- Limited control over agent behavior
- Latency from external API calls
- Not designed for coding agent orchestration
- Cost for hosted version

**Verdict:** ❌ Rejected - not suitable for coding agent coordination

---

## Option 2: tmux-based (agent-of-empires)

**Description:** Use tmux sessions with shell wrappers for each agent.

**Reference:** https://github.com/njbrake/agent-of-empires

**Pros:**
- Simple shell-based approach
- Works with any CLI agent
- Lightweight

**Cons:**
- No native MCP support
- Limited session state management
- tmux dependency
- Manual coordination

**Verdict:** ❌ Rejected - too basic, lacks MCP integration

---

## Option 3: Direct Subprocess Management

**Description:** Python/Node scripts that spawn agent CLIs as subprocesses.

**Pros:**
- Full control over process lifecycle
- Can wrap any CLI tool
- Flexible

**Cons:**
- Reinventing session management
- No MCP server built-in
- Error handling complexity
- No Rust performance benefits

**Verdict:** ❌ Rejected - better to build proper system in Rust

---

## Option 4: Custom Rust Orchestrator (Chosen)

**Description:** Build a custom Rust application that:
- Manages OpenCode sessions via HTTP API
- Manages Claude Code sessions via CLI wrapping
- Exposes MCP tools for external control
- Uses SQLite for persistence

**Pros:**
- Full control over behavior
- Native Rust performance
- Built-in MCP server
- SQLite integration
- Can create custom agent prompts

**Cons:**
- Initial development effort
- Need to maintain

**Verdict:** ✅ Selected - best fit for requirements

---

## Hybrid Approach

We will use:
- **OpenCode** as primary agent harness (already familiar with)
- **Claude Code** via CLI wrapping (licensed)
- **Custom Rust MCP server** for orchestration
- **SQLite** for state persistence
