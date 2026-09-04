#!/usr/bin/env bash
# ==============================================================================
# Micro-Crucible Launcher for macOS / Linux
# Starts FastAPI Backend (Port 8081) and React Frontend (Port 8080)
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "${SCRIPT_DIR}")"

echo "================================================================================"
echo "  🔬 Starting Micro-Crucible Target Sandbox (FastAPI :8081 + React :8080) 🔬"
echo "================================================================================"

# Check Python
if ! command -v python3 &> /dev/null && ! command -v python &> /dev/null; then
    echo "❌ Python is not installed. Please install Python 3.11+."
    exit 1
fi
PYTHON_CMD="python3"
if ! command -v python3 &> /dev/null; then
    PYTHON_CMD="python"
fi

# Check Node / npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm / Node.js is not installed. Please install Node.js 18+."
    exit 1
fi

# 1. Install backend dependencies if needed
echo "📦 Verifying Python backend dependencies..."
"${PYTHON_CMD}" -m pip install -q -r "${ROOT_DIR}/crucible/backend/requirements.txt" 2>/dev/null || true

# 2. Install frontend dependencies if needed
if [ ! -d "${ROOT_DIR}/crucible/frontend/node_modules" ]; then
    echo "📦 Installing React frontend dependencies..."
    (cd "${ROOT_DIR}/crucible/frontend" && npm install --silent)
fi

# Trap to kill child processes on exit
cleanup() {
    echo ""
    echo "🛑 Shutting down Micro-Crucible services..."
    kill $(jobs -p) 2>/dev/null || true
    exit 0
}
trap cleanup SIGINT SIGTERM EXIT

# 3. Start Backend on 8081
echo "🚀 Starting FastAPI Backend on http://127.0.0.1:8081..."
(cd "${ROOT_DIR}" && "${PYTHON_CMD}" -m uvicorn crucible.backend.app:app --host 127.0.0.1 --port 8081 --log-level info) &

# 4. Start Frontend on 8080
echo "🚀 Starting React Frontend on http://localhost:8080..."
(cd "${ROOT_DIR}/crucible/frontend" && npx vite --port 8080 --host 127.0.0.1) &

echo "================================================================================"
echo "  ✅ Sandbox Live!"
echo "  • Frontend:        http://localhost:8080"
echo "  • Mission Control: http://localhost:8080/mission-control"
echo "  • Backend API:     http://localhost:8081/docs"
echo "  Press Ctrl+C to stop all services."
echo "================================================================================"

wait
