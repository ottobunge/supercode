#!/usr/bin/env bash
# Generate opencode.json for the Supercode MCP server
# This script should be run from the project root

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Try to find the supercode binary
if [ -f "$PROJECT_ROOT/target/release/supercode" ]; then
    SUPERCODE_BIN="$PROJECT_ROOT/target/release/supercode"
elif [ -f "$PROJECT_ROOT/target/debug/supercode" ]; then
    SUPERCODE_BIN="$PROJECT_ROOT/target/debug/supercode"
elif command -v supercode &> /dev/null; then
    SUPERCODE_BIN=$(which supercode)
else
    echo "Error: supercode binary not found. Please build the project first."
    echo "  cargo build --release"
    exit 1
fi

# Get absolute path
SUPERCODE_BIN=$(realpath "$SUPERCODE_BIN")

# Check if port 9091 is available, otherwise use next available
PORT=9091
while lsof -i :$PORT &> /dev/null; do
    PORT=$((PORT + 1))
    if [ $PORT -gt 9100 ]; then
        echo "Error: No available port found between 9091-9100"
        exit 1
    fi
done

# Generate opencode.json
cat > "$PROJECT_ROOT/opencode.json" << EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "mcp": {
    "supercode": {
      "type": "local",
      "command": ["$SUPERCODE_BIN", "serve", "--port", "$PORT"],
      "enabled": true,
      "timeout": 60000
    }
  }
}
EOF

echo "Generated opencode.json with supercode binary: $SUPERCODE_BIN"
echo "Using port: $PORT"
echo ""
echo "To use with OpenCode:"
echo "  1. Make sure OpenCode can see the MCP server: opencode mcp list"
echo "  2. The supercode MCP will be available as 'supercode'"
echo ""
echo "To start the server manually:"
echo "  $SUPERCODE_BIN serve --port $PORT"
