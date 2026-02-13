# Supercode Code Quality Review

**Overall Assessment:** Changes Requested - Multiple critical and high-severity issues

---

## Critical Issues 🔴

### 1. Command Injection Vulnerability
**File:** `src/agent/gates.rs:54-108`

```rust
let output = Command::new("cargo")
    .args(["check", "--manifest-path", &format!("{}/Cargo.toml", project_dir)])
```

The `project_dir` parameter is directly interpolated without validation. Path traversal possible.

**Fix:** Validate and sanitize the project directory path.

---

### 2. No Connection Limits (DoS Vulnerability)
**File:** `src/mcp/server.rs:28-38`

The TCP server accepts unlimited concurrent connections without timeouts or rate limiting.

---

### 3. Process Leak on Error
**File:** `src/session/claude/client.rs:76-95`

If `processes.insert()` fails after spawn, the Claude process continues running orphaned.

---

## High Priority Issues 🟠

### 4. Missing Input Validation with Silent Defaults
**File:** `src/mcp/server.rs:331-335`

```rust
let agent_type = args["agent_type"].as_str().unwrap_or("developer");
let session_type = args["session_type"].as_str().unwrap_or("opencode");
```

Silently defaults instead of returning errors for invalid values.

---

### 5. Panic on Invalid Database Data
**File:** `src/db/repositories/session.rs:181-183`

```rust
.unwrap()
```

These will panic if database contains corrupted data.

---

### 6. Silent Fallback on Date Parsing Errors
**File:** `src/db/repositories/project.rs:84-90`

```rust
.unwrap_or_else(|_| Utc::now())
```

Masks data corruption.

---

### 7. Single-threaded Runtime
**File:** `src/cli/commands.rs:94-96`

Cannot handle concurrent operations efficiently.

---

### 8. Blocking Calls in Async Context
**File:** `src/agent/gates.rs:19-51`

Synchronous `Command::output()` blocks the entire async runtime.

---

### 9. No HTTP Client Timeouts
**File:** `src/session/opencode/client.rs:61-67`

Missing timeouts can cause indefinite hangs.

---

## Medium Priority Issues 🟡

- Large Hardcoded Strings in agent prompts
- Content-Type check is dead code
- Large handle_request method (~420 lines)
- Confusing fallback pattern in repository
- Blocking mutex in async code

---

## Security Notes

1. **Command Injection** (Critical): Address in gates.rs
2. **Path Traversal**: Validate all file paths
3. **No Authentication**: MCP server has no auth
4. **Error Messages**: May leak sensitive paths

---

## Test Coverage

- **Status:** No tests visible
- **Recommendation:** Add unit tests, integration tests, mocks for external APIs
