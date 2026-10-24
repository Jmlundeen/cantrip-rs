"""Tests of the worktree pool (worktrees.py), each against a throwaway origin and clone:

    python -m unittest tools/maintenance/test_worktrees.py
"""

import datetime
import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("worktrees", os.path.join(HERE, "worktrees.py"))
wt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wt)

GIT_ENV = {"GIT_AUTHOR_NAME": "Test", "GIT_AUTHOR_EMAIL": "test@example.com", "GIT_COMMITTER_NAME": "Test",
           "GIT_COMMITTER_EMAIL": "test@example.com"}


def run(cwd, *args):
    return subprocess.run(["git", *args], cwd=cwd, check=True, capture_output=True, text=True).stdout.strip()


class PoolTest(unittest.TestCase):
    def setUp(self):
        self.saved_env = {k: os.environ.get(k) for k in GIT_ENV}
        os.environ.update(GIT_ENV)
        self.tmp = tempfile.mkdtemp(prefix="pool-test-")
        origin = os.path.join(self.tmp, "origin.git")
        run(self.tmp, "init", "--quiet", "--bare", "-b", "main", origin)
        self.main = os.path.join(self.tmp, "main")
        run(self.tmp, "clone", "--quiet", origin, self.main)
        run(self.main, "checkout", "--quiet", "-b", "main")
        self.write(self.main, ".gitignore", ".claude/worktrees/\ntarget/\n")
        self.commit(self.main, "first")
        run(self.main, "push", "--quiet", "-u", "origin", "main")
        self.clock = [datetime.datetime(2026, 10, 23, 12, 0, tzinfo=datetime.timezone.utc)]
        self.pool = wt.Pool(root=self.main, now=lambda: self.clock[0], warm=lambda path: None,
                            hooks_ready=lambda: True)

    def tearDown(self):
        for k, v in self.saved_env.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        shutil.rmtree(self.tmp, onerror=lambda f, p, e: (os.chmod(p, 0o700), f(p)))

    def write(self, where, name, text):
        with open(os.path.join(where, name), "w", encoding="utf-8") as f:
            f.write(text)

    def commit(self, where, message):
        run(where, "add", "-A")
        run(where, "commit", "--quiet", "-m", message)
        return run(where, "rev-parse", "HEAD")

    def later(self, **delta):
        self.clock[0] += datetime.timedelta(**delta)

    def lease_id(self, text):
        return next(line.split()[1] for line in text.splitlines() if line.startswith("LEASE "))

    def token_of(self, text):
        return next(line.split()[1] for line in text.splitlines() if line.startswith("TOKEN "))

    def member_id(self, text):
        return next(line.split()[1] for line in text.splitlines() if line.startswith("MEMBER "))

    def granted(self, purpose="card: Test"):
        return self.lease_id(self.pool.acquire(purpose))

    def joined(self, lease, agent="agent-1", read_only=False):
        return self.member_id(self.pool.join(self.token_of(self.pool.token(lease, read_only)), agent))

    def slot_path(self, n=1):
        return os.path.join(self.main, ".claude", "worktrees", f"pool-{n}")

    # --- acquiring -----------------------------------------------------------------------------------------------

    def test_acquire_grants_a_slot_then_says_none_is_free(self):
        self.pool.create()
        text = self.pool.acquire("card: Test")
        self.assertIn("GRANTED pool-1", text)
        self.assertIn(f"PATH {self.slot_path()}", text)
        with self.assertRaisesRegex(wt.Refused, "NO WORKTREE AVAILABLE"):
            self.pool.acquire("card: Other")

    def test_acquire_puts_the_slot_on_todays_main(self):
        self.pool.create()
        self.write(self.main, "new.txt", "x")
        sha = self.commit(self.main, "newer main")
        run(self.main, "push", "--quiet")
        self.granted()
        self.assertEqual(run(self.slot_path(), "rev-parse", "HEAD"), sha)

    def test_a_dirty_slot_is_never_granted(self):
        self.pool.create()
        self.write(self.slot_path(), "stray.txt", "x")
        with self.assertRaisesRegex(wt.Refused, "uncommitted"):
            self.pool.acquire("card: Test")
        self.assertEqual(self.pool.read()["slots"]["pool-1"]["state"], "attention")
        os.remove(os.path.join(self.slot_path(), "stray.txt"))
        self.pool.reconcile(fix=True)
        self.assertEqual(self.pool.read()["slots"]["pool-1"]["state"], "free")

    def test_a_slot_with_unlanded_commits_is_never_granted(self):
        self.pool.create()
        self.write(self.slot_path(), "work.txt", "x")
        self.commit(self.slot_path(), "unlanded")
        with self.assertRaisesRegex(wt.Refused, "unlanded commit"):
            self.pool.acquire("card: Test")

    # --- tokens and members --------------------------------------------------------------------------------------

    def test_a_token_is_single_use(self):
        self.pool.create()
        lease = self.granted()
        token = self.token_of(self.pool.token(lease))
        self.assertIn(f"PATH {self.slot_path()}", self.pool.join(token, "agent-1"))
        with self.assertRaisesRegex(wt.Refused, "not valid"):
            self.pool.join(token, "agent-2")

    def test_a_token_dies_after_two_minutes(self):
        self.pool.create()
        lease = self.granted()
        token = self.token_of(self.pool.token(lease))
        self.later(minutes=2, seconds=1)
        with self.assertRaisesRegex(wt.Refused, "died"):
            self.pool.join(token, "agent-1")
        self.token_of(self.pool.token(lease))  # the dead one no longer blocks a new one

    def test_one_token_out_at_a_time(self):
        self.pool.create()
        lease = self.granted()
        self.pool.token(lease)
        with self.assertRaisesRegex(wt.Refused, "already out"):
            self.pool.token(lease)

    def test_no_token_while_a_member_is_in_the_slot(self):
        self.pool.create()
        lease = self.granted()
        member = self.joined(lease)
        with self.assertRaisesRegex(wt.Refused, "has a member"):
            self.pool.token(lease)
        self.pool.leave(member)
        self.token_of(self.pool.token(lease, read_only=True))

    def test_join_needs_an_agent_id(self):
        self.pool.create()
        lease = self.granted()
        with self.assertRaisesRegex(wt.Refused, "--agent-id"):
            self.pool.join(self.token_of(self.pool.token(lease)), None)

    def test_evict_ends_a_member(self):
        self.pool.create()
        lease = self.granted()
        member = self.joined(lease)
        self.pool.evict(lease, member, "its agent failed")
        self.assertEqual(self.pool.read()["members"], {})
        with self.assertRaisesRegex(wt.Refused, "no member"):
            self.pool.leave(member)

    # --- releasing -----------------------------------------------------------------------------------------------

    def test_release_frees_the_slot(self):
        self.pool.create()
        lease = self.granted()
        self.pool.release(lease)
        self.assertEqual(self.pool.read()["slots"]["pool-1"]["state"], "free")
        self.granted()

    def test_release_refused_while_a_member_is_in(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease)
        with self.assertRaisesRegex(wt.Refused, "still has a member"):
            self.pool.release(lease)

    def test_release_refused_until_the_work_has_landed_by_cherry_pick(self):
        self.pool.create()
        lease = self.granted()
        member = self.joined(lease)
        self.write(self.slot_path(), "card.txt", "the card")
        sha = self.commit(self.slot_path(), "Card: Test")
        self.pool.leave(member)
        with self.assertRaisesRegex(wt.Refused, "unlanded commit"):
            self.pool.release(lease)
        self.write(self.main, "other.txt", "someone else's work first")  # so the landed commit's hash differs
        self.commit(self.main, "other")
        run(self.main, "cherry-pick", sha)
        run(self.main, "push", "--quiet")
        self.assertIn("Released pool-1", self.pool.release(lease))
        self.assertEqual(run(self.slot_path(), "rev-parse", "HEAD"), run(self.main, "rev-parse", "HEAD"))

    def test_release_discard_throws_unlanded_work_away(self):
        self.pool.create()
        lease = self.granted()
        self.write(self.slot_path(), "card.txt", "the card")
        self.commit(self.slot_path(), "Card: Test")
        self.write(self.slot_path(), "scratch.txt", "x")
        with self.assertRaisesRegex(wt.Refused, "--reason"):
            self.pool.release(lease, discard=True)
        self.pool.release(lease, discard=True, reason="abandoned card")
        self.assertFalse(os.path.exists(os.path.join(self.slot_path(), "card.txt")))
        self.assertFalse(os.path.exists(os.path.join(self.slot_path(), "scratch.txt")))
        self.assertEqual(self.pool.read()["log"][-1]["reason"], "abandoned card")

    def test_sync_rebases_onto_main(self):
        self.pool.create()
        lease = self.granted()
        member = self.joined(lease)
        self.write(self.slot_path(), "card.txt", "the card")
        self.commit(self.slot_path(), "Card: Test")
        self.write(self.main, "other.txt", "x")
        main_sha = self.commit(self.main, "other")
        run(self.main, "push", "--quiet")
        with self.assertRaisesRegex(wt.Refused, "only it may sync"):
            self.pool.sync(lease)
        self.pool.sync(member)
        self.assertEqual(run(self.slot_path(), "rev-parse", "HEAD~1"), main_sha)

    # --- reconcile and the lock ----------------------------------------------------------------------------------

    def test_reconcile_adopts_existing_pool_directories(self):
        run(self.main, "worktree", "add", "--quiet", "-b", "pool-1", self.slot_path(), "origin/main")
        run(self.main, "worktree", "add", "--quiet", "-b", "pool-2", self.slot_path(2), "origin/main")
        self.write(self.slot_path(2), "w.txt", "x")
        self.commit(self.slot_path(2), "unlanded")
        self.assertIn("not in the pool", self.pool.reconcile())
        self.assertEqual(self.pool.read()["slots"], {})
        self.pool.reconcile(fix=True)
        slots = self.pool.read()["slots"]
        self.assertEqual(slots["pool-1"]["state"], "free")
        self.assertEqual(slots["pool-2"]["state"], "attention")

    def test_clean_frees_a_slot_set_aside_with_uncommitted_changes(self):
        run(self.main, "worktree", "add", "--quiet", "-b", "pool-1", self.slot_path(), "origin/main")
        self.write(self.slot_path(), "stub.txt", "x")
        self.pool.reconcile(fix=True)
        self.assertIn("stub.txt", self.pool.status("pool-1"))
        self.assertIn("nobody holds", self.pre("Bash", None, command="rm .claude/worktrees/pool-1/stub.txt"))
        with self.assertRaisesRegex(wt.Refused, "--reason"):
            self.pool.clean("pool-1", "")
        self.assertIn("Cleaned pool-1", self.pool.clean("pool-1", "a stale card stub"))
        self.assertFalse(os.path.exists(os.path.join(self.slot_path(), "stub.txt")))
        self.assertEqual(self.pool.read()["slots"]["pool-1"]["state"], "free")
        self.assertEqual(self.pool.read()["log"][-1]["reason"], "a stale card stub")
        self.granted()

    def test_clean_keeps_unlanded_commits_unless_told_to_discard_them(self):
        run(self.main, "worktree", "add", "--quiet", "-b", "pool-1", self.slot_path(), "origin/main")
        self.write(self.slot_path(), "w.txt", "x")
        self.commit(self.slot_path(), "unlanded")
        self.pool.reconcile(fix=True)
        with self.assertRaisesRegex(wt.Refused, "on no origin branch"):
            self.pool.clean("pool-1", "old work")
        self.assertTrue(os.path.exists(os.path.join(self.slot_path(), "w.txt")))
        run(self.slot_path(), "push", "--quiet", "origin", "pool-1:refs/heads/kept")
        self.pool.clean("pool-1", "pushed to kept")
        self.assertFalse(os.path.exists(os.path.join(self.slot_path(), "w.txt")))
        self.write(self.slot_path(), "w2.txt", "y")
        self.commit(self.slot_path(), "unwanted")
        self.pool.clean("pool-1", "unwanted", discard_commits=True)
        self.assertEqual(run(self.slot_path(), "rev-parse", "HEAD"), run(self.main, "rev-parse", "origin/main"))

    def test_clean_refused_on_a_leased_slot(self):
        self.pool.create()
        self.granted()
        with self.assertRaisesRegex(wt.Refused, "held by lease"):
            self.pool.clean("pool-1", "mine now")

    def test_reconcile_lists_stale_leases(self):
        self.pool.create()
        self.granted("card: Slow")
        self.later(hours=7)
        self.assertIn("is its holder still at work", self.pool.reconcile())

    def test_a_held_lock_refuses_and_a_dead_one_is_broken(self):
        self.pool.create()
        with open(self.pool.lock_path, "w") as f:
            f.write("someone")
        saved = wt.LOCK_WAIT
        wt.LOCK_WAIT = 0.3
        try:
            with self.assertRaisesRegex(wt.Refused, "lock"):
                self.pool.acquire("card: Test")
            old = os.path.getmtime(self.pool.lock_path) - wt.LOCK_STALE - 1
            os.utime(self.pool.lock_path, (old, old))
            self.granted()
        finally:
            wt.LOCK_WAIT = saved

    # --- hooks ---------------------------------------------------------------------------------------------------

    def pre(self, tool, agent=None, cwd=None, **tool_input):
        event = {"hook_event_name": "PreToolUse", "tool_name": tool, "tool_input": tool_input,
                 "cwd": cwd or self.main}
        if agent:
            event["agent_id"] = agent
        answer = self.pool.hook(event)
        return answer and answer["hookSpecificOutput"]["permissionDecisionReason"]

    def test_hook_denies_writes_by_anyone_but_the_member(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease, "agent-1")
        file = os.path.join(self.slot_path(), "x.txt")
        self.assertIsNone(self.pre("Write", "agent-1", file_path=file))
        self.assertIn("not a member", self.pre("Write", "agent-2", file_path=file))
        self.assertIn("member at work", self.pre("Edit", None, file_path=file))
        self.assertIsNone(self.pre("Write", "agent-2", file_path=os.path.join(self.main, "x.txt")))
        self.assertIsNone(self.pre("Read", "agent-2", file_path=file))

    def test_hook_reads_shell_commands_and_the_working_directory(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease, "agent-1")
        self.assertIn("not a member", self.pre("Bash", "agent-2", command="cd /c/x/.claude/worktrees/pool-1 && ls"))
        self.assertIn("not a member", self.pre("PowerShell", "agent-2",
                                               command="Get-ChildItem C:\\x\\.claude\\worktrees\\pool-1"))
        self.assertIn("not a member", self.pre("Bash", "agent-2", cwd=self.slot_path(), command="ls"))
        self.assertIsNone(self.pre("Bash", "agent-1", cwd=self.slot_path(), command="cargo build"))
        self.assertIsNone(self.pre("Bash", "agent-2", command="tools\\worktree.cmd status"))
        self.assertIsNone(self.pre("Bash", "agent-2", command="ls .claude/worktrees/pool-10"))  # not in the pool

    def test_hook_lets_the_tool_run_from_any_slot_but_not_a_command_that_only_names_it(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease, "agent-1")
        slot = self.slot_path()
        for command in ["tools\\worktree.cmd status",
                        "export PATH=/x:$PATH; cd /c/depot && python tools/maintenance/worktrees.py leave M-1",
                        "$env:PYTHONUTF8 = 1; & tools\\worktree.cmd status"]:
            self.assertIsNone(self.pre("Bash", "agent-2", cwd=slot, command=command), command)
        for command in ["rm x.txt # worktrees.py",
                        "rm x.txt; tools\\worktree.cmd status",
                        "tools\\worktree.cmd status > x.txt",
                        "tools\\worktree.cmd status $(rm x.txt)",
                        "echo tools/worktree.cmd"]:
            self.assertIn("not a member", self.pre("Bash", "agent-2", cwd=slot, command=command), command)
        self.assertIn("not a member", self.pre("Bash", "agent-2",
                                               command="rm .claude/worktrees/pool-1/x.txt # tools\\worktree.cmd"))

    def test_hook_keeps_a_read_only_member_from_editing(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease, "agent-1", read_only=True)
        file = os.path.join(self.slot_path(), "x.txt")
        self.assertIn("read-only", self.pre("Edit", "agent-1", file_path=file))
        self.assertIsNone(self.pre("Bash", "agent-1", cwd=self.slot_path(), command="cargo test"))

    def test_hook_keeps_everyone_out_of_a_free_slot_and_lets_the_main_thread_into_an_empty_lease(self):
        self.pool.create()
        file = os.path.join(self.slot_path(), "x.txt")
        self.assertIn("nobody holds", self.pre("Write", None, file_path=file))
        self.granted()
        self.assertIsNone(self.pre("Write", None, file_path=file))
        self.assertIn("not a member", self.pre("Write", "agent-1", file_path=file))

    def test_subagent_stop_ends_its_membership(self):
        self.pool.create()
        lease = self.granted()
        self.joined(lease, "agent-1")
        self.pool.hook({"hook_event_name": "SubagentStop", "agent_id": "agent-1"})
        self.assertEqual(self.pool.read()["members"], {})
        self.token_of(self.pool.token(lease))

    def test_acquire_refused_until_the_hooks_are_installed(self):
        self.pool.create()
        pool = wt.Pool(root=self.main, now=lambda: self.clock[0], warm=lambda path: None)
        with self.assertRaisesRegex(wt.Refused, "opt-in"):
            pool.acquire("card: Test")
        wt.hooks_command(self.main, "install")
        self.assertIn("GRANTED", pool.acquire("card: Test"))

    def test_hooks_install_and_uninstall_keep_the_rest_of_the_settings(self):
        os.makedirs(os.path.join(self.main, ".claude"))
        self.write(os.path.join(self.main, ".claude"), "settings.local.json",
                   '{"permissions": {"allow": ["Bash(git push)"]}, "hooks": {"PreToolUse": '
                   '[{"matcher": "Bash", "hooks": [{"type": "command", "command": "mine"}]}]}}')
        wt.hooks_command(self.main, "install")
        wt.hooks_command(self.main, "install")  # twice is once
        self.assertTrue(wt.hooks_installed(self.main))
        with open(wt.settings_path(self.main), encoding="utf-8") as f:
            settings = wt.json.load(f)
        self.assertEqual(len(settings["hooks"]["PreToolUse"]), 2)
        self.assertEqual(len(settings["hooks"]["SubagentStop"]), 1)
        wt.hooks_command(self.main, "uninstall")
        self.assertFalse(wt.hooks_installed(self.main))
        with open(wt.settings_path(self.main), encoding="utf-8") as f:
            settings = wt.json.load(f)
        self.assertEqual(settings, {"permissions": {"allow": ["Bash(git push)"]}, "hooks": {"PreToolUse": [
            {"matcher": "Bash", "hooks": [{"type": "command", "command": "mine"}]}]}})

    def test_subagent_start_tells_the_agent_its_id(self):
        answer = self.pool.hook({"hook_event_name": "SubagentStart", "agent_id": "agent-7"})
        self.assertIn("agent-7", answer["hookSpecificOutput"]["additionalContext"])


if __name__ == "__main__":
    unittest.main()
