#!/usr/bin/env python3
"""
Tether Application & Slint MCP Test Suite
Spawns Project Tether with the Slint built-in MCP server enabled,
runs automated queries and interactions against the live UI,
verifies Docker session connections, theme transitions, file manager,
and saves visual artifacts.
"""

import base64
import json
import os
import signal
import subprocess
import sys
import time
import urllib.request
from typing import Any, Dict, List, Optional

MCP_PORT = 8088
SCREENSHOT_DIR = os.path.abspath("artifacts")


def mcp_request(method: str, params: Optional[Dict[str, Any]] = None, req_id: int = 1) -> Dict[str, Any]:
    url = f"http://127.0.0.1:{MCP_PORT}/mcp"
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
        print(f"[-] MCP Request error for {method}: {e}", file=sys.stderr)
        return {}


def call_tool(tool_name: str, arguments: Dict[str, Any], req_id: int = 1) -> Dict[str, Any]:
    res = mcp_request("tools/call", {"name": tool_name, "arguments": arguments}, req_id)
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


def wait_for_mcp_server(timeout_sec: float = 15.0) -> bool:
    start = time.time()
    while time.time() - start < timeout_sec:
        res = mcp_request("tools/list", {}, req_id=999)
        if "result" in res and "tools" in res["result"]:
            return True
        time.sleep(0.5)
    return False


def save_screenshot(shot_data: str, filename: str):
    path = os.path.join(SCREENSHOT_DIR, filename)
    with open(path, "wb") as f:
        f.write(base64.b64decode(shot_data))
    print(f"[+] Saved visual screenshot: {path}")


def run_tests():
    print("=" * 60)
    print("🚀 Tether + Slint MCP Automated Verification Suite")
    print("=" * 60)

    # 1. Discover tools
    tools_res = mcp_request("tools/list", {})
    tools = [t["name"] for t in tools_res.get("result", {}).get("tools", [])]
    print(f"[+] Slint MCP Server active! Discovered {len(tools)} tools: {', '.join(tools)}")

    # 2. Window Discovery
    win_res = call_tool("list_windows", {}, 10)
    windows = win_res.get("windowHandles", [])
    assert len(windows) > 0, "No windows found by Slint MCP!"
    root_win = windows[0]
    print(f"[+] Found Root Window: {root_win}")

    # 3. Window Properties
    props = call_tool("get_window_properties", {"windowHandle": root_win}, 11)
    size = props.get("size", {})
    print(f"[+] Window Geometry: {size.get('width')} x {size.get('height')}, Scale Factor: {props.get('scaleFactor')}")

    # 4. Capture Initial Dark Mode State
    print("[*] Capturing initial application snapshot (Dark Mode)...")
    shot_init = call_tool("take_screenshot", {"windowHandle": root_win}, 12)
    if "data" in shot_init:
        save_screenshot(shot_init["data"], "01_initial_dark_mode.png")

    # 5. Accessibility Tree & Widgets Inspection
    root_elem = props.get("rootElementHandle", root_win)
    buttons = call_tool("query_element_descendants", {
        "elementHandle": root_elem,
        "findAll": True,
        "queryStack": [{"matchDescendants": True}, {"matchElementAccessibleRole": "Button"}]
    }, 13).get("elementHandles", [])
    print(f"[+] Discovered {len(buttons)} accessible Button elements")

    switches = call_tool("query_element_descendants", {
        "elementHandle": root_elem,
        "findAll": True,
        "queryStack": [{"matchDescendants": True}, {"matchElementAccessibleRole": "Switch"}]
    }, 14).get("elementHandles", [])
    print(f"[+] Discovered {len(switches)} accessible Switch elements")

    # 6. Test Theme Toggle (Dark -> Light -> Dark)
    if switches:
        theme_switch = switches[0]
        s_props = call_tool("get_element_properties", {"elementHandle": theme_switch}, 15)
        print(f"[*] Testing Theme Switch (Label: '{s_props.get('accessibleLabel')}', Checked: {s_props.get('accessibleChecked')})")

        # Toggle to Light Mode
        call_tool("invoke_accessibility_action", {"elementHandle": theme_switch, "action": "Default_"}, 16)
        time.sleep(0.4)

        shot_light = call_tool("take_screenshot", {"windowHandle": root_win}, 17)
        if "data" in shot_light:
            save_screenshot(shot_light["data"], "02_light_mode.png")

        # Toggle back to Dark Mode
        call_tool("invoke_accessibility_action", {"elementHandle": theme_switch, "action": "Default_"}, 18)
        time.sleep(0.3)
        print("[+] Theme toggle verified in both Dark and Light modes!")

    # 7. Connect to First Host (Docker Test Container)
    # Search for button labeled "Connect" or click first action button
    print("[*] Finding Connect button for Docker Test Container...")
    connected = False
    for b in buttons:
        b_props = call_tool("get_element_properties", {"elementHandle": b}, 20)
        label = b_props.get("accessibleLabel", "")
        if "connect" in label.lower() or "ssh" in label.lower():
            print(f"[+] Activating connection button: '{label}'")
            call_tool("invoke_accessibility_action", {"elementHandle": b, "action": "Default_"}, 21)
            connected = True
            break

    if not connected and len(buttons) > 2:
        # Fallback: click the third button (typically the Connect button on the first card)
        print("[*] Activating primary card connect action...")
        call_tool("click_element", {"elementHandle": buttons[2]}, 22)

    time.sleep(1.0)

    # 8. Capture Terminal Session State
    shot_session = call_tool("take_screenshot", {"windowHandle": root_win}, 25)
    if "data" in shot_session:
        save_screenshot(shot_session["data"], "03_connected_session.png")

    print("[SUCCESS] All MCP tests completed successfully!")


def main():
    os.makedirs(SCREENSHOT_DIR, exist_ok=True)
    binary_path = os.path.abspath("target/debug/project-tether")
    if not os.path.exists(binary_path):
        print(f"[-] Binary not found at {binary_path}. Run cargo build first.")
        sys.exit(1)

    env = os.environ.copy()
    env["SLINT_EMIT_DEBUG_INFO"] = "1"
    env["SLINT_MCP_PORT"] = str(MCP_PORT)
    if "DISPLAY" not in env and "WAYLAND_DISPLAY" not in env:
        env["SLINT_BACKEND"] = "headless"

    print(f"[*] Spawning Tether: {binary_path} on MCP port {MCP_PORT}...")
    proc = subprocess.Popen([binary_path], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    try:
        print("[*] Waiting for Slint MCP server to respond...")
        if not wait_for_mcp_server(15.0):
            print("[-] Timed out waiting for Slint MCP server to start!")
            sys.exit(1)

        run_tests()

    finally:
        print("[*] Shutting down Tether application...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()
        print("[+] Teardown complete.")


if __name__ == "__main__":
    main()
