# Garry's Mod MCP HTTP Server

This project bridges Garry's Mod and AI agents using the Model Context Protocol (MCP). By running an embedded HTTP server within a Garry's Mod entity, it allows external LLM agents (like Claude Desktop or Antigravity) to execute commands, spawn entities, and interact with the game world in real-time via JSON-RPC 2.0.

## Demonstration

[Gmod1.webm](https://github.com/user-attachments/assets/273f56bd-cc6b-4367-98bd-b5bcd7d744e3)

## Overview

The project consists of two main components:
1. Rust Binary Module (.dll): A high-performance, multithreaded HTTP server that handles MCP JSON-RPC 2.0 requests, authorization, and queues incoming tasks.
2. Lua Addon (radio_mcp): An in-game entity ("HTTP Server Radio") that safely polls the task queue from the Rust module and executes the game logic (e.g., printing to console, spawning NPCs or props).

## Features

* Full MCP Compliance: Implements the official JSON-RPC 2.0 Model Context Protocol.
* Secure Authorization: Uses Bearer token authentication to prevent unauthorized execution.
* Safe Execution: Network requests are handled asynchronously in Rust, while game state modifications are safely queued and executed on the main Lua thread during the entity's Think tick.
* Available Tools:
  * print_message: Prints custom text directly to the server console.
  * spawn_crate: Drops a wooden crate above the radio entity.
  * spawn_metrocop: Spawns a Metro Cop NPC at a safe, randomized offset from the radio.

## Prerequisites

* Garry's Mod (x86-64 branch): The DLL is compiled for 64-bit Windows (win64.dll).
* Rust Toolchain: Required to compile the C- ABI dynamic library (Cargo).

## Installation & Build Instructions

This project includes automated batch scripts to compile the Rust code and deploy both the DLL and the Lua addon directly to your Garry's Mod installation.

IMPORTANT: Garry's Mod x86-64 Branch Required
Because this project compiles a 64-bit DLL (win64.dll), standard 32-bit Garry's Mod will fail to load it. You must switch to the 64-bit branch before launching the game:
1. Open Steam and go to your Library.
2. Right-click Garry's Mod -> Properties -> Betas.
3. Under Beta Participation, select "x86-64 - Chromium + 64-bit binaries".

### Setup Steps

1. Configure Environment & Build: 
   Copy or rename example_gmod_env.bat to gmod_env.bat. Open it and ensure the GMOD_PATH variable points to your Garry's Mod root directory. Once the path is set, double-click and run build.bat to automatically compile the Rust project and deploy the necessary files to your game.

2. Configure Security Token:
   Navigate to the radio_mcp/lua/entities/mcp_radio/ directory inside the project and create a file named mcp_password.lua. Add your secret token:
   return "Bearer FOO_BAR_123"

3. Launch the Game:
   Start Garry's Mod from Steam. When prompted, make sure you choose the 64-bit launch option.

### Manual Installation

If you prefer to install the project manually without using the batch scripts:

1. Lua Addon: Copy the radio_mcp folder directly into your garrysmod\addons\ directory.
2. Rust DLL: You can build the HTTP MCP server from source. Open a terminal in the project root and compile it using Cargo:
   cargo build --release
3. Navigate to the target\release\ directory, rename the compiled gmod_mcp_http_server.dll to gmsv_gmod_mcp_http_server_win64.dll, and place it inside your garrysmod\lua\bin\ directory (create the bin folder if it does not exist).
4. Follow Step 2 from Setup Steps to create your mcp_password.lua file.

## Agent Configuration

To connect your MCP-compatible AI agent, add the server to your agent's configuration file (e.g., mcp_config.json). Ensure the token matches the one set in your Lua files.

{
  "mcpServers": {
    "gmod-spawn": {
      "type": "http",
      "url": "http://localhost:8000",
      "headers": {
        "Authorization": "Bearer FOO_BAR_123"
      }
    }
  }
}

Note: If you are running your agent in a Docker container (like Antigravity), use http://host.docker.internal:8000 as the URL.

## Usage

1. Start a Garry's Mod sandbox game on the 64-bit branch.
2. Open the Spawn Menu (Q), navigate to the Entities tab, and find My Addons.
3. Spawn the HTTP Server Radio.
4. Check the game console. You should see a confirmation that the MCP Server has started on port 8000.
5. Issue prompts to your AI agent (e.g., "Spawn a Metro Cop in Garry's Mod").
6. Removing or destroying the radio entity will automatically shut down the HTTP server and free the port.

## Acknowledgments

Huge thanks to the creator of the gmod Rust crate WilliamVenner. This project relies heavily on their incredible work, which makes writing native Garry's Mod extensions in Rust a seamless and safe experience.
