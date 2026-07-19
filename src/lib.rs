#[macro_use] extern crate gmod;

use std::sync::Mutex;
use tiny_http::{Server, Response, Header, Method, StatusCode};
use serde_json::Value;

// --- Global Server State ---
static SERVER: Mutex<Option<Server>> = Mutex::new(None);
static TASK_QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());
static AUTH_TOKEN: Mutex<String> = Mutex::new(String::new());

// --- Exported Functions for Entity Lifecycle ---
#[lua_function]
unsafe fn set_mcp_token(lua: gmod::lua::State) -> i32 {
    if let Some(token) = lua.get_string(1) {
        *AUTH_TOKEN.lock().unwrap() = token.to_string();
        lua.get_global(lua_string!("print"));
        lua.push_string("[MCP Server HTTP] Security token successfully injected from Lua.");
        lua.call(1, 0);
    }
    0
}

#[lua_function]
unsafe fn start_mcp_server(lua: gmod::lua::State) -> i32 {
    let mut server_lock = SERVER.lock().unwrap();
    if server_lock.is_none() {
        if let Ok(server) = Server::http("0.0.0.0:8000") {
            *server_lock = Some(server);
            lua.get_global(lua_string!("print"));
            lua.push_string("[MCP Server HTTP] Entity spawned! Server started on port 8000 (0.0.0.0).");
            lua.call(1, 0);
        } else {
            lua.get_global(lua_string!("print"));
            lua.push_string("[MCP Server HTTP] ERROR: Failed to start server. Port 8000 might be in use.");
            lua.call(1, 0);
        }
    }
    0
}

#[lua_function]
unsafe fn stop_mcp_server(lua: gmod::lua::State) -> i32 {
    let mut server_lock = SERVER.lock().unwrap();
    if server_lock.is_some() {
        *server_lock = None;
        lua.get_global(lua_string!("print"));
        lua.push_string("[MCP Server HTTP] Entity removed! Server stopped and port freed.");
        lua.call(1, 0);
    }
    0
}

// --- Queue System ---
#[lua_function]
unsafe fn pop_mcp_task(lua: gmod::lua::State) -> i32 {
    let mut queue = TASK_QUEUE.lock().unwrap();
    if !queue.is_empty() {
        let task = queue.remove(0);
        lua.push_string(&task);
        return 1;
    }
    lua.push_nil();
    1
}

