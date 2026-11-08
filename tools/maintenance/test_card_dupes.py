"""Shared recipe calls are composition; duplicated local decisions still need review."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("card_dupes", Path(__file__).with_name("card-dupes.py"))
dupes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dupes)


class DupesTest(unittest.TestCase):
    def test_shared_wrapper_exempt_but_repeated_local_logic_is_not(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory, "card.rs")
            path.write_text("""
fn composed(g: &mut Game) { effects::draw(g, 1); }
fn keyword(g: &mut Game) { cantrip_mechanics::evoke::evoke(g, cost); }
fn local(g: &mut Game) { let n = g.query().controller(); effects::draw(g, n); }
""", encoding="utf-8")
            found = {name: statements for name, _, statements in dupes.functions(path)}
            self.assertLess(found["composed"], dupes.MIN_STATEMENTS)
            self.assertLess(found["keyword"], dupes.MIN_STATEMENTS)
            self.assertGreaterEqual(found["local"], dupes.MIN_STATEMENTS)


if __name__ == "__main__":
    unittest.main()
