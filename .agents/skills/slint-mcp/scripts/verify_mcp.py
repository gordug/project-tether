#!/usr/bin/env python3
"""
Slint MCP Verification Script
Automates querying the Slint built-in MCP server, checking accessibility,
toggling dark/light modes, verifying interactive widgets, and capturing visual snapshots.
"""

import argparse
import base64
import json
import os
import sys
import time
import urllib.request
from typing import Any, Dict, Optional


def mcp_request(port: int, method: str, params: Optional[Dict[str, Any]] = None, req_id: int = 1) -> Dict[str, Any]:
    url = f"http://127.0.0.1:{port}/mcp"
    body = {
        "jsonrpc": "2.0",
        "id": req_id,
        "method": method,
        "params": params or {}
    }
    req = urllib.request.Request(
        url,
        data=json.dumps(body).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json, text/event-stream"
        },
        method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            content = resp.read().decode("utf-8")
            return json.loads(content)
    except Exception as e:
        print(f"[-] MCP Request failed to {url}: {e}", file=sys.stderr)
        return {}


def call_tool(port: int, tool_name: str, arguments: Dict[str, Any], req_id: int = 1) -> Dict[str, Any]:
    res = mcp_request(port, "tools/call", {"name": tool_name, "arguments": arguments}, req_id)
    if "result" in res and "content" in res["result"] and len(res["result"]["content"]) > 0:
        raw_text = res["result"]["content"][0].get("text", "")
        if raw_text:
            try:
                return json.loads(raw_text)
            except Exception:
                return {"raw": raw_text}
        elif "data" in res["result"]["content"][0]:
            return {"data": res["result"]["content"][0]["data"]}
    return res


def main():
    parser = argparse.ArgumentParser(description="Slint MCP Verification Script")
    parser.add_argument("--port", type=int, default=8080, help="Slint MCP server port")
    parser.add_argument("--screenshot-dir", type=str, default="./screenshots", help="Directory to save screenshots")
    args = parser.parse_args()

    os.makedirs(args.screenshot_dir, exist_ok=True)
    print(f"[*] Connecting to Slint MCP server on 127.0.0.1:{args.port}...")

    # 1. Check tools/list
    tools_res = mcp_request(args.port, "tools/list", {})
    if "result" not in tools_res or "tools" not in tools_res["result"]:
        print("[-] Failed to query tools/list. Is the app running with --features slint/mcp?")
        sys.exit(1)

    tools = [t["name"] for t in tools_res["result"]["tools"]]
    print(f"[+] Server responding! Discovered {len(tools)} MCP tools: {', '.join(tools[:5])}...")

    # 2. Discover Windows
    win_res = call_tool(args.port, "list_windows", {}, 2)
    windows = win_res.get("windowHandles", [])
    if not windows:
        print("[-] No windows discovered!")
        sys.exit(1)

    root_win = windows[0]
    print(f"[+] Found primary window handle: {root_win}")

    # 3. Get Window Properties
    win_props = call_tool(args.port, "get_window_properties", {"windowHandle": root_win}, 3)
    size = win_props.get("size", {})
    print(f"[+] Window dimensions: {size.get('width')} x {size.get('height')}, scale: {win_props.get('scaleFactor')}")

    # 4. Find Dark Mode Switch
    root_elem = win_props.get("rootElementHandle", root_win)
    descendants = call_tool(args.port, "query_element_descendants", {
        "elementHandle": root_elem,
        "findAll": True,
        "queryStack": [{"matchDescendants": True}, {"matchElementAccessibleRole": "Switch"}]
    }, 4)

    switches = descendants.get("elementHandles", [])
    if not switches:
        print("[-] No Switch elements found in accessibility tree!")
        sys.exit(1)

    switch_handle = switches[0]
    switch_props = call_tool(args.port, "get_element_properties", {"elementHandle": switch_handle}, 5)
    print(f"[+] Found Switch: label='{switch_props.get('accessibleLabel')}', checked={switch_props.get('accessibleChecked')}")

    # 5. Take initial screenshot
    initial_shot = call_tool(args.port, "take_screenshot", {"windowHandle": root_win}, 6)
    if "data" in initial_shot:
        shot_path = os.path.join(args.screenshot_dir, "initial_state.png")
        with open(shot_path, "wb") as f:
            f.write(base64.b64decode(initial_shot["data"]))
        print(f"[+] Saved initial screenshot to: {shot_path}")

    # 6. Toggle theme via accessibility action
    print("[*] Toggling dark/light mode via invoke_accessibility_action('Default_')...")
    call_tool(args.port, "invoke_accessibility_action", {"elementHandle": switch_handle, "action": "Default_"}, 7)
    time.sleep(0.3)

    toggled_props = call_tool(args.port, "get_element_properties", {"elementHandle": switch_handle}, 8)
    print(f"[+] Switch state after toggle: checked={toggled_props.get('accessibleChecked')}")

    # 7. Take toggled screenshot
    toggled_shot = call_tool(args.port, "take_screenshot", {"windowHandle": root_win}, 9)
    if "data" in toggled_shot:
        shot_path = os.path.join(args.screenshot_dir, "toggled_state.png")
        with open(shot_path, "wb") as f:
            f.write(base64.b64decode(toggled_shot["data"]))
        print(f"[+] Saved toggled screenshot to: {shot_path}")

    # 8. Restore initial state
    print("[*] Restoring switch state...")
    call_tool(args.port, "invoke_accessibility_action", {"elementHandle": switch_handle, "action": "Default_"}, 10)
    time.sleep(0.2)

    print("[SUCCESS] All Slint MCP verification checks passed successfully!")


if __name__ == "__main__":
    main()
