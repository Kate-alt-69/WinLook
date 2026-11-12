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


    def test_windows_generation_selectors_include_legacy_and_modern_targets(self):
        parser = build.build_parser()
        expected = {
            "windows.95", "windows.98", "windows.me", "windows.nt4",
            "windows.2000", "windows.xp", "windows.vista", "windows.7",
            "windows.8", "windows.8.1", "windows.10", "windows.11",
        }
        for selector in expected:
            with self.subTest(selector=selector):
                self.assertEqual(parser.parse_args([selector]).action, selector)
        self.assertEqual(set(build.WINDOWS_BUILD_COMMANDS), expected)

    def test_windows_81_selector_preserves_the_full_dotted_version(self):
        args = build.build_parser().parse_args(["windows.8.1", "--dry-run"])
        self.assertEqual(args.action[len("windows."):], "8.1")
        self.assertTrue(args.dry_run)

    def test_generation_source_gate_prevents_any_compiler_setup(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = build.Runner(root=Path(temp))
            with mock.patch.object(build, "_select_mingw_cmake") as mingw:
                with self.assertRaisesRegex(build.BuildError, "C/win7/CMakeLists.txt"):
                    build.build_windows_generation(runner, "7")
                mingw.assert_not_called()

    def test_win9x_source_gate_prevents_open_watcom_setup(self):
        with tempfile.TemporaryDirectory() as temp:
            runner = build.Runner(root=Path(temp))
            with mock.patch.object(build, "command_path") as command:
                with self.assertRaisesRegex(build.BuildError, "C/win95/Makefile"):
                    build.build_windows_generation(runner, "95")
                command.assert_not_called()

    def test_generic_gcc_is_not_mistaken_for_mingw(self):
        paths = {"gcc": "gcc.exe"}
        with mock.patch.object(build, "command_path", side_effect=lambda name: paths.get(name)):
            with mock.patch.object(
                build.subprocess, "run",
                return_value=build.subprocess.CompletedProcess(
                    ["gcc.exe", "-dumpmachine"], 0, stdout="x86_64-pc-linux-gnu", stderr=""
                ),
            ):
                self.assertIsNone(build.probe_mingw_gcc())

    def test_existing_mingw_gcc_is_selected_without_installing_visual_studio(self):
        paths = {"gcc": "C:/msys64/mingw64/bin/gcc.exe", "ninja": "C:/tools/ninja.exe"}
        runner = build.Runner(root=Path("D:/WinLook"))
        with mock.patch.object(build, "command_path", side_effect=lambda name: paths.get(name)):
            with mock.patch.object(
                build.subprocess, "run",
                return_value=build.subprocess.CompletedProcess(
                    ["gcc.exe", "-dumpmachine"], 0, stdout="x86_64-w64-mingw32", stderr=""
                ),
            ):
                self.assertTrue(build._select_mingw_cmake(runner))
        self.assertEqual(runner.c_compiler, "mingw")
        self.assertEqual(runner.tool_env["CMAKE_GENERATOR"], "Ninja")
        self.assertEqual(runner.tool_env["CC"], paths["gcc"])

    def test_visual_studio_instance_is_never_reinstalled_through_winget(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        with mock.patch.object(build, "_visual_studio_installations", return_value=[Path("C:/VS/2022/BuildTools")]):
            with mock.patch.object(build, "_vs_installer_path", return_value=None):
                with mock.patch.object(build, "command_path", return_value="winget.exe") as command:
                    with self.assertRaisesRegex(build.BuildError, "Visual Studio Installer could not be located"):
                        build._install_or_modify_visual_studio(runner)
                    command.assert_not_called()

    def test_existing_visual_studio_instance_uses_setup_modify_without_bootstrapper_wait(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        installation = Path("C:/Program Files/Microsoft Visual Studio/2022/Enterprise")
        setup = Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/setup.exe")
        with mock.patch.object(build, "_visual_studio_installations", return_value=[installation]):
            with mock.patch.object(build, "_vs_installer_path", return_value=setup):
                with mock.patch.object(build, "command_path") as command_path:
                    with mock.patch.object(runner, "run") as run:
                        build._install_or_modify_visual_studio(runner)
        command = run.call_args.args[1]
        self.assertEqual(command[0], str(setup))
        self.assertEqual(command[1], "modify")
        self.assertIn("--installPath", command)
        self.assertIn("--add", command)
        self.assertIn("--passive", command)
        self.assertIn("--norestart", command)
        self.assertNotIn("--wait", command)
        command_path.assert_not_called()

    def test_existing_visual_studio_workload_uses_hidden_elevated_setup(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        installation = Path("C:/Program Files/Microsoft Visual Studio/2022/BuildTools")
        setup = Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/setup.exe")
        with mock.patch.object(build, "_visual_studio_installations", return_value=[installation]):
            with mock.patch.object(build, "_vs_installer_path", return_value=setup):
                with mock.patch.object(build, "run_visual_studio_setup") as run_setup:
                    with mock.patch.object(runner, "run") as normal_run:
                        build._install_or_modify_visual_studio(runner)
        normal_run.assert_not_called()
        command = run_setup.call_args.args[2]
        self.assertEqual(run_setup.call_args.args[1], setup)
        self.assertEqual(command[0], "modify")
        self.assertIn("--installPath", command)
        self.assertIn("--add", command)
        self.assertIn("--quiet", command)
        self.assertIn("--norestart", command)
        self.assertNotIn("--passive", command)
        self.assertNotIn("--wait", command)

    def test_visual_studio_setup_failure_includes_exit_reason_and_log_excerpt(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        with mock.patch.object(build, "_run_elevated_windows_command", return_value=5007):
            with mock.patch.object(build, "_recent_visual_studio_setup_logs", return_value="setup log: workload blocked"):
                with mock.patch.object(build.time, "time", return_value=1234.0):
                    with self.assertRaisesRegex(
                        build.BuildError,
                        "5007.*requirements.*setup log: workload blocked",
                    ):
                        build.run_visual_studio_setup(
                            runner, Path("C:/VS/setup.exe"), ["modify", "--quiet"]
                        )

    def test_visual_studio_exit_codes_have_actionable_descriptions(self):
        self.assertIn("Elevation", build.visual_studio_exit_reason(740))
        self.assertIn("already running", build.visual_studio_exit_reason(1618))
        self.assertIn("requirements", build.visual_studio_exit_reason(5007))
        self.assertIn("unspecified failure", build.visual_studio_exit_reason(987654))

    def test_help_colorization_uses_ansi_and_respects_no_color(self):
        rendered = build.colorize_help("usage: winlook [options]\\n  --help  Show help\\n")
        self.assertIn("\x1b[", rendered)
        with mock.patch.dict("os.environ", {"NO_COLOR": "1"}):
            self.assertFalse(build.color_enabled())

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


    def test_existing_global_rustup_195_is_reused_without_download(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        rustup = r"C:\Users\Merge\.cargo\bin\rustup.exe"
        with mock.patch.object(build, "command_path", side_effect=lambda name: rustup if name == "rustup" else None):
            with mock.patch.object(build, "_probe_rustup_toolchain", return_value=(True, "rustc 1.95.0 (59807616e 2026-04-14)")) as probe:
                with mock.patch.object(runner, "run") as run:
                    self.assertTrue(build.ensure_rust_toolchain(runner))
        self.assertEqual(runner.rust_env, {"RUSTUP_TOOLCHAIN": "1.95.0"})
        probe.assert_called_once_with(rustup, "1.95.0")
        run.assert_not_called()
        self.assertFalse(build.rust_paths(runner.root)["bootstrap"].exists())

    def test_existing_global_rustup_default_is_not_modified(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        rustup = r"C:\Users\Merge\.cargo\bin\rustup.exe"
        with mock.patch.dict("os.environ", {"RUSTUP_TOOLCHAIN": "stable"}):
            with mock.patch.object(build, "command_path", side_effect=lambda name: rustup if name == "rustup" else None):
                with mock.patch.object(build, "_probe_rustup_toolchain", return_value=(True, "rustc 1.95.0 (59807616e 2026-04-14)")):
                    self.assertTrue(build.ensure_rust_toolchain(runner))
                    self.assertEqual(build.os.environ["RUSTUP_TOOLCHAIN"], "stable")
        self.assertEqual(runner.rust_env["RUSTUP_TOOLCHAIN"], "1.95.0")

    def test_global_rustup_installs_missing_version_side_by_side(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        rustup = r"C:\Users\Merge\.cargo\bin\rustup.exe"
        with mock.patch.object(build, "command_path", side_effect=lambda name: rustup if name == "rustup" else None):
            with mock.patch.object(build, "_probe_rustup_toolchain", side_effect=[
                (False, "toolchain not installed"),
                (False, "toolchain not installed"),
                (True, "rustc 1.95.0 (59807616e 2026-04-14)"),
            ]):
                with mock.patch.object(runner, "run") as run:
                    self.assertTrue(build.ensure_rust_toolchain(runner))
        self.assertEqual(run.call_args.args[1][:3], [rustup, "toolchain", "install"])
        self.assertEqual(runner.rust_env, {"RUSTUP_TOOLCHAIN": "1.95.0"})

    def test_runner_does_not_force_empty_project_rust_homes_for_global_toolchain(self):
        runner = build.Runner(root=Path("D:/WinLook"), rust_env={"RUSTUP_TOOLCHAIN": "1.95.0"})
        captured = {}

        def fake_run(command, **kwargs):
            captured.update(kwargs)
            return build.subprocess.CompletedProcess(command, 0, stdout="", stderr="")

        clean_env = {
            key: value for key, value in build.os.environ.items()
            if key not in {"CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN"}
        }
        with mock.patch.dict("os.environ", clean_env, clear=True):
            with mock.patch.object(build, "command_path", return_value=None):
                with mock.patch.object(build.subprocess, "run", side_effect=fake_run):
                    runner.run("environment test", ["example-command"])
        child_env = captured["env"]
        self.assertNotIn("CARGO_HOME", child_env)
        self.assertNotIn("RUSTUP_HOME", child_env)
        self.assertEqual(child_env["RUSTUP_TOOLCHAIN"], "1.95.0")

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
