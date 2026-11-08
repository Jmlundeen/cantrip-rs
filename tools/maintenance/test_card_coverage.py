"""Coverage isolation and artifact reuse without Cargo or LLVM execution."""

import contextlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
spec = importlib.util.spec_from_file_location("card_coverage", HERE / "card-coverage.py")
coverage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coverage)


class CoverageArtifacts(unittest.TestCase):
    def test_external_coverage_script_selects_callers_git_worktree_from_nested_cwd(self):
        script = "\n".join([
            "import importlib.util, json, sys", f"sys.path.insert(0, {str(HERE)!r})",
            f"spec = importlib.util.spec_from_file_location('external_coverage', {str(HERE / 'card-coverage.py')!r})",
            "coverage = importlib.util.module_from_spec(spec)", "spec.loader.exec_module(coverage)",
            "print(json.dumps({'root': coverage.ROOT, 'card': coverage.card_file('Pinned Card'),",
            "                  'target': coverage.build_env(coverage.ROOT)['CARGO_TARGET_DIR']}))",
        ])
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder) / "pinned"
            root.mkdir()
            subprocess.run(["git", "init", "--quiet", str(root)], check=True, capture_output=True)
            card = root / "crates" / "cantrip-cards" / "src" / "cards" / "pinned_card.rs"
            card.parent.mkdir(parents=True)
            card.write_text("fn pinned_card() {}", encoding="utf-8")
            child = subprocess.run([sys.executable, "-B", "-c", script], cwd=card.parent, capture_output=True, text=True)
            self.assertEqual(child.returncode, 0, child.stderr)
            actual = json.loads(child.stdout)
            self.assertEqual(Path(actual["root"]), root.resolve())
            self.assertEqual(Path(actual["card"]), card.resolve())
            self.assertEqual(Path(actual["target"]), root.resolve() / "target")

    def test_independent_process_cannot_enter_an_active_worktree_build_lock(self):
        script = "\n".join([
            "import sys", f"sys.path.insert(0, {str(HERE)!r})", "from card_tool_runtime import build_lock",
            "try:", "    with build_lock(sys.argv[1], timeout=0.05):", "        sys.exit(3)",
            "except TimeoutError:", "    sys.exit(0)",
        ])
        with tempfile.TemporaryDirectory() as root:
            with coverage.build_lock(root):
                blocked = subprocess.run([sys.executable, "-B", "-c", script, root], capture_output=True, text=True)
                separate = subprocess.run([sys.executable, "-B", "-c", script, str(Path(root) / "other-worktree")],
                                          capture_output=True, text=True)
            acquired = subprocess.run([sys.executable, "-B", "-c", script, root], capture_output=True, text=True)
        self.assertEqual(blocked.returncode, 0, blocked.stderr)
        self.assertIn("waiting for the other card tool", blocked.stdout)
        self.assertEqual(separate.returncode, 3, separate.stderr)
        self.assertEqual(acquired.returncode, 3, acquired.stderr)

    def test_two_runs_keep_compiled_artifacts_and_merge_only_their_own_raw_profiles(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            cov = root / "target" / "llvm-cov-target"
            ext = ".exe" if os.name == "nt" else ""
            binary = cov / "cards" / ("corpus" + ext)
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"compatible instrumented binary")
            tools = root / "sysroot" / "lib" / "rustlib" / "test-host" / "bin"
            tools.mkdir(parents=True)
            for tool in ("llvm-profdata", "llvm-cov"):
                (tools / (tool + ext)).touch()
            (cov / "stale.profraw").write_text("stale")
            merged, identities, commands = [], [], []

            def run(args, **kwargs):
                env = kwargs["env"]
                self.assertEqual(Path(env["CARGO_TARGET_DIR"]), root / "target")
                self.assertEqual(env["CANTRIP_VERIFY"], "1")
                self.assertNotIn("CANTRIP_CARDS_ONLY", env)
                commands.append(args)
                out = ""
                if args[:3] == ["cargo", "llvm-cov", "clean"]:
                    # A real workspace clean would invalidate the saved binary.
                    if "--profraw-only" not in args:
                        binary.unlink()
                    for raw in cov.glob("*.profraw"):
                        raw.unlink()
                elif args[:3] == ["cargo", "llvm-cov", "run"]:
                    self.assertIn("--no-report", args)
                    self.assertNotIn("--no-clean", args)  # --no-report implies it; the explicit pair is invalid.
                    self.assertEqual(args[args.index("--profile") + 1], "cards")
                    identity = env["LLVM_PROFILE_FILE_NAME"]
                    identities.append(identity)
                    (cov / identity.replace("%p", "123").replace("%m", "456")).write_text("fresh")
                    # A foreign profile appearing after cleanup must also be excluded.
                    (cov / "foreign.profraw").write_text("foreign")
                    out = "1 scenarios: 1 pass, 0 fail, 0 known bugs, 0 need cards the engine does not have.\n"
                elif args == ["rustc", "-vV"]:
                    out = "host: test-host\n"
                elif args == ["rustc", "--print", "sysroot"]:
                    out = str(root / "sysroot")
                elif "merge" in args:
                    raw = [Path(arg) for arg in args if arg.endswith(".profraw")]
                    self.assertEqual(len(raw), 1)
                    self.assertEqual(raw[0].read_text(), "fresh")
                    merged.extend(raw)
                elif "export" in args:
                    kwargs["stdout"].write("SF:alpha.rs\nDA:1,1\nend_of_record\n")
                else:
                    self.fail(f"unexpected command {args}")
                return subprocess.CompletedProcess(args, 0, out, "")

            with mock.patch.object(coverage, "ROOT", str(root)), mock.patch.object(coverage.subprocess, "run", side_effect=run), \
                 mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "shared-target", "CANTRIP_CARDS_ONLY": "unrelated-list"}):
                for _ in range(2):
                    with coverage.build_lock(root):
                        summary, lcov = coverage.coverage_lcov("Alpha")
                    self.assertIn("1 pass", summary[-1])
                    self.assertIn("DA:1,1", lcov)
            self.assertEqual(binary.read_bytes(), b"compatible instrumented binary")
            self.assertNotEqual(identities[0], identities[1])
            self.assertEqual(len(merged), 2)


if __name__ == "__main__":
    unittest.main()
