import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import build


class BuildHelpersTests(unittest.TestCase):
    def test_c_and_rust_share_build_root_but_have_separate_outputs_and_caches(self):
        root = Path("D:/WinLook")
        paths = build.build_paths(root)
        self.assertEqual(paths["rust"], root / "build" / "rust")
        self.assertEqual(paths["c"], root / "build" / "c")
        self.assertEqual(paths["rust_cache"], root / "build" / ".rcache")
        self.assertEqual(paths["c_cache"], root / "build" / ".Ccache")
        self.assertEqual(paths["artifacts"], root / "build" / "artifacts")

    def test_rust_setup_is_opt_in_and_watcom_can_be_selected_separately(self):
        parser = build.build_parser()
        watcom = parser.parse_args(["setup", "--open-watcom"])
        self.assertTrue(watcom.open_watcom)
        self.assertFalse(watcom.with_rust)
        rust = parser.parse_args(["setup", "--with-rust"])
        self.assertTrue(rust.with_rust)
        self.assertFalse(rust.open_watcom)
        c = parser.parse_args(["setup", "--with-c"])
        self.assertTrue(c.with_c)
        self.assertFalse(c.with_rust)


    def test_rust_toolchain_is_scoped_to_repository_build_cache(self):
        root = Path("D:/WinLook")
        paths = build.rust_paths(root)
        env = build.rust_environment(root)
        self.assertEqual(paths["cargo_home"], root / "build" / ".rcache" / "cargo-home")
        self.assertEqual(paths["rustup_home"], root / "build" / ".rcache" / "rustup-home")
        self.assertEqual(env["CARGO_HOME"], str(paths["cargo_home"]))
        self.assertEqual(env["RUSTUP_HOME"], str(paths["rustup_home"]))
        self.assertEqual(env["RUSTUP_TOOLCHAIN"], build.RUST_TOOLCHAIN)
        self.assertTrue(env["PATH"].startswith(str(paths["cargo_home"] / "bin")))

    def test_rustup_host_selects_windows_msvc_for_windows_builds(self):
        with mock.patch.object(build.os, "name", "nt"), mock.patch.object(build.platform, "machine", return_value="AMD64"):
            self.assertEqual(build.rustup_host(), "x86_64-pc-windows-msvc")

    def test_setup_parser_exposes_c_compiler_and_cmake_options(self):
        args = build.build_parser().parse_args(["setup", "--with-c", "--with-cmake", "--apply"])
        self.assertTrue(args.with_c)
        self.assertTrue(args.with_cmake)
        self.assertTrue(args.apply)

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

    def test_target_status_reports_gated_legacy_and_bgw_components(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self.assertEqual(
                build.target_available(root, "win95"),
                (False, "C/win95/Makefile is not present"),
            )
            self.assertEqual(
                build.target_available(root, "bgw"),
                (False, "BGW binary target is not implemented"),
            )
            manifest = root / "rust" / "win10" / "Cargo.toml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(
                '[[bin]]\nname = "winlook.bg"\npath = "src/bin/bgw.rs"\n',
                encoding="utf-8",
            )
            self.assertTrue(build.target_available(root, "bgw")[0])

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

    def test_package_dry_run_does_not_create_dist_directory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            executable = root / "build" / "rust" / "win10" / "release" / "winlook.exe"
            executable.parent.mkdir(parents=True)
            executable.write_bytes(b"preview")
            runner = build.Runner(root=root, dry_run=True)
            result = build.package_windows_app(runner, "win10")
            self.assertEqual(result.name, "winlook-win10-x64-preview.zip")
            self.assertFalse((root / "build" / "artifacts").exists())

    def test_format_command_uses_argument_quoting(self):
        command = ["cmake", "-S", "C:/WinLook Project/C/library"]
        rendered = build.format_command(command)
        self.assertIn("cmake", rendered)
        self.assertIn("C:/WinLook Project/C/library", rendered)

    def test_no_arguments_defaults_to_read_only_doctor(self):
        with mock.patch("build.command_doctor", return_value=0) as doctor:
            self.assertEqual(build.main([]), 0)
        args = doctor.call_args.args[0]
        self.assertEqual(args.action, "doctor")
        self.assertFalse(args.strict)

    def test_workflow_commands_are_supported_by_the_cli_parser(self):
        commands = (
            ["status", "libraries"],
            ["status", "ui"],
            ["status", "win10"],
            ["status", "win11"],
            ["status", "win95"],
            ["status", "bgw"],
            ["verify", "rust"],
            ["verify", "c", "--sanitizers"],
            ["verify", "ui"],
            ["verify", "all"],
            ["build", "win10"],
            ["build", "win11"],
            ["build", "win95"],
            ["build", "bgw"],
            ["package", "win10"],
            ["package", "win11"],
            ["docs"],
            ["workflow-lint", "--apply"],
            ["secret-scan", "--apply"],
        )
        for command in commands:
            with self.subTest(command=command):
                self.assertEqual(build.build_parser().parse_args(command).action, command[0])


if __name__ == "__main__":
    unittest.main()
