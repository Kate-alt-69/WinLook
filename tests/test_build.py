import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import build


class BuildHelpersTests(unittest.TestCase):
    def test_bgw_target_requires_binary_kind_and_supported_name(self):
        self.assertTrue(build.is_bgw_target("winlook.bg", ["bin"]))
        self.assertTrue(build.is_bgw_target("winlook-bg", ["bin"]))
        self.assertTrue(build.is_bgw_target("winlook_bg", ["bin"]))
        self.assertFalse(build.is_bgw_target("winlook", ["bin"]))
        self.assertFalse(build.is_bgw_target("winlook.bg", ["lib"]))

    def test_metadata_selection_returns_only_bgw_binary_targets(self):
        metadata = {
            "packages": [
                {
                    "name": "winlook-win10",
                    "targets": [
                        {"name": "winlook", "kind": ["bin"]},
                        {"name": "winlook.bg", "kind": ["bin"]},
                        {"name": "wl_ui", "kind": ["lib"]},
                    ],
                },
                {
                    "name": "winlook-win11",
                    "targets": [
                        {"name": "winlook-bg", "kind": ["bin"]},
                    ],
                },
            ]
        }
        self.assertEqual(
            build.select_bgw_targets(metadata),
            [("winlook-win10", "winlook.bg"), ("winlook-win11", "winlook-bg")],
        )

    def test_manager_ui_requires_all_three_manifests(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self.assertFalse(build.check_ui_sources(root))
            for path in (
                "rust/library/wl_ui/Cargo.toml",
                "rust/win10/Cargo.toml",
                "rust/win11/Cargo.toml",
            ):
                target = root / path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("[package]\nname = 'test'\n", encoding="utf-8")
            self.assertTrue(build.check_ui_sources(root))

    def test_only_supported_modern_apps_can_be_packaged(self):
        self.assertEqual(
            build.package_filename("win10"),
            "winlook-win10-x64-preview.zip",
        )
        self.assertEqual(
            build.package_filename("win11"),
            "winlook-win11-x64-preview.zip",
        )
        with self.assertRaises(ValueError):
            build.package_filename("win95")

    def test_runner_dry_run_does_not_execute_unknown_command(self):
        runner = build.Runner(root=Path.cwd(), dry_run=True)
        result = runner.run("dry-run test", ["this-command-must-not-exist"])
        self.assertEqual(result.returncode, 0)

    def test_format_command_uses_argument_quoting(self):
        command = ["cmake", "-S", "C:/WinLook Project/C/library"]
        rendered = build.format_command(command)
        self.assertIn("cmake", rendered)
        self.assertIn("C:/WinLook Project/C/library", rendered)


if __name__ == "__main__":
    unittest.main()
