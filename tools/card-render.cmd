@echo off
rem Draws a sheet of sample card faces with the real web client in a real browser and saves it as
rem TestResults\web\CardRenderTests\card-sheet.png, to look at after a change to the card template.
setlocal
cd /d "%~dp0.."
dotnet build dotnet\src\Cantrip.Client.Web -v q || exit /b 1
dotnet test dotnet\tests\Cantrip.Client.Web.Tests --filter "FullyQualifiedName~CardRenderTests" || exit /b 1
echo.
echo %CD%\TestResults\web\CardRenderTests\card-sheet.png
