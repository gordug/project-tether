---
name: slint-mcp
description: "MANDATORY: Use for verifying any Slint UI changes, inspecting elements, testing accessibility, simulating interactions, or capturing runtime screenshots using Slint's built-in MCP server. Keywords: slint mcp, mcp verification, verify ui, test ui, inspect ui, ui testing, ui screenshot, accessibility check, dark mode verify, click element, ui validation, UI 验证, MCP 测试, 端到端UI验证"
---

# Slint MCP Verification Skill

This skill governs the **mandatory runtime verification protocol** for all Slint UI changes in the project.

> [!IMPORTANT]
> **MANDATORY VERIFICATION GATE**: Never declare any UI work (new components, layout refactoring, theming, accessibility attributes, or user interaction wiring) complete without running the application with Slint's built-in MCP server, querying the element tree, executing interactions, and visually verifying the result across both Dark and Light modes.

---

## 1. Quick Workflow

Whenever UI modifications are made:

1. **Pre-flight Compile Check**:
   ```bash
   cargo check --features slint/mcp
   ```
2. **Launch Application in Background with MCP Server**:
   ```bash
   SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 cargo run -p project-tether --features slint/mcp
   ```
   *(Note: Use `SLINT_BACKEND=headless` if running in an environment without an active display server).*
3. **Verify Server Ready**:
   ```bash
   curl -s -X POST http://127.0.0.1:8080/mcp \
     -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
     -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
   ```
4. **Discover Windows and Elements**:
   - `list_windows` to get the root `windowHandle`.
   - `query_element_descendants` or `find_elements_by_id` to locate target widgets.
5. **Inspect Properties & Accessibility**:
   - `get_element_properties` to verify `accessibleRole`, `accessibleLabel`, `accessibleChecked`, and geometry.
6. **Simulate User Interactions**:
   - `invoke_accessibility_action` (e.g. `action: "Default_"` to toggle switches/buttons).
   - `click_element` to simulate mouse clicks.
   - `set_element_value` to enter text or set slider positions.
   - `dispatch_key_event` to simulate keystrokes.
7. **Verify Visual Rendering in Both Themes**:
   - Capture screenshot via `take_screenshot`.
   - Toggle theme (e.g. Dark Mode Switch).
   - Verify readability, contrast, and absence of hardcoded illegible colors.
8. **Teardown**:
   - Terminate the background application when verification is complete.

---

## 2. Standard JSON-RPC MCP Commands

The Slint MCP server listens on `http://127.0.0.1:<PORT>/mcp` using JSON-RPC 2.0. Always pass headers:
`-H "Content-Type: application/json" -H "Accept: application/json, text/event-stream"`

### A. Discover Available Tools
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

### B. List Open Windows
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_windows","arguments":{}}}'
```
*Response returns:* `{"windowHandles": [{"generation": "1", "index": "1"}]}`

### C. Find Elements by Semantic Role or Type
Find all `Switch` or `Button` elements:
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"query_element_descendants","arguments":{"elementHandle":{"generation":"1","index":"1"},"findAll":true,"queryStack":[{"matchDescendants":true},{"matchElementAccessibleRole":"Switch"}]}}}'
```

### D. Find Elements by ID
Find an element assigned an ID in Slint (e.g. `dark_mode_switch := Switch {}` inside `HeaderBar`):
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"find_elements_by_id","arguments":{"windowHandle":{"generation":"1","index":"1"},"elementsId":"HeaderBar::dark-mode-switch"}}}'
```

### E. Inspect Element Properties & Accessibility
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"get_element_properties","arguments":{"elementHandle":{"generation":"1","index":"2"}}}}'
```
*Returns:*
* `accessibleRole` (e.g. `"Switch"`, `"Button"`, `"TextInput"`)
* `accessibleLabel` (e.g. `"Dark mode switch"`)
* `accessibleCheckable` and `accessibleChecked`
* `absolutePosition` (`x`, `y`) and `size` (`width`, `height`)
* `computedOpacity`

### F. Trigger Accessibility Action
Toggles a switch or activates a button:
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"invoke_accessibility_action","arguments":{"elementHandle":{"generation":"1","index":"2"},"action":"Default_"}}}'
```

### G. Simulate Pointer Click
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"click_element","arguments":{"elementHandle":{"generation":"1","index":"2"}}}}'
```

### H. Input Text into TextInput
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"set_element_value","arguments":{"elementHandle":{"generation":"1","index":"4"},"value":"192.168.1.100"}}}'
```

### I. Capture Window Screenshot
```bash
curl -s -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"take_screenshot","arguments":{"windowHandle":{"generation":"1","index":"1"}}}}' \
  | jq -r '.result.content[0].data' | base64 -d > screenshot.png
```

---

## 3. The Light/Dark Contrast Verification Rule

When implementing or modifying UI:
1. **Never use hardcoded hex colors** like `#ffffff` or `#161b22` for surfaces or text unless it is inside a fixed console canvas.
2. Always bind surfaces and text to `Theme.bg_*` and `Theme.text_*`.
3. Use the verification script or manual curl commands to capture screenshots in **both Dark Mode (`Theme.dark = true`) and Light Mode (`Theme.dark = false`)**.
4. Check pixel luminance at key surfaces (sidebar, cards, hero banners, headers) to guarantee contrast ratio \u2265 4.5:1.

---

## 4. Automated Verification Script

Use `scripts/verify_mcp.py` to run an automated sanity suite:
```bash
python3 .agents/skills/slint-mcp/scripts/verify_mcp.py --port 8080 --screenshot-dir ./artifacts
```
It will:
- Validate server availability.
- Discover all windows and switches.
- Toggle dark/light mode and capture screenshots of both states.
- Verify text input and click handlers.
- Assert that contrast and accessibility trees are intact.
