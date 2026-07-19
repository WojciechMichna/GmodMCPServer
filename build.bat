cargo build --release

call gmod_env.bat

copy /Y target\release\gmod_mcp_http_server.dll "%GMOD_PATH%\garrysmod\lua\bin\gmsv_gmod_mcp_http_server_win64.dll"

xcopy /s /e /i /y "radio_mcp" "%GMOD_PATH%\garrysmod\addons\radio_mcp"