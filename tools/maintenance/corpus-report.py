"""Validate and print scenario accounting emitted by the checked workspace tests, without replaying games."""
import argparse
import json
import sys


def validate(data, run_id):
    if not isinstance(data, dict) or type(data.get("schema")) is not int or data.get("schema") != 1 or data.get("run_id") != run_id:
        raise ValueError("corpus accounting belongs to a different test run")
    if (data.get("complete") is not True or type(data.get("expected")) is not int
            or type(data.get("completed")) is not int or data["expected"] < 0
            or data.get("completed") != data.get("expected")):
        raise ValueError("corpus accounting is incomplete (filtered or interrupted scenario tests)")
    fields = ("passed", "failed", "known_bugs", "fixed", "missing")
    if any(type(data.get(k)) is not int or data[k] < 0 for k in fields):
        raise ValueError("corpus accounting contains invalid counts")
    if sum(data[k] for k in fields) != data["completed"]:
        raise ValueError("corpus accounting totals disagree")
    if data["failed"] or data["fixed"]:
        raise ValueError("corpus accounting contains failed or unexpectedly fixed scenarios")
    missing = data.get("missing_cards")
    if not isinstance(missing, dict) or any(not isinstance(k, str) or not k or type(v) is not int or v < 0 for k, v in missing.items()):
        raise ValueError("corpus accounting contains invalid missing-card counts")
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path")
    parser.add_argument("--run-id", required=True)
    args = parser.parse_args()
    try:
        with open(args.path, encoding="utf-8") as source:
            data = validate(json.load(source), args.run_id)
    except (OSError, ValueError, TypeError) as error:
        print(f"corpus accounting: FAIL: {error}")
        return 1
    print(f"{data['expected']} scenarios: {data['passed']} pass, {data['failed']} fail, "
          f"{data['known_bugs']} known bugs, {data['missing']} need cards the engine does not have.")
    for name, count in sorted(data["missing_cards"].items()):
        print(f"  needs {name}: {count} scenario(s)")
    print(f"corpus report: {args.path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
