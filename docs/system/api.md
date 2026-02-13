# Supercode API

## MCP Tools

### spawn_session

Create a new agent session.

**Parameters:**
```json
{
  "agent_type": "manager|developer|reviewer",
  "session_type": "opencode|claude",
  "project_id": "string (optional)",
  "working_dir": "string (optional)",
  "initial_message": "string (optional)"
}
```

**Returns:**
```json
{
  "session_id": "string",
  "status": "running"
}
```

---

### list_sessions

List all sessions, optionally filtered.

**Parameters:**
```json
{
  "project_id": "string (optional)",
  "status": "string (optional)",
  "agent_type": "string (optional)"
}
```

**Returns:**
```json
{
  "sessions": [
    {
      "id": "string",
      "agent_type": "string",
      "session_type": "string",
      "status": "string",
      "project_id": "string",
      "created_at": "string"
    }
  ]
}
```

---

### send_message

Send a message to a session.

**Parameters:**
```json
{
  "session_id": "string",
  "content": "string"
}
```

**Returns:**
```json
{
  "response": "string",
  "session_status": "running|completed"
}
```

---

### kill_session

Terminate a running session.

**Parameters:**
```json
{
  "session_id": "string"
}
```

**Returns:**
```json
{
  "success": true,
  "session_id": "string"
}
```

---

### get_session

Get session details and history.

**Parameters:**
```json
{
  "session_id": "string"
}
```

**Returns:**
```json
{
  "id": "string",
  "agent_type": "string",
  "status": "string",
  "messages": [...],
  "created_at": "string",
  "updated_at": "string"
}
```

---

### fork_session

Fork an existing session for parallel work.

**Parameters:**
```json
{
  "session_id": "string",
  "fork_message": "string (optional)"
}
```

**Returns:**
```json
{
  "session_id": "string",
  "forked_from": "string",
  "status": "running"
}
```

---

## Internal REST API

### POST /sessions
Create a new session.

### GET /sessions
List sessions.

### GET /sessions/:id
Get session details.

### DELETE /sessions/:id
Kill session.

### POST /sessions/:id/message
Send message to session.

### POST /sessions/:id/fork
Fork session.

### GET /projects
List projects.

### POST /projects
Create project.

### GET /projects/:id
Get project details.

### GET /projects/:id/sessions
Get sessions for project.
