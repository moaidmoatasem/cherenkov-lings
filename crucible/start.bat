@echo off
setlocal enabledelayedexpansion

echo ======================================================================
echo  CHERENKOV-LINGS: Micro-Crucible Target Sandbox
echo ======================================================================

echo [1/3] Verifying Python dependencies...
python -m pip install -q -r crucible\backend\requirements.txt

echo [2/3] Verifying Frontend dependencies...
cd crucible\frontend
call npm install --silent
cd ..\..

echo [3/3] Starting Backend (Port 8081) and Frontend (Port 8080)...
start "Crucible-Backend-8081" cmd /c "python -m uvicorn crucible.backend.app:app --host 127.0.0.1 --port 8081"
start "Crucible-Frontend-8080" cmd /c "cd crucible\frontend && npm run dev -- --port 8080 --host 127.0.0.1"

echo.
echo ======================================================================
echo  Micro-Crucible is LIVE:
echo    - Frontend UI:  http://localhost:8080
echo    - Backend API:  http://localhost:8081
echo    - API Docs:     http://localhost:8081/docs
echo ======================================================================
