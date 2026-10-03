import json
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class CookiePermissionContractTests(unittest.TestCase):
    def test_permission_request_is_directly_bound_to_deliberate_settings_click(self):
        source = (ROOT / "ui" / "options.js").read_text(encoding="utf-8")
        self.assertIn('$("#grantCookies").addEventListener("click", () => {', source)
        self.assertIn("browser.permissions.request(COOKIE_PERMISSION)", source)
        self.assertNotIn('runtime.sendMessage({ type: "request-cookie-permission"', source)

        handler = source.split('$("#grantCookies").addEventListener("click", () => {', 1)[1]
        guard_index = handler.index("if (!nativeMode || !forwardingRequested)")
        request_index = handler.index("browser.permissions.request(COOKIE_PERMISSION)")
        self.assertLess(guard_index, request_index)

    def test_permission_request_requires_native_mode_and_cookie_forwarding(self):
        source = (ROOT / "ui" / "options.js").read_text(encoding="utf-8")
        self.assertIn('const nativeMode = nativeModeSelected();', source)
        self.assertIn('const forwardingRequested = $("#forwardCookies").checked;', source)
        self.assertIn("if (!nativeMode || !forwardingRequested)", source)
        self.assertIn('|| !nativeMode', source)
        self.assertIn('|| !forwardingRequested', source)

    def test_cookie_permission_can_be_revoked_from_settings(self):
        source = (ROOT / "ui" / "options.js").read_text(encoding="utf-8")
        html = (ROOT / "ui" / "options.html").read_text(encoding="utf-8")
        self.assertIn('$("#revokeCookies").addEventListener("click", () => {', source)
        self.assertIn("browser.permissions.remove(COOKIE_PERMISSION)", source)
        self.assertIn('id="revokeCookies"', html)
        self.assertIn("Revoke cookie access", html)

    def test_native_settings_have_explicit_inactive_state(self):
        source = (ROOT / "ui" / "options.js").read_text(encoding="utf-8")
        html = (ROOT / "ui" / "options.html").read_text(encoding="utf-8")
        self.assertIn('id="nativeSettings"', html)
        self.assertIn('id="nativeState"', html)
        self.assertIn('aria-disabled="true"', html)
        self.assertIn('$("#mode").addEventListener("change"', source)
        self.assertIn('$("#forwardCookies").addEventListener("change"', source)
        self.assertIn('$("#nativeSettings").setAttribute("aria-disabled", String(!nativeMode))', source)
        self.assertIn('$("#test").disabled = permissionBusy || !nativeMode;', source)

    def test_permission_wording_is_disclosed_before_request(self):
        html = (ROOT / "ui" / "options.html").read_text(encoding="utf-8")
        self.assertIn("Access your data for all websites", html)
        self.assertIn("will not request it unless native mode and cookie forwarding are both selected", html)
        self.assertIn("Allow cookie access", html)

    def test_cookie_permission_remains_optional(self):
        manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
        self.assertIn("cookies", manifest.get("optional_permissions", []))
        self.assertIn("<all_urls>", manifest.get("optional_host_permissions", []))
        self.assertNotIn("cookies", manifest.get("permissions", []))

    def test_save_only_requires_permission_for_active_native_cookie_forwarding(self):
        source = (ROOT / "ui" / "options.js").read_text(encoding="utf-8")
        save_block = source.split('$("#save").addEventListener', 1)[1]
        self.assertNotIn("browser.permissions.request", save_block)
        self.assertIn('mode === "native" && forwardCookies', save_block)
        self.assertIn("Cookie forwarding was not saved", save_block)


if __name__ == "__main__":
    unittest.main()
