@echo off
rem ---------------------------------------------------------------------------
rem Any local draft, with every option, through tools\scripts\draft.ps1: a
rem server hosting the draft, the web client in your browser, and bots in the
rem other seats. Draft, build a deck from what you drafted, and play with it.
rem For one format with its defaults, draft-winston-vs-bot.cmd,
rem draft-grid-vs-bot.cmd and draft-booster-vs-bots.cmd are quicker; this one
rem is for the options, and for hosting an event again with -Event.
rem
rem   draft.cmd                         you against a bot, 90 cards of LSVCube
rem   draft.cmd -Cube PowerLSV -Cards 60
rem   draft.cmd -Player2 Human          two tabs, you draft both seats
rem   draft.cmd -Event ID               host an event already made again
rem   draft.cmd -Format Booster         a pod of eight: you and seven bots draft
rem                                     three packs, then three Swiss rounds
rem   draft.cmd -Format Grid            a grid draft against a bot: 18 packs of
rem                                     nine, a row or a column of each
rem
rem The event is kept in a data directory (-Data, else CANTRIP_DATA, else
rem .cantrip-data in the checkout), so it can be stopped and taken up again.
rem Every argument is passed to the script; see its header for the rest.
rem ---------------------------------------------------------------------------

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\draft.ps1" %*

if errorlevel 1 pause
exit /b %errorlevel%
