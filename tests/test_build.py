import contextlib
import io
import os
import sys
import tempfile
import time
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

    def test_visual_studio_channel_id_is_read_from_matching_vswhere_instance(self):
        installation = Path("C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools")
        vswhere = Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/vswhere.exe")
        instances = [
            {
                "installationPath": "c:/program files (x86)/microsoft visual studio/2022/buildtools",
                "channelId": "VisualStudio.17.Release",
            },
            {
                "installationPath": "C:/Program Files/Microsoft Visual Studio/2022/Community",
                "channelId": "VisualStudio.17.Preview",
            },
        ]
        completed = build.subprocess.CompletedProcess(
            [str(vswhere)], 0, stdout=build.json.dumps(instances), stderr=""
        )
        with mock.patch.object(build, "_vswhere_path", return_value=vswhere):
            with mock.patch.object(build.subprocess, "run", return_value=completed) as run:
                result = build._visual_studio_channel_id(installation)
        self.assertEqual(result, "VisualStudio.17.Release")
        self.assertEqual(run.call_args.args[0], [str(vswhere), "-all", "-products", "*", "-format", "json"])

    def test_visual_studio_channel_id_refuses_to_guess_when_instance_is_not_found(self):
        installation = Path("D:/Custom/VS/BuildTools")
        vswhere = Path("C:/VS/Installer/vswhere.exe")
        completed = build.subprocess.CompletedProcess(
            [str(vswhere)], 0,
            stdout=build.json.dumps([{"installationPath":"D:/Other/VS/BuildTools","channelId":"VisualStudio.17.Release"}]),
            stderr=""
        )
        with mock.patch.object(build, "_vswhere_path", return_value=vswhere):
            with mock.patch.object(build.subprocess, "run", return_value=completed):
                with self.assertRaisesRegex(build.BuildError, "Refusing to guess a channel ID"):
                    build._visual_studio_channel_id(installation)

    def test_visual_studio_channel_id_requires_metadata_for_matching_instance(self):
        installation = Path("C:/VS/2022/BuildTools")
        vswhere = Path("C:/VS/Installer/vswhere.exe")
        completed = build.subprocess.CompletedProcess(
            [str(vswhere)], 0,
            stdout=build.json.dumps([{"installationPath":"C:/VS/2022/BuildTools"}]),
            stderr=""
        )
        with mock.patch.object(build, "_vswhere_path", return_value=vswhere):
            with mock.patch.object(build.subprocess, "run", return_value=completed):
                with self.assertRaisesRegex(build.BuildError, "did not report its channelId"):
                    build._visual_studio_channel_id(installation)

    def test_existing_visual_studio_instance_uses_setup_modify_without_bootstrapper_wait(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        installation = Path("C:/Program Files/Microsoft Visual Studio/2022/Enterprise")
        setup = Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/setup.exe")
        with mock.patch.object(build, "_visual_studio_installations", return_value=[installation]):
            with mock.patch.object(build, "_vs_installer_path", return_value=setup):
                with mock.patch.object(build, "_visual_studio_channel_id", return_value="VisualStudio.17.Release") as channel:
                    with mock.patch.object(build, "run_visual_studio_setup") as run_setup:
                        with mock.patch.object(runner, "run") as normal_run:
                            build._install_or_modify_visual_studio(runner)
        channel.assert_called_once_with(installation)
        normal_run.assert_not_called()
        command = run_setup.call_args.args[2]
        self.assertEqual(run_setup.call_args.args[1], setup)
        self.assertEqual(command[0], "modify")
        self.assertIn("--installPath", command)
        self.assertEqual(command[command.index("--channelId") + 1], "VisualStudio.17.Release")
        for component_id in (
            "Microsoft.VisualStudio.Workload.VCTools",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "Microsoft.VisualStudio.Component.Windows10SDK.19041",
        ):
            self.assertIn(component_id, command)
        self.assertIn("--quiet", command)
        self.assertIn("--norestart", command)
        self.assertNotIn("--passive", command)
        self.assertNotIn("--wait", command)
    def test_existing_visual_studio_workload_uses_hidden_elevated_setup(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        installation = Path("C:/Program Files/Microsoft Visual Studio/2022/BuildTools")
        setup = Path("C:/Program Files (x86)/Microsoft Visual Studio/Installer/setup.exe")
        with mock.patch.object(build, "_visual_studio_installations", return_value=[installation]):
            with mock.patch.object(build, "_vs_installer_path", return_value=setup):
                with mock.patch.object(build, "_visual_studio_channel_id", return_value="VisualStudio.17.Release"):
                    with mock.patch.object(build, "run_visual_studio_setup") as run_setup:
                        with mock.patch.object(runner, "run") as normal_run:
                            build._install_or_modify_visual_studio(runner)
        normal_run.assert_not_called()
        command = run_setup.call_args.args[2]
        self.assertEqual(run_setup.call_args.args[1], setup)
        self.assertEqual(command[0], "modify")
        self.assertEqual(command[command.index("--channelId") + 1], "VisualStudio.17.Release")
        self.assertIn("Microsoft.VisualStudio.Component.VC.Tools.x86.x64", command)
        self.assertIn("Microsoft.VisualStudio.Component.Windows10SDK.19041", command)
        self.assertIn("--quiet", command)
        self.assertIn("--norestart", command)
        self.assertNotIn("--passive", command)
        self.assertNotIn("--wait", command)
    def test_missing_winget_uses_signed_official_buildtools_bootstrapper(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        bootstrapper = Path("D:/WinLook/build/.rcache/installers/vs_buildtools_2022.exe")
        with mock.patch.object(build, "command_path", return_value=None):
            with mock.patch.object(build, "download_vs_build_tools_bootstrapper", return_value=bootstrapper) as download:
                with mock.patch.object(build, "run_visual_studio_setup") as run_setup:
                    build._install_visual_studio_build_tools(runner)
        download.assert_called_once_with(runner)
        command = run_setup.call_args.args[2]
        self.assertEqual(run_setup.call_args.args[1], bootstrapper)
        self.assertIn("--quiet", command)
        self.assertIn("--wait", command)
        self.assertIn("--norestart", command)
        self.assertIn("--installPath", command)
        self.assertIn("--add", command)
        self.assertIn("Microsoft.VisualStudio.Workload.VCTools", command)
        self.assertIn("Microsoft.VisualStudio.Component.VC.Tools.x86.x64", command)
        self.assertIn("Microsoft.VisualStudio.Component.Windows10SDK.19041", command)
        self.assertIn("--includeRecommended", command)
        self.assertEqual(run_setup.call_args.kwargs["operation"], "Install Visual Studio 2022 C++ Build Tools")

    def test_authenticode_verifier_explicitly_imports_security_module(self):
        with mock.patch.object(build.subprocess, "run") as run:
            run.return_value = build.subprocess.CompletedProcess(
                ["powershell.exe"], 0,
                stdout="CN=Microsoft Corporation, O=Microsoft Corporation",
                stderr="",
            )
            signer = build._verify_microsoft_authenticode(
                Path("C:/Temp/vs_buildtools.exe"), "powershell.exe"
            )
        self.assertIn("Microsoft Corporation", signer)
        command = run.call_args.args[0]
        script = command[-1]
        self.assertIn("Import-Module Microsoft.PowerShell.Security -Force", script)
        self.assertIn("Get-AuthenticodeSignature", script)
        self.assertEqual(run.call_args.kwargs["env"]["WINLOOK_AUTHENTICODE_FILE"], str(Path("C:/Temp/vs_buildtools.exe")))

    def test_buildtools_bootstrapper_cache_is_reused_after_authenticode_check(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            runner = build.Runner(root=root)
            cached = build.build_paths(root)["rust_cache"] / "installers" / "vs_buildtools_2022.exe"
            cached.parent.mkdir(parents=True)
            cached.write_bytes(b"cached signed bootstrapper")
            with mock.patch.object(build, "_powershell_executable", return_value="powershell.exe"):
                with mock.patch.object(
                    build, "_verify_microsoft_authenticode",
                    return_value="CN=Microsoft Corporation, O=Microsoft Corporation",
                ) as verify:
                    with mock.patch.object(build.urllib.request, "urlopen") as urlopen:
                        result = build.download_vs_build_tools_bootstrapper(runner)
            self.assertEqual(result, cached)
            verify.assert_called_once_with(cached, "powershell.exe")
            urlopen.assert_not_called()

    def test_buildtools_bootstrapper_download_uses_official_url_and_verifies_signature(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            runner = build.Runner(root=root)
            payload = b"microsoft bootstrapper" * 4096
            response = io.BytesIO(payload)
            with mock.patch.object(build, "_powershell_executable", return_value="powershell.exe"):
                with mock.patch.object(
                    build, "_verify_microsoft_authenticode",
                    return_value="CN=Microsoft Corporation, O=Microsoft Corporation",
                ) as verify:
                    with mock.patch.object(
                        build.urllib.request, "urlopen", return_value=response
                    ) as urlopen:
                        result = build.download_vs_build_tools_bootstrapper(runner)
            self.assertTrue(result.is_file())
            self.assertEqual(result.read_bytes(), payload)
            self.assertEqual(
                urlopen.call_args.args[0].full_url,
                "https://aka.ms/vs/17/release/vs_buildtools.exe",
            )
            verify.assert_called_once()
            self.assertNotIn(".download", result.name)

    def test_visual_studio_setup_failure_includes_exit_reason_and_log_excerpt(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        with mock.patch.object(build, "_run_elevated_windows_command", return_value=5007):
            with mock.patch.object(build, "_recent_visual_studio_setup_logs", return_value="setup log: workload blocked"):
                with mock.patch.object(build.time, "time", return_value=1234.0):
                    with self.assertRaisesRegex(
                        build.BuildError,
                        r"(?s)5007.*requirement.*setup log: workload blocked",
                    ):
                        build.run_visual_studio_setup(
                            runner, Path("C:/VS/setup.exe"), ["modify", "--quiet"]
                        )

    def test_visual_studio_exit_codes_have_actionable_descriptions(self):
        self.assertIn("Elevation", build.visual_studio_exit_reason(740))
        self.assertIn("already running", build.visual_studio_exit_reason(1618))
        self.assertIn("requirement", build.visual_studio_exit_reason(5007))
        self.assertIn("unspecified failure", build.visual_studio_exit_reason(987654))

    def test_help_colorization_uses_ansi_and_respects_no_color(self):
        rendered = build.colorize_help("usage: winlook [options]\n  --help  Show help\n")
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
                with mock.patch.object(build, "_installed_rust_components", return_value={"rustfmt", "clippy"}):
                    with mock.patch.object(runner, "run") as run:
                        self.assertTrue(build.ensure_rust_toolchain(runner))
        self.assertEqual(runner.rust_env, {"RUSTUP_TOOLCHAIN": "1.95.0"})
        probe.assert_called_once_with(rustup, "1.95.0")
        run.assert_not_called()
        self.assertFalse(build.rust_paths(runner.root)["bootstrap"].exists())

    def test_existing_global_rustup_adds_missing_rustfmt_and_clippy(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        rustup = r"C:\Users\Merge\.cargo\bin\rustup.exe"
        with mock.patch.object(build, "command_path", side_effect=lambda name: rustup if name == "rustup" else None):
            with mock.patch.object(build, "_probe_rustup_toolchain", return_value=(True, "rustc 1.95.0 (59807616e 2026-04-14)")):
                with mock.patch.object(build, "_installed_rust_components", return_value=set()):
                    with mock.patch.object(runner, "run") as run:
                        self.assertTrue(build.ensure_rust_toolchain(runner))
        self.assertEqual(
            run.call_args.args[1],
            [rustup, "component", "add", "--toolchain", "1.95.0", "rustfmt", "clippy"],
        )
        self.assertEqual(runner.rust_env, {"RUSTUP_TOOLCHAIN": "1.95.0"})

    def test_existing_global_rustup_default_is_not_modified(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        rustup = r"C:\Users\Merge\.cargo\bin\rustup.exe"
        with mock.patch.dict("os.environ", {"RUSTUP_TOOLCHAIN": "stable"}):
            with mock.patch.object(build, "command_path", side_effect=lambda name: rustup if name == "rustup" else None):
                with mock.patch.object(build, "_probe_rustup_toolchain", return_value=(True, "rustc 1.95.0 (59807616e 2026-04-14)")):
                    with mock.patch.object(build, "_installed_rust_components", return_value={"rustfmt", "clippy"}):
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
                with mock.patch.object(build, "_installed_rust_components", return_value={"rustfmt", "clippy"}):
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

    def test_command_path_falls_back_to_explicit_exe_on_windows(self):
        with mock.patch.object(build.os, "name", "nt"):
            with mock.patch.object(
                build.shutil, "which",
                side_effect=lambda name: "C:/tools/" + name if name == "cl.exe" else None,
            ) as which:
                self.assertEqual(build.command_path("cl"), "C:/tools/cl.exe")
        self.assertEqual([call.args[0] for call in which.call_args_list], ["cl", "cl.exe"])

    @staticmethod
    def _create_windows_sdk_fixture(root: Path, version: str) -> None:
        files = (
            Path("Include") / version / "um" / "Windows.h",
            Path("Include") / version / "shared" / "sdkddkver.h",
            Path("Include") / version / "ucrt" / "stdio.h",
            Path("Lib") / version / "um" / "x64" / "kernel32.lib",
            Path("Lib") / version / "ucrt" / "x64" / "ucrt.lib",
        )
        for relative in files:
            destination = root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(b"Windows SDK test fixture")

    def test_windows_batch_environment_command_uses_comspec_and_keeps_arguments_separate(self):
        script = Path(
            r"C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools"
            r"\\Common7\\Tools\\VsDevCmd.bat"
        )
        with mock.patch.dict(
            "os.environ",
            {"ComSpec": r"C:\\Windows\\System32\\cmd.exe"},
            clear=True,
        ):
            command = build._windows_batch_environment_command(
                script,
                ("-no_logo", "-arch=x64", "-host_arch=x64"),
            )

        self.assertEqual(
            command,
            [
                r"C:\\Windows\\System32\\cmd.exe",
                "/d",
                "/c",
                "call",
                str(script),
                "-no_logo",
                "-arch=x64",
                "-host_arch=x64",
                "&&",
                "set",
            ],
        )
        rendered = build.subprocess.list2cmdline(command)
        self.assertIn('call "' + str(script) + '"', rendered)
        self.assertTrue(rendered.endswith("&& set"))
        self.assertNotIn(r'\\"', rendered)

    @unittest.skipUnless(os.name == "nt", "requires the Windows command processor")
    def test_windows_batch_environment_command_executes_a_path_with_spaces(self):
        with tempfile.TemporaryDirectory() as temp:
            script_dir = Path(temp) / "WinLook Batch Runner Fixture"
            script_dir.mkdir()
            script = script_dir / "emit-environment.cmd"
            script.write_bytes(
                b"@echo off\\r\\n"
                b"set WINLOOK_BATCH_ENVIRONMENT_TEST=ready\\r\\n"
            )
            result = build.run_subprocess(
                build._windows_batch_environment_command(script),
                text=True,
                capture_output=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, result.stderr or result.stdout)
        self.assertIn("WINLOOK_BATCH_ENVIRONMENT_TEST=ready", result.stdout or "")

    def test_msvc_environment_probe_uses_explicit_exe_names(self):
        observed = []

        def which(name, path=None):
            observed.append((name, path))
            return "C:/VS/bin/" + name

        with mock.patch.object(build.shutil, "which", side_effect=which):
            with mock.patch.object(
                build, "_windows_sdk_installation",
                return_value=(Path("C:/Windows Kits/10"), "10.0.26100.0"),
            ):
                self.assertTrue(build._msvc_environment_tools({"PATH": "C:/VS/bin"}))

        self.assertEqual(
            observed,
            [
                ("cl.exe", "C:/VS/bin"),
                ("link.exe", "C:/VS/bin"),
                ("rc.exe", "C:/VS/bin"),
            ],
        )

    def test_windows_sdk_probe_accepts_case_insensitive_environment_keys(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            version = "10.0.26100.0"
            self._create_windows_sdk_fixture(root, version)
            self.assertTrue(
                build._windows_sdk_is_present(
                    {
                        "WINDOWSSDKDIR": str(root) + os.sep,
                        "WINDOWSSDKVERSION": version + os.sep,
                    }
                )
            )

    def test_windows_sdk_environment_fallback_populates_compiler_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            program_files = Path(temp) / "Program Files (x86)"
            root = program_files / "Windows Kits" / "10"
            version = "10.0.26100.0"
            self._create_windows_sdk_fixture(root, version)
            sdk_bin = root / "bin" / version / "x64"
            sdk_bin.mkdir(parents=True)
            sdk_include = root / "Include" / version
            sdk_lib = root / "Lib" / version

            with mock.patch.object(build, "_windows_sdk_registry_roots", return_value=[]):
                environment = build._prepare_windows_sdk_environment(
                    {
                        "ProgramFiles(x86)": str(program_files),
                        "PATH": "C:/VS/VC/bin",
                        "INCLUDE": "C:/VS/VC/include",
                        "LIB": "C:/VS/VC/lib/x64",
                    }
                )

        self.assertEqual(Path(environment["WindowsSdkDir"]), root)
        self.assertEqual(environment["WindowsSDKVersion"], version + os.sep)
        self.assertIn(str(sdk_bin), environment["PATH"])
        self.assertIn(str(sdk_include / "um"), environment["INCLUDE"])
        self.assertIn(str(sdk_include / "ucrt"), environment["INCLUDE"])
        self.assertIn(str(sdk_lib / "um" / "x64"), environment["LIB"])
        self.assertIn(str(sdk_lib / "ucrt" / "x64"), environment["LIB"])
        self.assertIn(str(sdk_lib / "um" / "x64"), environment["LIBPATH"])

    def test_msvc_environment_status_reports_each_missing_tool_separately(self):
        with mock.patch.object(build, "_windows_sdk_installation", return_value=None):
            with mock.patch.object(build.shutil, "which", return_value=None):
                status = build._msvc_environment_status({"PATH": ""})
        self.assertEqual(
            build._msvc_environment_missing(status),
            ["cl.exe", "link.exe", "rc.exe", "Windows SDK headers/libraries"],
        )

    def test_strict_doctor_requires_complete_msvc_for_modern_windows_apps(self):
        checks = {
            "win10_project": True,
            "win11_project": True,
            "msvc_available": False,
        }
        self.assertTrue(build._windows_msvc_requirement_missing(checks, windows_host=True))

        checks["msvc_available"] = True
        self.assertFalse(build._windows_msvc_requirement_missing(checks, windows_host=True))
        self.assertFalse(build._windows_msvc_requirement_missing(checks, windows_host=False))

        checks["win10_project"] = False
        checks["win11_project"] = False
        checks["msvc_available"] = False
        self.assertFalse(build._windows_msvc_requirement_missing(checks, windows_host=True))

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


    def test_failed_existing_visual_studio_modify_uses_one_buildtools_fallback(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        enterprise = Path("C:/Program Files/Microsoft Visual Studio/2022/Enterprise")
        with mock.patch.object(build.os, "name", "nt"):
            with mock.patch.object(build, "load_msvc_environment", side_effect=[False, True]):
                with mock.patch.object(build, "probe_mingw_gcc", return_value=None):
                    with mock.patch.object(build, "_visual_studio_installations", return_value=[enterprise]):
                        with mock.patch.object(
                            build, "_install_or_modify_visual_studio",
                            side_effect=build.BuildError("existing instance modify failed"),
                        ):
                            with mock.patch.object(build, "_install_visual_studio_build_tools") as install:
                                build.ensure_windows_linker(runner, apply=True)
        install.assert_called_once_with(runner)

    def test_failed_initial_buildtools_install_is_not_retried(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        with mock.patch.object(build.os, "name", "nt"):
            with mock.patch.object(build, "load_msvc_environment", return_value=False):
                with mock.patch.object(build, "probe_mingw_gcc", return_value=None):
                    with mock.patch.object(build, "_visual_studio_installations", return_value=[]):
                        with mock.patch.object(
                            build, "_install_visual_studio_build_tools",
                            side_effect=build.BuildError("WinGet installation failed"),
                        ) as install:
                            with self.assertRaisesRegex(build.BuildError, "WinGet installation failed"):
                                build.ensure_windows_linker(runner, apply=True)
        install.assert_called_once_with(runner)

    def test_existing_buildtools_failure_does_not_trigger_second_install(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        buildtools = Path("C:/Program Files/Microsoft Visual Studio/2022/BuildTools")
        with mock.patch.object(build.os, "name", "nt"):
            with mock.patch.object(build, "load_msvc_environment", return_value=False):
                with mock.patch.object(build, "probe_mingw_gcc", return_value=None):
                    with mock.patch.object(build, "_visual_studio_installations", return_value=[buildtools]):
                        with mock.patch.object(
                            build, "_install_or_modify_visual_studio",
                            side_effect=build.BuildError("existing Build Tools modify failed"),
                        ):
                            with mock.patch.object(build, "_install_visual_studio_build_tools") as install:
                                with self.assertRaisesRegex(build.BuildError, "existing Build Tools modify failed"):
                                    build.ensure_windows_linker(runner, apply=True)
        install.assert_not_called()

    def test_uac_decline_does_not_fall_back_to_a_second_installer(self):
        runner = build.Runner(root=Path("D:/WinLook"))
        enterprise = Path("C:/Program Files/Microsoft Visual Studio/2022/Enterprise")
        with mock.patch.object(build.os, "name", "nt"):
            with mock.patch.object(build, "load_msvc_environment", return_value=False):
                with mock.patch.object(build, "probe_mingw_gcc", return_value=None):
                    with mock.patch.object(build, "_visual_studio_installations", return_value=[enterprise]):
                        with mock.patch.object(
                            build, "_install_or_modify_visual_studio",
                            side_effect=build.BuildError("Windows elevation was declined"),
                        ):
                            with mock.patch.object(build, "_install_visual_studio_build_tools") as install:
                                with self.assertRaisesRegex(build.BuildError, "elevation was declined"):
                                    build.ensure_windows_linker(runner, apply=True)
        install.assert_not_called()

    def test_debug_flags_are_removed_and_accepted_after_subcommands(self):
        cleaned, debug, all_debug = build._extract_debug_flags(
            ["build", "win10", "--debug", "--root", "D:/WinLook"]
        )
        self.assertEqual(cleaned, ["build", "win10", "--root", "D:/WinLook"])
        self.assertTrue(debug)
        self.assertFalse(all_debug)
        cleaned, debug, all_debug = build._extract_debug_flags(
            ["--all-debug", "build", "win11"]
        )
        self.assertEqual(cleaned, ["build", "win11"])
        self.assertFalse(debug)
        self.assertTrue(all_debug)
        self.assertTrue(build.build_parser().parse_args(["--debug", "doctor"]).debug)

    def test_elevated_wait_polls_and_invokes_progress_callback(self):
        class FakeKernel:
            def __init__(self):
                self.results = [258, 258, 0]
                self.timeouts = []

            def WaitForSingleObject(self, handle, timeout):
                self.timeouts.append(timeout)
                return self.results.pop(0)

        kernel = FakeKernel()
        progress = []
        build._wait_for_windows_process(
            kernel, 123, on_wait=lambda: progress.append("tick"), poll_interval_ms=750
        )
        self.assertEqual(progress, ["tick", "tick"])
        self.assertEqual(kernel.timeouts, [750, 750, 750])

    def test_visual_studio_log_tailer_reads_only_new_appended_lines(self):
        with tempfile.TemporaryDirectory() as temp:
            log = Path(temp) / "dd_setup_test.log"
            log.write_text("[1234:0001][2026-11-12T12:00:00] Installing package Microsoft.VC.Tools\n", encoding="utf-8")
            with mock.patch.object(build.tempfile, "gettempdir", return_value=temp):
                tailer = build.VisualStudioSetupLogTailer(time.time() - 1)
                first = tailer.read_new_lines()
                self.assertEqual(len(first), 1)
                self.assertIn("Installing package", first[0][1])
                with log.open("a", encoding="utf-8") as output:
                    output.write("[1234:0001][2026-11-12T12:00:01] Downloading package Microsoft.Windows.SDK\n")
                second = tailer.read_new_lines()
        self.assertEqual(len(second), 1)
        self.assertIn("Downloading package", second[0][1])

    def test_visual_studio_activity_classifies_package_operations(self):
        self.assertTrue(
            build.visual_studio_activity(
                "[1234:0001][2026-11-12T12:00:00] Installing package Microsoft.VisualStudio.Component.VC.Tools"
            ).startswith("INSTALL:")
        )
        self.assertTrue(
            build.visual_studio_activity(
                "[1234:0001][2026-11-12T12:00:00] Downloading package Microsoft.Windows.SDK"
            ).startswith("DOWNLOAD:")
        )
        self.assertTrue(
            build.visual_studio_activity(
                "[1234:0001][2026-11-12T12:00:00] Error: package signature check failed"
            ).startswith("ERROR:")
        )
        self.assertIsNone(build.visual_studio_activity("Installer telemetry heartbeat"))

    def test_visual_studio_debug_mode_streams_live_package_events(self):
        with tempfile.TemporaryDirectory() as temp:
            log = Path(temp) / "dd_setup_test.log"
            log.write_text(
                "[1234:0001][2026-11-12T12:00:00] Installing package Microsoft.VisualStudio.Component.VC.Tools\n",
                encoding="utf-8",
            )
            runner = build.Runner(root=Path(temp), debug=True)
            output = io.StringIO()

            def fake_elevated(command, cwd, on_wait=None):
                if on_wait:
                    on_wait()
                return 0

            with mock.patch.object(build.tempfile, "gettempdir", return_value=temp):
                with mock.patch.object(build, "_run_elevated_windows_command", side_effect=fake_elevated):
                    with contextlib.redirect_stdout(output):
                        build.run_visual_studio_setup(
                            runner, Path("C:/VS/setup.exe"), ["modify", "--quiet"]
                        )
            rendered = output.getvalue()
        self.assertIn("INSTALL:", rendered)
        self.assertIn("Microsoft.VisualStudio.Component.VC.Tools", rendered)
        self.assertIn("validating the resulting toolchain", rendered)

    def test_runner_suppresses_successful_subprocess_chatter_by_default(self):
        runner = build.Runner(root=Path("."), debug=False)
        result = build.subprocess.CompletedProcess(
            ["example-tool"], 0, stdout="verbose compiler output", stderr=""
        )
        output = io.StringIO()
        with mock.patch.object(build, "command_path", return_value=None):
            with mock.patch.object(build.subprocess, "run", return_value=result) as run:
                with contextlib.redirect_stdout(output):
                    runner.run("Quiet check", ["example-tool"])
        self.assertNotIn("verbose compiler output", output.getvalue())
        self.assertNotIn("$ example-tool", output.getvalue())
        self.assertTrue(run.call_args.kwargs["capture_output"])

    def test_runner_all_debug_replays_subprocess_output(self):
        runner = build.Runner(root=Path("."), debug=True, all_debug=True)
        result = build.subprocess.CompletedProcess(
            ["example-tool"], 0, stdout="full compiler output\n", stderr=""
        )
        output = io.StringIO()
        with mock.patch.object(build, "command_path", return_value=None):
            with mock.patch.object(build.subprocess, "run", return_value=result):
                with contextlib.redirect_stdout(output):
                    runner.run("Detailed check", ["example-tool"])
        self.assertIn("$ example-tool", output.getvalue())
        self.assertIn("full compiler output", output.getvalue())

    def test_all_debug_records_direct_subprocess_stdout_and_stderr(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result = build.subprocess.CompletedProcess(
                ["probe-tool", "--version"], 0,
                stdout="probe version details\n", stderr="probe diagnostic details\n"
            )
            with mock.patch.object(build.subprocess, "run", return_value=result):
                build.run_with_all_debug_log(
                    root,
                    lambda: (build.run_subprocess(
                        ["probe-tool", "--version"], text=True, capture_output=True
                    ), 0)[1],
                )
            log = (root / "debug.log").read_text(encoding="utf-8")
        self.assertIn("[SUBPROCESS] probe-tool --version (exit 0)", log)
        self.assertIn("probe version details", log)
        self.assertIn("probe diagnostic details", log)

    def test_all_debug_writes_a_complete_session_log(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            console = io.StringIO()
            with mock.patch("sys.stdout", console):
                result = build.run_with_all_debug_log(
                    root, lambda: (print("diagnostic marker"), 7)[1]
                )
            log = (root / "debug.log").read_text(encoding="utf-8")
        self.assertEqual(result, 7)
        self.assertIn("diagnostic marker", console.getvalue())
        self.assertIn("diagnostic marker", log)
        self.assertIn("finished with exit code 7", log)

    def test_windows_sdk_probe_uses_standard_install_root_without_sdk_environment_variables(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "Windows Kits" / "10"
            version = "10.0.26100.0"
            self._create_windows_sdk_fixture(root, version)
            environment = {"ProgramFiles(x86)": temp}
            with mock.patch.object(build, "_windows_sdk_registry_roots", return_value=[]):
                self.assertEqual(
                    build._windows_sdk_installation(environment),
                    (root, version),
                )

    def test_windows_sdk_probe_uses_registry_root_when_environment_is_missing(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "KitsRoot10"
            version = "10.0.19041.0"
            self._create_windows_sdk_fixture(root, version)
            with mock.patch.object(build, "_windows_sdk_registry_roots", return_value=[root]):
                self.assertEqual(
                    build._windows_sdk_installation({}),
                    (root, version),
                )

    def test_windows_sdk_completion_fills_sdk_variables_and_resource_compiler_path(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "Windows Kits" / "10"
            version = "10.0.26100.0"
            for folder in ("ucrt", "shared", "um", "winrt", "cppwinrt"):
                (root / "Include" / version / folder).mkdir(parents=True)
            for folder in ("ucrt", "um"):
                (root / "Lib" / version / folder / "x64").mkdir(parents=True)
            sdk_bin = root / "bin" / version / "x64"
            sdk_bin.mkdir(parents=True)
            (sdk_bin / "rc.exe").write_bytes(b"resource compiler")
            self._create_windows_sdk_fixture(root, version)

            completed, sdk = build._complete_msvc_environment(
                {
                    "Path": "C:/VisualStudio/VC/bin",
                    "WindowsSdkDir": str(root),
                    "WindowsSDKVersion": version + os.sep,
                    "ProgramFiles(x86)": temp,
                }
            )
        self.assertEqual(sdk, (root, version))
        self.assertEqual(completed["WindowsSdkDir"], str(root) + os.sep)
        self.assertEqual(completed["WindowsSDKVersion"], version + os.sep)
        self.assertIn(str(root / "Include" / version / "um"), completed["INCLUDE"])
        self.assertIn(str(root / "Lib" / version / "um" / "x64"), completed["LIB"])
        self.assertIn(str(sdk_bin), completed["PATH"])

    def test_msvc_toolset_is_discovered_from_visual_studio_instance_when_env_is_missing(self):
        with tempfile.TemporaryDirectory() as temp:
            installation = Path(temp) / "BuildTools"
            toolset = installation / "VC" / "Tools" / "MSVC" / "14.44.35207"
            (toolset / "include").mkdir(parents=True)
            (toolset / "lib" / "x64").mkdir(parents=True)
            binary = toolset / "bin" / "Hostx64" / "x64"
            binary.mkdir(parents=True)
            (binary / "cl.exe").write_bytes(b"compiler")
            (binary / "link.exe").write_bytes(b"linker")
            self.assertEqual(build._msvc_toolset_root({}, installation), toolset)

    def test_visual_studio_setup_diagnostics_detect_no_update_without_component_install(self):
        with tempfile.TemporaryDirectory() as temp:
            log = Path(temp) / "dd_setup_noop.log"
            log.write_text(
                "Warning: Didn't find any channel feed.\n"
                "WebClient error RequestCanceled https://aka.ms/vs/17/release/channel\n"
                "Status changed to NoUpdate\n"
                "Closing the installer with exit code 0\n",
                encoding="utf-8",
            )
            with mock.patch.object(build.tempfile, "gettempdir", return_value=temp):
                notes = build._visual_studio_setup_log_diagnostics(time.time() - 1)
        self.assertTrue(any("channel feed" in note for note in notes))
        self.assertTrue(any("NoUpdate" in note for note in notes))

    def test_visual_studio_setup_diagnostics_do_not_flag_a_package_install(self):
        with tempfile.TemporaryDirectory() as temp:
            log = Path(temp) / "dd_setup_install.log"
            log.write_text(
                "Status changed to Applying\n"
                "Downloading package Microsoft.VisualStudio.Component.VC.Tools.x86.x64\n"
                "Installing package Microsoft.VisualStudio.Component.Windows10SDK.19041\n",
                encoding="utf-8",
            )
            with mock.patch.object(build.tempfile, "gettempdir", return_value=temp):
                notes = build._visual_studio_setup_log_diagnostics(time.time() - 1)
        self.assertEqual(notes, [])



if __name__ == "__main__":
    unittest.main()
