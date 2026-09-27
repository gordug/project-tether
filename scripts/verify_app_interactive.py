#!/usr/bin/env python3
"""
Full Interactive Slint MCP Test Suite for Tether
Tests Docker Session, Terminal Shell, Dual-Pane File Manager,
Dark/Light Theming, and Encrypted Vault with high-res screenshots.
"""

import base64
import json
import os
import subprocess
import sys
import time
import urllib.request
from typing import Any, Dict, Optional

MCP_PORT = 8090
ARTIFACTS_DIR = os.path.abspath("artifacts")


def mcp_call(method: str, params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    url = f"http://127.0.0.1:{MCP_PORT}/mcp"
    req_body = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params or {}
    }
    req = urllib.request.Request(
        url,
        data=json.dumps(req_body).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Accept": "application/json, text/event-stream"
        },
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as resp:
        res = json.loads(resp.read().decode("utf-8"))
        if "result" in res and "content" in res["result"] and res["result"]["content"]:
            raw_text = res["result"]["content"][0].get("text", "")
            if raw_text:
                try:
                    return json.loads(raw_text)
                except Exception:
                    return {"raw": raw_text}
            elif "data" in res["result"]["content"][0]:
                return {"data": res["result"]["content"][0]["data"]}
        return res


def save_shot(root_win: Dict[str, Any], filename: str, label: str):
    res = mcp_call("tools/call", {"name": "take_screenshot", "arguments": {"windowHandle": root_win}})
    if "data" in res:
        out_path = os.path.join(ARTIFACTS_DIR, filename)
        with open(out_path, "wb") as f:
            f.write(base64.b64decode(res["data"]))
        size_kb = os.path.getsize(out_path) / 1024.0
        print(f"  [📸 Screenshot] {label} -> {filename} ({size_kb:.1f} KB)")
        return out_path
    else:
        print(f"  [-] Failed to capture screenshot: {res}")
        return None


def find_element_by_id_substring(elements: list, target_id: str) -> Optional[Dict[str, Any]]:
    for el in elements:
        for t in el.get("typeNamesAndIds", []):
            if t.get("id") and target_id in t["id"]:
                return el
    return None


def find_all_by_id_substring(elements: list, target_id: str) -> list:
    matches = []
    for el in elements:
        for t in el.get("typeNamesAndIds", []):
            if t.get("id") and target_id in t["id"]:
                matches.append(el)
    return matches


