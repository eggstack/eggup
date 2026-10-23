#!/usr/bin/env bash
# Documentation-truth regression checks for Eggup.
#
# Read-only, network-free, root-free, and cargo-free. It reads the repository's
# documentation surfaces and compares them against each other, the manifests,
# and the planning registry. It never writes, never fetches, and never needs
# elevated privileges.
#
# Usage:
#   ./scripts/check-docs-truth.sh                  # assertions 1-5
#   ./scripts/check-docs-truth.sh --baseline <ref> # also assertion 7, scoped to <ref>
#
# Assertion 6 (the transport-footprint dependency counts) is a `cargo tree`
# resolution check and therefore lives in the milestone's verification
# commands, not in this script: this script must not invoke cargo.
#
# Source of the assertions: planning C013,
# plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md
# §10.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASELINE=""
if [ "${1:-}" = "--baseline" ]; then
  BASELINE="${2:-}"
  if [ -z "$BASELINE" ]; then
    echo "FAIL --baseline requires a git ref" >&2
    exit 2
  fi
fi

python3 - "$BASELINE" <<'PY'
import os
import re
import subprocess
import sys

baseline = sys.argv[1] if len(sys.argv) > 1 else ""
failures = []


def fail(assertion, detail):
    failures.append("%s: %s" % (assertion, detail))


def read(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def lines(path):
    return read(path).split("\n")


def walk(root):
    out = []
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if name.endswith(".md"):
                out.append(os.path.join(dirpath, name))
    return sorted(out)


DOC_FILES = walk("docs")
SURFACES = DOC_FILES + ["README.md", "AGENTS.md"]

# ---------------------------------------------------------------- assertion 1
# No docs/ file, README.md, or AGENTS.md claims a version outside the known set.
KNOWN_VERSIONS = {"0.1.0", "0.1.1", "0.1.2", "0.1.3"}
for path in SURFACES:
    for number, line in enumerate(lines(path), 1):
        for literal in re.findall(r"0\.1\.\d+", line):
            if literal not in KNOWN_VERSIONS:
                fail("1", "%s:%d: version literal %s outside the known set %s"
                     % (path, number, literal, sorted(KNOWN_VERSIONS)))

# ---------------------------------------------------------------- assertion 2
# No docs/ file states the current version as 0.1.2.
#   - "workspace version" must not name 0.1.2
#   - "published at `0.1.2`" may survive exactly once: docs/crates.md's
#     eggup-curl entry, which is correct
#   - "=0.1.2" may survive only as the docs/releases.md history reference that
#     describes what eggup-eggpack 0.1.2 pinned
for path in DOC_FILES + ["README.md", "AGENTS.md"]:
    text = read(path)
    for paragraph in re.split(r"\n\s*\n", text):
        if "workspace version" in paragraph and "0.1.2" in paragraph:
            fail("2", "%s: a 'workspace version' paragraph names 0.1.2" % path)

published_hits = []
for path in DOC_FILES:
    for number, line in enumerate(lines(path), 1):
        if re.search(r"published at `0\.1\.2`", line):
            published_hits.append((path, number, line))
if len(published_hits) > 1:
    for path, number, line in published_hits:
        fail("2", "%s:%d: 'published at `0.1.2`' is not the single legitimate eggup-curl occurrence: %s"
             % (path, number, line.strip()))
elif len(published_hits) == 1:
    path, number, line = published_hits[0]
    if path != "docs/crates.md":
        fail("2", "%s:%d: the surviving 'published at `0.1.2`' must be docs/crates.md (eggup-curl)" % (path, number))
    if "eggup-curl" not in read(path):
        fail("2", "docs/crates.md no longer names eggup-curl, so the 0.1.2 occurrence is not the legitimate one")

for path in DOC_FILES + ["README.md", "AGENTS.md", "architecture/overview.md",
                         "architecture/tooling-governance.md"]:
    text = read(path)
    for paragraph in re.split(r"\n\s*\n", text):
        if "=0.1.2" not in paragraph:
            continue
        if path == "docs/releases.md" and "eggup-eggpack 0.1.2" in paragraph:
            continue  # allowlisted: the history of what eggup-eggpack 0.1.2 pinned
        fail("2", "%s: '=0.1.2' survives outside the allowlisted history reference" % path)

# ---------------------------------------------------------------- assertion 3
# No normative rule leaked into docs/: a file that states a rule whose owner is
# architecture/ or crates/eggup-core/docs/ must link to that owner.
RULE_CONSTRUCTS = (
    "no escape hatch",
    "fail closed with a receipt",
    "never creates the destination",
    "requires every member",
    "under the lock",
)
OWNER_LINKS = ("crates/eggup-core/docs/", "architecture/core-transaction.md")
for path in DOC_FILES:
    text = read(path)
    hits = [c for c in RULE_CONSTRUCTS if c in text]
    if not hits:
        continue
    if not any(owner in text for owner in OWNER_LINKS):
        fail("3", "%s: states a normative rule (%s) without linking its owner (%s)"
             % (path, ", ".join(hits), " / ".join(OWNER_LINKS)))

# ---------------------------------------------------------------- assertion 4
# The root CHANGELOG.md Unreleased section does not contradict the 0.1.3 section.
ALIAS = {"eggup-core": "Core", "eggup-acquisition": "Acquisition", "eggup-eggfetch": "Eggfetch",
         "eggup-curl": "Curl", "eggup-archive": "Archive", "eggup-eggpack": "Eggpack",
         "eggup-service": "Service"}
# A bullet that names a crate published at 0.1.3 is correctly marked when it
# states that the published version still carries the defect it corrects.
ABSENT_FROM_PUBLISHED = re.compile(
    r"still carries|remains the published version|not published|not yet published"
    r"|blocks propagation|do(?:es)? not (?:yet )?have|until a .{0,20}republication"
    r"|is not in the published", re.I)


def squash(text):
    return " ".join(text.replace("*", "").replace("`", "").split())


changelog = read("CHANGELOG.md")
if "## Unreleased" not in changelog:
    fail("4", "CHANGELOG.md has no '## Unreleased' section")
else:
    head, unreleased = changelog.split("## Unreleased", 1)
    unreleased = unreleased.split("\n## ", 1)[0]
    squashed = " ".join(unreleased.split())
    for header in ("Published baselines are `0.1.2`",
                   "No publication milestone authorizes these entries yet"):
        if header in squashed:
            fail("4", "root Unreleased header still asserts: %s" % header)
    published = sorted(set(re.findall(r"`(eggup-[a-z]+) 0\.1\.3`", head)))
    for bullet in [b for b in re.split(r"\n(?=- )", unreleased) if b.startswith("- ")]:
        names = [c for c in published if c in bullet or (ALIAS.get(c) and ALIAS[c] in bullet)]
        if not names:
            continue
        plain = squash(bullet)
        if "0.1.3" in plain and ABSENT_FROM_PUBLISHED.search(plain):
            continue  # correctly marked: explicitly absent from the published 0.1.3
        if re.search(r"\bunpublished\b", plain, re.I):
            fail("4", "published-as-unpublished: %s | %s" % (", ".join(names), bullet.split("\n")[0][:70]))
        else:
            fail("4", "published-with-no-state-marker: %s | %s" % (", ".join(names), bullet.split("\n")[0][:70]))

# ---------------------------------------------------------------- assertion 5
# No milestone appears with two different statuses in plans/registry.md, and the
# registry names its authoritative table.


def classify(cell):
    cell = cell.lower()
    if "conditional" in cell:
        return "conditional"
    if "closed" in cell:
        return "closed"
    if "historical" in cell or "superseded" in cell:
        return "historical"
    if "authorable" in cell or "writable" in cell:
        return "authorable"
    if "blocked" in cell:
        return "blocked"
    if "ready" in cell:
        return "ready"
    return "other"


registry = lines("plans/registry.md")
rows = {}
for number, line in enumerate(registry, 1):
    if not line.startswith("|"):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) < 4 or set("".join(cells)) <= set("-: ") or cells[0] == "Subsystem":
        continue
    rows.setdefault((cells[0], cells[1]), []).append((number, classify(cells[2])))
