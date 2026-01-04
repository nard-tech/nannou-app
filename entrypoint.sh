#!/bin/bash
set -e

# Fix ownership of cargo directories if they are mounted as volumes
if [ -d /usr/local/cargo ]; then
    chown -R app:app /usr/local/cargo 2>/dev/null || true
fi

# Fix ownership of workspace target directory if it is mounted as a volume
if [ -d /workspace/target ]; then
    chown -R app:app /workspace/target 2>/dev/null || true
fi

# Fix ownership of workspace directory
chown -R app:app /workspace 2>/dev/null || true

# Switch to app user and execute the command
exec gosu app "$@"
