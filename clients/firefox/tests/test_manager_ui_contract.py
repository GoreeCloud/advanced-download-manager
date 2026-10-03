import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class ManagerUiContractTests(unittest.TestCase):
    def test_manager_zero_state_and_bulk_controls(self):
        html = (ROOT / "ui" / "manager.html").read_text(encoding="utf-8")
        source = (ROOT / "ui" / "manager.js").read_text(encoding="utf-8")
        common = (ROOT / "ui" / "common.css").read_text(encoding="utf-8")
        self.assertIn('id="captureStatus"', html)
        self.assertIn('class="summary-card active"', html)
        self.assertIn('class="summary-card queued"', html)
        self.assertIn('class="summary-card completed"', html)
        self.assertIn('class="summary-card speed"', html)
        self.assertIn('header-meta', html)
        self.assertIn('--surface-soft:', common)
        self.assertIn('--surface-elevated:', common)
        self.assertIn('@media (prefers-reduced-transparency: reduce)', common)
        self.assertIn('@media (forced-colors: active)', common)
        self.assertIn('id="listTools" class="controls" hidden', html)
        self.assertIn('id="announcement" class="sr-only" aria-live="polite"', html)
        self.assertIn('id="jobs" class="panel" aria-label="Managed downloads"', html)
        self.assertIn('$("#listTools").hidden = jobs.length === 0;', source)
        self.assertIn('$("#pauseAll").disabled = !jobs.some', source)
        self.assertIn('$("#resumeAll").disabled = !jobs.some', source)
        self.assertIn('$("#clearCompleted").disabled = !jobs.some', source)
        self.assertIn('capture.textContent = enabled ? "Auto-capture on" : "Auto-capture off";', source)
        self.assertIn('title.textContent = "No downloads yet";', source)
        self.assertIn('if (announcement !== lastAnnouncement)', source)

    def test_popup_zero_state_is_structured(self):
        html = (ROOT / "ui" / "popup.html").read_text(encoding="utf-8")
        source = (ROOT / "ui" / "popup.js").read_text(encoding="utf-8")
        self.assertIn('id="popupAnnouncement" class="sr-only" aria-live="polite"', html)
        self.assertIn('id="jobs" class="panel" aria-label="Recent downloads"', html)
        self.assertIn('empty.className = "empty";', source)
        self.assertIn('title.textContent = "No downloads yet";', source)
        self.assertIn('icon.alt = "";', source)
        self.assertIn('if (announcement !== lastPopupAnnouncement)', source)

if __name__ == "__main__":
    unittest.main()
