"""Invocation-root and build-policy checks without Cargo, network or commits."""

import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
from card_tool_runtime import build_env, workspace_root
import worktrees


class WorktreeRuntime(unittest.TestCase):
    def test_pool_warm_uses_the_selected_slots_private_target(self):
        with tempfile.TemporaryDirectory() as folder:
            pool = worktrees.Pool.__new__(worktrees.Pool)
            with mock.patch.dict(os.environ, {"CARGO_TARGET_DIR": "borrowed-target", "CANTRIP_CARDS_ONLY": "stale"}), \
                    mock.patch.object(worktrees.subprocess, "run", return_value=mock.Mock(returncode=0)) as cargo:
                pool._cargo_warm(folder)
            self.assertEqual(Path(cargo.call_args.kwargs["env"]["CARGO_TARGET_DIR"]), Path(folder).resolve() / "target")
            self.assertEqual(cargo.call_args.kwargs["cwd"], folder)
            self.assertNotIn("CANTRIP_CARDS_ONLY", cargo.call_args.kwargs["env"])
            self.assertNotIn("cantrip-script", cargo.call_args.args[0])

    def test_outside_git_falls_back_to_the_external_tools_repository(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            lane = root / "lane"
            tool = lane / "tools" / "maintenance" / "card-coverage.py"
            tool.parent.mkdir(parents=True)
            tool.touch()
            outside = root / "outside"
            outside.mkdir()
            self.assertEqual(Path(workspace_root(tool, cwd=outside)), lane.resolve())

    def test_old_profile_and_inherited_environment_cannot_disable_mutation_assertions(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            manifest = root / "Cargo.toml"
            original = '[profile.mutate]\ninherits = "cards"\ndebug-assertions = false\n'
            manifest.write_text(original, encoding="utf-8")
            with mock.patch.dict(os.environ, {"CARGO_PROFILE_MUTATE_DEBUG_ASSERTIONS": "false", "CANTRIP_VERIFY": "0",
                                              "CARGO_TARGET_DIR": "another-worktrees-target", "CANTRIP_CARDS_ONLY": "stale-list"}):
                actual = build_env(root)
            self.assertEqual(actual["CARGO_PROFILE_MUTATE_DEBUG_ASSERTIONS"], "true")
            self.assertEqual(actual["CANTRIP_VERIFY"], "1")
            self.assertEqual(Path(actual["CARGO_TARGET_DIR"]), root.resolve() / "target")
            self.assertNotIn("CANTRIP_CARDS_ONLY", actual)
            self.assertEqual(manifest.read_text(encoding="utf-8"), original)


if __name__ == "__main__":
    unittest.main()
