@echo off
set "WORKDIR=F:/workspace/github.com/opentpl"

echo starting compiler environment with docker...
echo workspace: "%PROJ%"
set "PROJ="
for %%i in ("%cd%") do set PROJ=%%~ni
echo project: "%PROJ%"
set "PROJ=/workspace/%PROJ%"
echo command:
echo cargo test -- --nocapture
docker run -it --rm -v %WORKDIR%:/workspace/ -w %PROJ% rust bash