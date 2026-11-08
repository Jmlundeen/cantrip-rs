"""Cached open upstream PR inventory. Deferral is evidence of work, not implementation proof."""
import base64
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time
from urllib.parse import quote

SCHEMA = 2  # names are cached only after verifying the mutable PR response's identity
TTL = 300
CONTENT = re.compile(r"content/[^/]+/cards/([^/]+)\.json$")
RUST = re.compile(r"crates/cantrip-cards/src/cards/([^/]+)\.rs$")


def api(root, endpoint, paginated=False):
    command = ["gh", "api", endpoint]
    if paginated:
        command += ["--paginate", "--slurp"]
    result = subprocess.run(command, cwd=root, capture_output=True, text=True, encoding="utf-8", timeout=45)
    if result.returncode:
        raise ValueError(result.stderr.strip() or "GitHub API request failed")
    value = json.loads(result.stdout)
    if paginated:
        if not isinstance(value, list) or any(not isinstance(page, list) for page in value):
            raise ValueError("GitHub returned an invalid paginated response")
        return [item for page in value for item in page]
    return value


def read(lane, path):
    try:
        value = lane.read_json(str(path))
        return value if isinstance(value, dict) and value.get("schema") == SCHEMA else {}
    except (OSError, ValueError):
        return {}


def names_in(lane, value):
    names = set()
    if isinstance(value, dict):
        if isinstance(value.get("name"), str):
            names.add(lane.slug(value["name"]))
        for child in value.values():
            names.update(names_in(lane, child))
    elif isinstance(value, list):
        for child in value:
            names.update(names_in(lane, child))
    return names


def added_content(file):
    """Only a complete added JSON patch can substitute for fetching current content."""
    if file.get("status") != "added" or not file.get("patch"):
        return None
    source = "\n".join(line[1:] for line in file["patch"].splitlines() if line.startswith("+") and not line.startswith("+++"))
    try:
        return json.loads(source)
    except ValueError:
        return None


def pr_identity(pr, repo):
    return {"head": pr["head"]["sha"], "base": pr["base"]["sha"],
            "head_repo": (pr["head"].get("repo") or {}).get("full_name", repo)}


def pr_names(lane, root, cache, repo, pr):
    identity = pr_identity(pr, repo)
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    path = cache / f"{pr['number']}-{key}.json"
    saved = read(lane, path)
    if saved.get("identity") == identity and isinstance(saved.get("names"), list):
        return saved["names"]
    files = api(root, f"repos/{repo}/pulls/{pr['number']}/files?per_page=100", True)
    # GitHub caps this endpoint at 3000 files. An incomplete inventory must not silently defer cards.
    if len(files) >= 3000:
        raise ValueError(f"PR #{pr['number']} has too many files for a complete inventory")
    names = set()
    for file in files:
        if file.get("status") == "removed":
            continue
        filename = file["filename"]
        content, rust = CONTENT.fullmatch(filename), RUST.fullmatch(filename)
        if rust and rust[1] != "mod":
            names.add(lane.slug(rust[1].replace("_", " ")))
        if not content:
            continue
        data = added_content(file)
        if data is None:
            head_repo = identity["head_repo"]
            if not head_repo or not re.fullmatch(r"[\w.-]+/[\w.-]+", head_repo):
                raise ValueError(f"PR #{pr['number']} has no accessible head repository")
            payload = api(root, f"repos/{head_repo}/contents/{quote(filename, safe='/')}?ref={quote(identity['head'], safe='')}")
            if payload.get("encoding") != "base64":
                raise ValueError(f"PR #{pr['number']} content is not available: {filename}")
            data = json.loads(base64.b64decode(payload["content"]).decode("utf-8"))
        names.update(names_in(lane, data))
    # pulls/N/files is mutable. Never label newer patches with the metadata's older SHA pair.
    current = api(root, f"repos/{repo}/pulls/{pr['number']}")
    if current.get("state") != "open" or pr_identity(current, repo) != identity:
        raise ValueError(f"PR #{pr['number']} changed during inventory; rerun queue --refresh-prs")
    lane.write_json(str(path), {"schema": SCHEMA, "identity": identity, "names": sorted(names)})
    return sorted(names)


def inventory(lane, root, repo, refresh=False):
    """Return current work links, refreshing membership but reusing immutable head/base file results."""
    if not repo:
        return {}, {"state": "skipped", "reason": "canonical base has no GitHub repository"}
    cache = Path(lane.common_dir(root), "card-lane", "upstream-prs", repo)
    metadata = cache / "open.json"
    try:
        cache.mkdir(parents=True, exist_ok=True)
        saved = read(lane, metadata)
        timestamp = time.time()
        age = timestamp - saved.get("fetched", 0)
        hit = not refresh and 0 <= age < TTL and isinstance(saved.get("prs"), list)
        if hit:
            prs = saved["prs"]
        else:
            prs = api(root, f"repos/{repo}/pulls?state=open&per_page=100", True)
        result = {}
        # Requests are independent; bounded concurrency keeps cold intake reasonably quick.
        with ThreadPoolExecutor(max_workers=4) as pool:
            futures = [(pr, pool.submit(pr_names, lane, root, cache, repo, pr)) for pr in prs]
            for pr, future in futures:
                names = future.result()
                link = {"number": pr["number"], "url": pr["html_url"], "head_sha": pr["head"]["sha"]}
                for name in names:
                    result.setdefault(name, []).append(link)
        if not hit:
            lane.write_json(str(metadata), {"schema": SCHEMA, "fetched": timestamp, "prs": prs})
        return result, {"state": "cached" if hit else "refreshed", "repo": repo,
                        "fetched": saved["fetched"] if hit else timestamp, "ttl_seconds": TTL,
                        "open_prs": len(prs)}
    except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired) as error:
        # Expired metadata is never used to exclude cards after a failed refresh.
        return {}, {"state": "unavailable", "repo": repo, "warning": str(error)[:600]}


def reconcile(lane, state, work):
    for card in state["cards"]:
        if card["status"] not in ("pending", "deferred-pr") or card.get("claim"):
            continue
        links = work.get(lane.slug(card["name"]), [])
        if links:
            card.update(status="deferred-pr", upstream_prs=links)
        elif card["status"] == "deferred-pr":
            card.update(status="pending")
            card.pop("upstream_prs", None)
