@echo off
setlocal
if not defined PYTHON_BIN set "PYTHON_BIN=python"
"%PYTHON_BIN%" "%~dp0msvc_dev_linker.py" %*
exit /b %errorlevel%