for key, seen in sorted(rows.items()):
    classes = {c for _, c in seen}
    if len(classes) > 1:
        fail("5", "milestone %r resolves to several status classes: %s" % (key, seen))

registry_text = "\n".join(registry)
CROSS_SURFACE = (
    r"M010 eggup-acquisition 0\.1\.3 publication[^|]*\|[^|]*ready",
    r"M010 eggup-acquisition 0\.1\.3 publication \[READY",
    r"M004 Gregg.*blocked",
    r"M007 EggPool selective.*blocked",
    r"remains intentionally unwritten until Acquisition M010",
    r"once workspace 0\.1\.3 exists",
)
for pattern in CROSS_SURFACE:
    for number, line in enumerate(registry, 1):
        if re.search(pattern, line):
            fail("5", "registry:%d: cross-surface contradiction %r" % (number, pattern))
if not any("authoritative" in line and "Dependency-ready implementation work" in line
           for line in registry):
    fail("5", "registry does not name the 'Dependency-ready implementation work' table as authoritative")

# ---------------------------------------------------------------- assertion 7
# No production source changed. Scoped to the milestone's own change set, so it
# runs only when a baseline ref is supplied.
if baseline:
    try:
        changed = subprocess.run(
            ["git", "diff", "--name-only", baseline],
            check=True, capture_output=True, text=True).stdout.split("\n")
    except subprocess.CalledProcessError as error:
        fail("7", "git diff --name-only %s failed: %s" % (baseline, error.stderr.strip()))
        changed = []
    for path in sorted(p for p in changed if p):
        if path.startswith("crates/") and "/src/" in path:
            fail("7", "production source changed: %s" % path)
        if path == "Cargo.lock":
            fail("7", "Cargo.lock changed")
        if re.match(r"crates/[^/]+/Cargo\.toml$", path):
            fail("7", "a crate manifest changed: %s" % path)
else:
    print("note 7: skipped — pass --baseline <ref> to scope the no-production-delta check")

# ---------------------------------------------------------------------- report
if failures:
    for failure in failures:
        print("FAIL %s" % failure)
    print("documentation-truth checks failed: %d" % len(failures))
    sys.exit(1)
print("documentation-truth checks passed")
PY
