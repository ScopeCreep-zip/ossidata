@echo off
REM Windows flash launcher using native cmd.exe
REM Avoids Claude Code hanging by running flash in external cmd window

setlocal EnableDelayedExpansion

REM Default configuration
if "%1"=="" (set PORT=COM3) else (set PORT=%1)
if "%2"=="" (set BINARY_NAME=blink) else (set BINARY_NAME=%2)
set SCRIPT_DIR=%~dp0

echo ===================================
echo   Windows Flash Launcher
echo ===================================
echo.
echo [INFO] This will open a new Command Prompt window
echo [INFO] Port: %PORT%
echo [INFO] Binary: %BINARY_NAME%
echo.

REM Create temporary batch script for the flash process
set TEMP_SCRIPT=%TEMP%\flash_external_%RANDOM%.bat
set STATUS_FILE=%TEMP%\flash_status_latest.txt

REM Clear old status file
if exist "%STATUS_FILE%" del /F "%STATUS_FILE%" 2>nul

REM Create the temporary script
echo @echo off > "%TEMP_SCRIPT%"
echo setlocal EnableDelayedExpansion >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo echo =================================== >> "%TEMP_SCRIPT%"
echo echo     Arduino Flash Process >> "%TEMP_SCRIPT%"
echo echo =================================== >> "%TEMP_SCRIPT%"
echo echo. >> "%TEMP_SCRIPT%"
echo echo Port: %%1 >> "%TEMP_SCRIPT%"
echo echo Binary: %%2 >> "%TEMP_SCRIPT%"
echo echo. >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo cd /d "%%3" >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo set FLASH_SUCCESS=false >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo REM Try to run flash script using bash if available (Git Bash) >> "%TEMP_SCRIPT%"
echo if exist "flash-impl.sh" ( >> "%TEMP_SCRIPT%"
echo     echo [INFO] Running flash-impl.sh... >> "%TEMP_SCRIPT%"
echo     bash flash-impl.sh %%1 %%2 >> "%TEMP_SCRIPT%"
echo     if !errorlevel! equ 0 set FLASH_SUCCESS=true >> "%TEMP_SCRIPT%"
echo ) else if exist "flash.bat" ( >> "%TEMP_SCRIPT%"
echo     echo [INFO] Running flash.bat... >> "%TEMP_SCRIPT%"
echo     call flash.bat %%1 %%2 >> "%TEMP_SCRIPT%"
echo     if !errorlevel! equ 0 set FLASH_SUCCESS=true >> "%TEMP_SCRIPT%"
echo ) else ( >> "%TEMP_SCRIPT%"
echo     echo [ERROR] No flash script found! >> "%TEMP_SCRIPT%"
echo     timeout /t 3 ^>nul >> "%TEMP_SCRIPT%"
echo     exit /b 1 >> "%TEMP_SCRIPT%"
echo ) >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo echo. >> "%TEMP_SCRIPT%"
echo echo =================================== >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo REM Write status file >> "%TEMP_SCRIPT%"
echo if "!FLASH_SUCCESS!"=="true" ( >> "%TEMP_SCRIPT%"
echo     echo [SUCCESS] Flash completed successfully! >> "%TEMP_SCRIPT%"
echo     echo [SUCCESS] Arduino is running %%2 >> "%TEMP_SCRIPT%"
echo     echo SUCCESS:%%2:!date! !time! ^> "%STATUS_FILE%" >> "%TEMP_SCRIPT%"
echo     echo. >> "%TEMP_SCRIPT%"
echo     echo Window will close automatically in 3 seconds... >> "%TEMP_SCRIPT%"
echo     timeout /t 3 ^>nul >> "%TEMP_SCRIPT%"
echo ) else ( >> "%TEMP_SCRIPT%"
echo     echo [FAILED] Flash process failed! >> "%TEMP_SCRIPT%"
echo     echo Check the output above for errors >> "%TEMP_SCRIPT%"
echo     echo FAILED:%%2:!date! !time! ^> "%STATUS_FILE%" >> "%TEMP_SCRIPT%"
echo     echo. >> "%TEMP_SCRIPT%"
echo     echo Window will close in 10 seconds... >> "%TEMP_SCRIPT%"
echo     echo Press any key to keep window open >> "%TEMP_SCRIPT%"
echo     timeout /t 10 >> "%TEMP_SCRIPT%"
echo ) >> "%TEMP_SCRIPT%"
echo. >> "%TEMP_SCRIPT%"
echo echo DONE ^>^> "%STATUS_FILE%" >> "%TEMP_SCRIPT%"
echo exit >> "%TEMP_SCRIPT%"

echo [ACTION] Opening Command Prompt...

REM Launch the script in a new cmd window
start "Arduino Flash" cmd /c "%TEMP_SCRIPT%" "%PORT%" "%BINARY_NAME%" "%SCRIPT_DIR%"

echo [INFO] Waiting for flash to complete...

REM Monitor for completion
set MAX_WAIT=300
set ELAPSED=0

:MONITOR_LOOP
if !ELAPSED! geq !MAX_WAIT! goto TIMEOUT

REM Check if status file exists and contains DONE
if exist "%STATUS_FILE%" (
    findstr /C:"DONE" "%STATUS_FILE%" >nul 2>&1
    if !errorlevel! equ 0 (
        REM Check if successful or failed
        findstr /C:"SUCCESS" "%STATUS_FILE%" >nul 2>&1
        if !errorlevel! equ 0 (
            echo.
            echo [COMPLETE] Flash succeeded!
            echo [SUCCESS] Arduino is now running %BINARY_NAME%
            echo.
            echo ===================================
            echo [IMPORTANT] Claude Code terminal remains responsive!
            echo ===================================

            REM Clean up
            timeout /t 2 >nul
            del /F "%TEMP_SCRIPT%" 2>nul
            del /F "%STATUS_FILE%" 2>nul
            exit /b 0
        )

        findstr /C:"FAILED" "%STATUS_FILE%" >nul 2>&1
        if !errorlevel! equ 0 (
            echo.
            echo [COMPLETE] Flash failed! Check the Command Prompt window for errors.

            REM Clean up
            timeout /t 2 >nul
            del /F "%TEMP_SCRIPT%" 2>nul
            del /F "%STATUS_FILE%" 2>nul
            exit /b 1
        )
    )
)

REM Wait 1 second and continue monitoring
timeout /t 1 >nul
set /A ELAPSED+=1
goto MONITOR_LOOP

:TIMEOUT
echo [ERROR] Flash timed out after 5 minutes
del /F "%TEMP_SCRIPT%" 2>nul
del /F "%STATUS_FILE%" 2>nul
exit /b 1