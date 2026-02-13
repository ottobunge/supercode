# Supercode Goals

## Primary Objective

Build an orchestration system that manages 10+ parallel coding agent sessions for developing large projects and managing multiple projects simultaneously.

## Core Goals

1. **Parallel Session Management**
   - Spawn and manage 10+ concurrent coding sessions
   - Each session operates independently on different tasks
   - Centralized coordination and context sharing

2. **Multi-Project Support**
   - Handle multiple projects simultaneously
   - Maintain separate contexts per project
   - Enable project-to-project task delegation

3. **Dual Agent Orchestration**
   - Manage OpenCode sessions via HTTP API
   - Manage Claude Code sessions via CLI wrapping
   - Unified interface for both agent types

4. **MCP Server Integration**
   - Expose session management as MCP tools
   - Enable external tools to interact with sessions
   - Support for programmatic orchestration

5. **Quality Assurance**
   - Built-in review agents for code quality
   - Architecture enforcement
   - Linting and testing coordination

## Success Criteria

- [ ] Can spawn 10+ parallel sessions
- [ ] Sessions can work on different projects
- [ ] Manager agent can delegate to developer/reviewer agents
- [ ] Claude Code integration works (licensed)
- [ ] MCP tools available for external control
- [ ] SQLite persistence for session state

## Use Cases

1. **Large Project Development**
   - Multiple agents working on different features
   - Parallel implementation and code review
   - Coordinated through manager agent

2. **Multiple Projects**
   - Agents assigned to different projects
   - Context switching between projects
   - Resource optimization

3. **Redundancy/Failover**
   - Same task assigned to multiple agents
   - Best result selected
   - Fault tolerance