// --- Polling Function ---
#[lua_function]
unsafe fn poll_http_server(_lua: gmod::lua::State) -> i32 {
    let mut server_lock = SERVER.lock().unwrap();
    
    if let Some(server) = server_lock.as_mut() {
        while let Ok(Some(mut request)) = server.try_recv() {
            let url = request.url().to_string();
            let method = request.method().clone();

            // --- AUTHORIZATION SYSTEM ---
            let expected_token = AUTH_TOKEN.lock().unwrap().clone();
            let mut is_authorized = expected_token.is_empty();

            if !is_authorized {
                for header in request.headers() {
                    if header.field.equiv("Authorization") {
                        if header.value.as_str() == expected_token {
                            is_authorized = true;
                        }
                        break;
                    }
                }
            }

            let is_cors_preflight = method == Method::Options;
            let is_well_known = url.starts_with("/.well-known/");

            if !is_authorized && !is_cors_preflight && !is_well_known {
                let mut response = Response::from_string(r#"{"error": "Unauthorized"}"#)
                    .with_status_code(StatusCode(401));
                response.add_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = request.respond(response);
                continue; 
            }
            // --- END OF AUTHORIZATION ---

            let json_header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
            let close_header = Header::from_bytes(&b"Connection"[..], &b"close"[..]).unwrap();
            let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap();

            // ROUTE: Official MCP JSON-RPC 2.0 Protocol handling via POST /
            if method == Method::Post && url == "/" {
                let mut body = String::new();
                std::io::Read::read_to_string(&mut request.as_reader(), &mut body).unwrap_or_default();

                let mut response_json = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": { "code": -32700, "message": "Parse error" }
                });

                if let Ok(json_rpc) = serde_json::from_str::<Value>(&body) {
                    let req_id = json_rpc.get("id").cloned().unwrap_or(serde_json::json!(serde_json::Value::Null));
                    let rpc_method = json_rpc.get("method").and_then(|m| m.as_str()).unwrap_or("");

                    response_json = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req_id.clone(),
                        "error": { "code": -32601, "message": format!("Method '{}' not found", rpc_method) }
                    });

                    if rpc_method == "initialize" {
                        response_json = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "protocolVersion": "2024-11-05",
                                "serverInfo": { "name": "gmod-rust-mcp", "version": "1.1.0" },
                                "capabilities": { "tools": {} }
                            }
                        });
                    } 
                    else if rpc_method == "notifications/initialized" {
                        response_json = serde_json::json!({ "jsonrpc": "2.0", "result": {} });
                    } 
                    else if rpc_method == "server/discover" {
                        response_json = serde_json::json!({ "jsonrpc": "2.0", "id": req_id, "result": {} });
                    }
                    else if rpc_method == "tools/list" {
                        response_json = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": req_id,
                            "result": {
                                "tools": [
                                    {
                                        "name": "print_message",
                                        "description": "Prints a message to the Garry's Mod server console.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "message": { "type": "string", "description": "The text to print in the server console" }
                                            },
                                            "required": ["message"]
                                        }
                                    },
                                    {
                                        "name": "spawn_crate",
                                        "description": "Spawns a wooden crate directly above the radio in Garry's Mod.",
                                        "inputSchema": { "type": "object", "properties": {}, "required": [] }
                                    },
                                    {
                                        "name": "spawn_metrocop",
                                        "description": "Spawns a Metro Cop NPC randomly placed around the radio.",
                                        "inputSchema": { "type": "object", "properties": {}, "required": [] }
                                    }
                                ]
                            }
                        });
                    } 
                    else if rpc_method == "tools/call" {
                        let tool_name = json_rpc.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                        let args = json_rpc.get("params").and_then(|p| p.get("arguments")).cloned().unwrap_or(serde_json::json!({}));

                        if tool_name == "print_message" {
                            if let Some(msg) = args.get("message").and_then(|m| m.as_str()) {
                                let task_json = serde_json::json!({ "TaskPrint": { "text": msg } });
                                if let Ok(mut queue) = TASK_QUEUE.lock() { queue.push(task_json.to_string()); }

                                response_json = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": req_id,
                                    "result": { "content": [{ "type": "text", "text": "Message printed." }] }
                                });
                            } else {
                                response_json = serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": req_id,
                                    "error": { "code": -32602, "message": "Missing 'message' argument" }
                                });
                            }
                        } else if tool_name == "spawn_crate" {
                            let task_json = serde_json::json!({ "TaskSpawn": "Crate" });
                            if let Ok(mut queue) = TASK_QUEUE.lock() { queue.push(task_json.to_string()); }

                            response_json = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": req_id,
                                "result": { "content": [{ "type": "text", "text": "Crate spawned." }] }
                            });
                        } else if tool_name == "spawn_metrocop" {
                            let task_json = serde_json::json!({ "TaskSpawn": "Metrocop" });
                            if let Ok(mut queue) = TASK_QUEUE.lock() { queue.push(task_json.to_string()); }

                            response_json = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": req_id,
                                "result": { "content": [{ "type": "text", "text": "Metro Cop spawned nearby." }] }
                            });
                        }
                    }
                }

                let mut response = Response::from_string(response_json.to_string()).with_status_code(StatusCode(200));
                response.add_header(json_header);
                response.add_header(close_header);
                response.add_header(cors_header);
                let _ = request.respond(response);
            }
            // ROUTE: Disconnect 
            else if method == Method::Delete && url == "/" {
                let mut response = Response::empty(200);
                response.add_header(Header::from_bytes(&b"Connection"[..], &b"close"[..]).unwrap());
                response.add_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = request.respond(response);
            }
            // ROUTE: Preflight OPTIONS
            else if method == Method::Options {
                let mut response = Response::empty(204);
                response.add_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"GET, POST, OPTIONS"[..]).unwrap());
                response.add_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type, Authorization"[..]).unwrap());
                response.add_header(cors_header);
                response.add_header(close_header);
                let _ = request.respond(response);
            }
            // ROUTE: 404 Not Found
            else {
                let mut response = Response::from_string(r#"{"status": "error"}"#).with_status_code(StatusCode(404));
                response.add_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                response.add_header(close_header);
                response.add_header(cors_header);
                let _ = request.respond(response);
            }
        }
    }
    0
}

#[gmod13_open]
fn gmod13_open(lua: gmod::lua::State) -> i32 {
    unsafe {
        lua.push_function(set_mcp_token); lua.set_global(lua_string!("SetMcpToken"));
        lua.push_function(start_mcp_server); lua.set_global(lua_string!("StartMcpServer"));
        lua.push_function(stop_mcp_server); lua.set_global(lua_string!("StopMcpServer"));
        lua.push_function(poll_http_server); lua.set_global(lua_string!("RustPollMcpServer"));
        lua.push_function(pop_mcp_task); lua.set_global(lua_string!("PopMcpTask"));
    }
    0
}

#[gmod13_close]
fn gmod13_close(_lua: gmod::lua::State) -> i32 {
    *SERVER.lock().unwrap() = None;
    if let Ok(mut queue) = TASK_QUEUE.lock() { queue.clear(); }
    0
}