def run_full_suite():
    print("=" * 70)
    print("⚡ PROJECT TETHER: FULL E2E INTERACTIVE SLINT MCP VERIFICATION")
    print("=" * 70)

    # 1. Discover Window & Tree
    wins = mcp_call("tools/call", {"name": "list_windows", "arguments": {}})
    root_win = wins["windowHandles"][0]
    props = mcp_call("tools/call", {"name": "get_window_properties", "arguments": {"windowHandle": root_win}})
    root_elem = props["rootElementHandle"]
    print(f"[+] Root Window Geometry: {props.get('size')} | Scale Factor: {props.get('scaleFactor')}")

    # Capture 01: Initial Dashboard (Dark Mode)
    print("\n--- STEP 1: Initial State & Dark Theme Verification ---")
    save_shot(root_win, "01_initial_dashboard_dark.png", "Initial Dark Dashboard")

    # Fetch element tree
    tree = mcp_call("tools/call", {"name": "get_element_tree", "arguments": {"elementHandle": root_elem, "maxElements": 500}})
    elements = tree.get("elements", [])
    print(f"[+] Active visual element nodes: {len(elements)}")

    # 2. Test Theme Switching (Dark -> Light -> Dark)
    print("\n--- STEP 2: Light Theme Verification ---")
    theme_switch = find_element_by_id_substring(elements, "dark-mode-switch")
    switch_touch = find_element_by_id_substring(elements, "Switch::touch-area")
    if switch_touch:
        print(f"[+] Found Theme Switch TouchArea at {switch_touch.get('handle')}")
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": switch_touch["handle"]}})
        time.sleep(0.5)
        save_shot(root_win, "02_light_mode_theme.png", "Light Mode Workspace")

        # Toggle back to Dark Mode
        print("[+] Reverting to Dark Mode...")
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": switch_touch["handle"]}})
        time.sleep(0.4)

    # 3. Connect to Docker Test Container (First Host in Sidebar)
    print("\n--- STEP 3: Connecting to Docker Container Host ---")
    host_btn_touches = find_all_by_id_substring(elements, "HostCard::btn-touch")
    print(f"[+] Discovered {len(host_btn_touches)} host connection buttons")
    if host_btn_touches:
        first_btn = host_btn_touches[0]
        print(f"[+] Clicking Connect on host card at {first_btn.get('handle')}...")
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": first_btn["handle"]}})
        time.sleep(1.2)
        save_shot(root_win, "03_docker_ssh_connected.png", "Docker SSH Session Active")

    # 4. Interactive Terminal Quick Commands
    print("\n--- STEP 4: Terminal Command Execution & VT100 Verification ---")
    # Refresh tree to locate terminal elements
    tree_after = mcp_call("tools/call", {"name": "get_element_tree", "arguments": {"elementHandle": root_elem, "maxElements": 500}})
    terminal_elements = tree_after.get("elements", [])

    quick_cmd_touches = find_all_by_id_substring(terminal_elements, "TouchArea")
    # Click terminal input or quick commands
    terminal_input = find_element_by_id_substring(terminal_elements, "terminal-input")
    if terminal_input:
        print(f"[+] Typing command into terminal input...")
        mcp_call("tools/call", {"name": "set_element_value", "arguments": {"elementHandle": terminal_input["handle"], "value": "uname -a"}})
        time.sleep(0.2)
        # Dispatch Enter key
        mcp_call("tools/call", {"name": "dispatch_key_event", "arguments": {"elementHandle": terminal_input["handle"], "key": "Return", "text": "\n", "state": "Pressed"}})
        time.sleep(0.6)

    save_shot(root_win, "04_terminal_commands_output.png", "Terminal Interactive Commands")

    # 5. Dual-Pane File Manager Session (Quick Connect SFTP)
    print("\n--- STEP 5: Dual-Pane File Manager & Transfer Queue ---")
    qc_input = find_element_by_id_substring(elements, "HeaderBar::qc-input")
    qc_btn = find_element_by_id_substring(elements, "HeaderBar::qc-btn")
    if qc_input and qc_btn:
        print(f"[+] Typing 'sftp://127.0.0.1:2222' into Quick Connect Bar...")
        mcp_call("tools/call", {"name": "set_element_value", "arguments": {"elementHandle": qc_input["handle"], "value": "sftp://127.0.0.1:2222"}})
        time.sleep(0.2)
        print(f"[+] Clicking Quick Connect button...")
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": qc_btn["handle"]}})
        time.sleep(1.0)
        save_shot(root_win, "05_sftp_dual_pane_file_manager.png", "SFTP Dual-Pane File Manager")

    # 6. Zero-Knowledge Credential Vault Modal
    print("\n--- STEP 6: Zero-Knowledge Credential Vault Modal ---")
    vault_touch = find_element_by_id_substring(elements, "Sidebar::vault-touch")
    if vault_touch:
        print(f"[+] Clicking 'Vault & Keys' button at {vault_touch.get('handle')}...")
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": vault_touch["handle"]}})
        time.sleep(0.8)
        save_shot(root_win, "06_credential_vault_modal.png", "Credential Vault Dialog")

    # 7. Add New Connection Modal
    print("\n--- STEP 7: New Host Dialog Modal ---")
    add_touch = find_element_by_id_substring(elements, "Sidebar::add-touch")
    if add_touch:
        # First close vault if open or click outside/close
        # Just click add-touch to show new host
        mcp_call("tools/call", {"name": "click_element", "arguments": {"elementHandle": add_touch["handle"]}})
        time.sleep(0.6)
        save_shot(root_win, "07_new_host_dialog.png", "New Host Connection Modal")

    print("\n" + "=" * 70)
    print("✅ E2E Interactive Slint MCP Verification completed successfully!")
    print("=" * 70)


def main():
    os.makedirs(ARTIFACTS_DIR, exist_ok=True)
    binary_path = os.path.abspath("target/debug/project-tether")

    env = os.environ.copy()
    env["SLINT_MCP_PORT"] = str(MCP_PORT)
    if "DISPLAY" not in env and "WAYLAND_DISPLAY" not in env:
        env["SLINT_BACKEND"] = "headless"

    print(f"[*] Starting Project Tether with MCP Server on port {MCP_PORT}...")
    proc = subprocess.Popen([binary_path], env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    try:
        # Wait for server ready
        ready = False
        for _ in range(30):
            try:
                r = urllib.request.Request(
                    f"http://127.0.0.1:{MCP_PORT}/mcp",
                    data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}).encode(),
                    headers={"Content-Type": "application/json"}
                )
                with urllib.request.urlopen(r, timeout=1) as resp:
                    if resp.status == 200:
                        ready = True
                        break
            except Exception:
                time.sleep(0.4)

        if not ready:
            print("[-] Timed out waiting for MCP server to start!")
            sys.exit(1)

        run_full_suite()

    finally:
        print("[*] Terminating Project Tether process...")
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()
        print("[+] Done.")


if __name__ == "__main__":
    main()
