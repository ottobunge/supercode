# Research Notes

## OpenCode Capabilities

### HTTP Server
- `opencode serve` starts HTTP server on localhost
- API endpoints: `/session`, `/session/:id/message`, `/session/:id/children`, `/session/:id/fork`
- Session management via REST API
- Supports session forking

### Subagent System
- Built-in Task tool for launching subagents
- `task_id` parameter for session resumption
- PR #7756 adds subagent-to-subagent delegation with configurable budgets
- Supports `task_budget` and depth limits

### References
- https://opencode.ai/docs/cli/
- https://opencode.ai/docs/server/
- https://opencode.ai/docs/agents/

---

## Claude Code CLI

### Non-Interactive Mode
- `-p` flag for print mode (non-interactive)
- `--resume` to continue session
- `--session-id` for specific session
- `--fork-session` to fork

### MCP Server
- `claude mcp serve` runs as MCP server
- Agent teams feature for multi-agent coordination

### References
- https://code.claude.com/docs/en/cli-reference
- https://code.claude.com/docs/en/headless
- https://code.claude.com/docs/en/mcp

---

## Legal / Licensing

### Consumer Terms
- Restrictions on "automated" use except where explicitly permitted
- Third-party integrations allowed but automation limited

### Commercial Terms
- API/Bedrock/Vertex explicitly allow programmatic use
- Claude Code commercial license permits automation

### Status
- ✅ User has confirmed commercial license for Claude Code
- ✅ Legal approval obtained

### References
- https://www.anthropic.com/legal/commercial-terms
- https://www.anthropic.com/legal/consumer-terms

---

## Existing Projects

### agent-of-empires
- https://github.com/njbrake/agent-of-empires
- Terminal session manager for AI coding agents via tmux
- 632 stars
- Approach: tmux-based, shell wrappers

### opencode-mcp
- https://github.com/nosolosoft/opencode-mcp
- Community MCP server for OpenCode
- Alternative to building custom

### claude-mcp
- https://github.com/ebeloded/claude-mcp
- Claude Code MCP integration
- Session continuity with async execution

---

## Key Technical Findings

1. **OpenCode HTTP API** is sufficient for session management
2. **Claude Code CLI** supports non-interactive mode but needs proper wrapping
3. **Both support MCP** - can use as clients or be wrapped
4. **SQLite** is appropriate for 10+ session scale
5. **Rust** has good libraries for all requirements (tokio, sqlx, reqwest)
