@echo off
setlocal enabledelayedexpansion

REM Define the list of targets
set TARGETS=Accessory Cap Cape Coat Glove Longcoat Pants PetEquip Ring Shield Shoes Weapon

REM Base path for MapleStory data
set BASE_PATH=C:\Program Files\gamania Games\MapleStory\Data\Character
set OUTPUT_PATH=..\ms_data

echo Starting extraction process...
echo.

REM Loop through each target
for %%t in (%TARGETS%) do (
    echo ================================================
    echo Processing: %%t
    echo ================================================

    REM Process all {target}_{number}.wz files
    echo.
    echo [1/2] Converting %%t_*.wz files to JSON...
    for %%f in ("%BASE_PATH%\%%t\%%t_*.wz") do (
        if exist "%%f" (
            echo   - Processing: %%~nxf
            cargo run --example wz_to_json --features="json" -- "%%f" "%OUTPUT_PATH%"
        )
    )

    REM Process {target}.wz file for PNG extraction
    echo.
    echo [2/2] Extracting PNGs from %%t.wz...
    if exist "%BASE_PATH%\%%t\%%t.wz" (
        cargo run --example extracting_pngs --features "image/png" -- folder "%BASE_PATH%\%%t\%%t.wz" "%OUTPUT_PATH%\%%t"
    ) else (
        echo   Warning: %%t.wz not found, skipping PNG extraction
    )

    echo.
)

echo ================================================
echo Extraction process completed!
echo ================================================
pause
