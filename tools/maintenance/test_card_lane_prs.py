"""Offline PR-cache and queue lifecycle regressions; no live GitHub mutations."""
import base64
import contextlib
import io
import json
import os
from pathlib import Path
import unittest
from unittest import mock
from tools.maintenance import test_card_lane as fixtures


def pr(number=1, head="h1", base="b1"):
    return {"number": number, "state": "open", "html_url": f"https://github.com/owner/repo/pull/{number}",
            "head": {"sha": head, "repo": {"full_name": "fork/repo"}}, "base": {"sha": base}}


def added(name, filename="content/mtg/cards/card-b.json", faces=None):
    data = {"name": name, "faces": faces or []}
    return {"filename": filename, "status": "added", "patch": "@@ -0,0 +1 @@\n+" + json.dumps(data)}


class PrCacheTest(unittest.TestCase):
    setUp = fixtures.LaneTest.setUp
    tearDown = fixtures.LaneTest.tearDown
    git = fixtures.LaneTest.git
    write = fixtures.LaneTest.write
    lane = fixtures.LaneTest.lane
    state = fixtures.LaneTest.state
    worktree = fixtures.LaneTest.worktree

    def inventory(self, responses, refresh=False):
        lane = fixtures.load_lane()
        with mock.patch.object(lane.upstream_prs, "api", side_effect=responses) as calls:
            work, info = lane.upstream_prs.inventory(lane.api(), self.main, "owner/repo", refresh)
        return work, info, calls

    def command(self, responses, *extra):
        lane = fixtures.load_lane()
        saved = os.getcwd()
        try:
            os.chdir(self.main)
            argv = ["queue", "--queue", "q", "--repo", "owner/repo", *extra]
            args = lane.command_line().parse_args(argv)
            with mock.patch.object(lane.upstream_prs, "api", side_effect=responses), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(lane.cmd_queue(args), 0)
        finally:
            os.chdir(saved)
        return self.state()

    def test_cached_metadata_and_unchanged_head_avoid_file_requests(self):
        work, info, calls = self.inventory([[pr()], [added("Card B")], pr()])
        self.assertEqual(info["state"], "refreshed")
        self.assertEqual(calls.call_count, 3)
        self.assertEqual(work["card-b"][0]["number"], 1)
        cached, info, calls = self.inventory([])
        self.assertEqual(cached, work)
        self.assertEqual(info["state"], "cached")
        calls.assert_not_called()
        refreshed, info, calls = self.inventory([[pr()]], refresh=True)
        self.assertEqual(refreshed, work)
        self.assertEqual(calls.call_count, 1)

    def test_head_or_base_change_invalidates_file_inventory(self):
        self.inventory([[pr()], [added("Card B")], pr()])
        changed, _, calls = self.inventory([[pr(head="h2")], [added("Card C")], pr(head="h2")], refresh=True)
        self.assertNotIn("card-b", changed)
        self.assertIn("card-c", changed)
        self.assertEqual(calls.call_count, 3)
        changed, _, calls = self.inventory([[pr(head="h2", base="b2")], [added("Card D")], pr(head="h2", base="b2")], refresh=True)
        self.assertIn("card-d", changed)
        self.assertEqual(calls.call_count, 3)

    def test_face_names_rust_files_and_removed_files(self):
        work, _, _ = self.inventory([[pr()], [
            added("Fire // Ice", faces=[{"name": "Fire"}, {"name": "Ice"}]),
            {"filename": "crates/cantrip-cards/src/cards/captains_log.rs", "status": "modified"},
            {"filename": "crates/cantrip-cards/src/cards/mod.rs", "status": "modified"},
            {"filename": "content/mtg/cards/removed.json", "status": "removed"},
            {"filename": "scenarios/irrelevant.cantrip", "status": "added"}], pr()])
        self.assertEqual(set(work), {"fire-ice", "fire", "ice", "captains-log"})

    def test_truncated_patch_fetches_current_head_content_and_caches_it(self):
        data = {"name": "Card B", "back": {"name": "Face B"}}
        payload = {"encoding": "base64", "content": base64.b64encode(json.dumps(data).encode()).decode()}
        work, info, calls = self.inventory([[pr()], [{"filename": "content/mtg/cards/card-b.json",
                                                   "status": "added", "patch": "@@\n+{\n+\"name\": \"Card B\","}], payload, pr()])
        self.assertEqual(info["state"], "refreshed")
        self.assertEqual(set(work), {"card-b", "face-b"})
        self.assertIn("repos/fork/repo/contents/content/mtg/cards/card-b.json?ref=h1", calls.call_args_list[-2].args)
        self.inventory([])[2].assert_not_called()

    def test_expired_refresh_failure_never_excludes_from_stale_cache(self):
        _, info, _ = self.inventory([[pr()], [added("Card B")], pr()])
        lane = fixtures.load_lane()
        with mock.patch.object(lane.upstream_prs.time, "time", return_value=info["fetched"] + 301):
            work, failed, _ = self.inventory(ValueError("offline"))
        self.assertEqual(work, {})
        self.assertEqual(failed["state"], "unavailable")
        self.assertIn("offline", failed["warning"])
        state = {"cards": [{"name": "Card B", "status": "deferred-pr", "claim": None, "upstream_prs": [pr()]}]}
        lane.upstream_prs.reconcile(lane.api(), state, work)
        self.assertEqual(state["cards"][0]["status"], "pending")

    def test_closed_unmerged_requeues_without_creating_branch_for_deferred_only(self):
        deferred = self.command([[pr()], [added("Card B")], pr()], "--card", "Card B")
        self.assertEqual(deferred["cards"][0]["status"], "deferred-pr")
        self.assertEqual(deferred["cards"][0]["upstream_prs"][0]["number"], 1)
        self.assertFalse(self.git(self.main, "show-ref", "--verify", "refs/heads/card-lane/q", check=False))
        pending = self.command([[]], "--refresh-prs", "--card", "Card B")
        self.assertEqual(pending["cards"][0]["status"], "pending")
        self.assertNotIn("upstream_prs", pending["cards"][0])
        self.assertTrue(self.git(self.main, "rev-parse", "card-lane/q"))

    def test_refresh_preserves_active_claim_and_retires_merged_deferral(self):
        self.command([[]], "--card", "Card B", "--card", "Card C")
        worker = self.worktree("author")
        claimed = self.lane(worker, "next", "--queue", "q", "--id", "w1")
        self.assertEqual(claimed.returncode, 0, claimed.stderr)
        before = self.state()
        deferred = self.command([[pr()], [added("Card B"), added("Card C", "content/mtg/cards/card-c.json")], pr()],
                                "--refresh-prs", "--card", "Card B", "--card", "Card C")
        self.assertEqual(deferred["cards"][0]["claim"], before["cards"][0]["claim"])
        self.assertEqual(deferred["cards"][0]["status"], "working")
        self.assertEqual(deferred["cards"][1]["status"], "deferred-pr")
        self.write("content/mtg/cards/card-c.json", json.dumps({"name": "Card C"}))
        self.git(self.main, "add", ".")
        self.git(self.main, "commit", "--quiet", "-m", "upstream card C")
        self.git(self.main, "push", "--quiet")
        after = self.command([[]], "--refresh-prs", "--card", "Card B", "--card", "Card C")
        self.assertEqual(after["cards"][0]["claim"], before["cards"][0]["claim"])
        self.assertEqual(after["cards"][1]["status"], "done")
        self.assertNotIn("upstream_prs", after["cards"][1])

    def test_mutable_file_response_is_not_cached_under_stale_head(self):
        work, info, _ = self.inventory([[pr()], [added("Wrong Head Card")], pr(head="new")])
        self.assertEqual(work, {})
        self.assertEqual(info["state"], "unavailable")
        self.assertIn("changed during inventory", info["warning"])
        cache = Path(self.main, ".git", "card-lane", "upstream-prs", "owner", "repo")
        self.assertEqual(list(cache.glob("*.json")), [])

    def test_cache_directory_error_warns_instead_of_aborting_queue(self):
        lane = fixtures.load_lane()
        with mock.patch.object(lane.upstream_prs.Path, "mkdir", side_effect=OSError("cache unavailable")):
            work, info = lane.upstream_prs.inventory(lane.api(), self.main, "owner/repo")
        self.assertEqual(work, {})
        self.assertEqual(info["state"], "unavailable")
        self.assertIn("cache unavailable", info["warning"])


if __name__ == "__main__":
    unittest.main()
