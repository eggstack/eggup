#!/usr/bin/env bash
# Documentation-truth regression checks for Eggup.
#
# Read-only, network-free, root-free, and cargo-free. It reads the repository's
# documentation surfaces and compares them against each other, the crate
# manifests, the per-crate changelogs, and the planning registry. It never
# writes, never fetches, and never needs elevated privileges.
#
# Usage:
#   ./scripts/check-docs-truth.sh                  # assertions 1-5 and 8
#   ./scripts/check-docs-truth.sh --baseline <ref> # also assertion 7, scoped to <ref>
#
# Assertion 6 (the transport-footprint dependency counts) is a `cargo tree`
# resolution check and therefore lives in the milestone's verification
# commands, not in this script: this script must not invoke cargo.
#
# Source of the assertions: planning C013 and C014,
# plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md
# §10 and plans/implementation/planning-closure-hygiene-corrective/014-documentation-truth-corrective.md
# §7.

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
notes = []


def fail(assertion, detail):
    failures.append("%s: %s" % (assertion, detail))


def note(detail):
    notes.append(detail)


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
ARCH_FILES = sorted(
    os.path.join("architecture", name)
    for name in os.listdir("architecture")
    if name.endswith(".md")
)
SURFACES = DOC_FILES + ["README.md", "AGENTS.md"]
ALL_SURFACES = SURFACES + ARCH_FILES

# ---------------------------------------------------------------------- manifests
# A deliberately minimal reader for the manifest fields this gate checks: the
# package name, the package version, the `publish` flag, and the declared
# dependency edges. It is not a general TOML implementation -- it exists so the
# gate keeps one behaviour on every interpreter and zero dependencies.
def parse_manifest(path):
    section = ""
    data = {
        "name": None,
        "version": None,
        "version_workspace": False,
        "publish": True,
        "deps": set(),
    }
    for raw in read(path).split("\n"):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1].strip()
            continue
        if "=" not in line:
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        value = value.strip()
        if section == "package":
            if key == "name":
                data["name"] = value.strip('"').strip("'")
            elif key == "version":
                if value == "true":
                    data["version_workspace"] = True
                else:
                    data["version"] = value.strip('"').strip("'")
            elif key == "publish":
                data["publish"] = value != "false"
        elif section == "workspace.package":
            if key == "version":
                data["version"] = value.strip('"').strip("'")
        else:
            parts = section.split(".")
            tail = parts[-1]
            if tail in ("dependencies", "dev-dependencies", "build-dependencies"):
                data["deps"].add(key)
            elif len(parts) >= 2 and parts[-2] in ("dependencies", "dev-dependencies",
                                                   "build-dependencies"):
                data["deps"].add(tail)
    return data


ROOT_MANIFEST = parse_manifest("Cargo.toml")
WORKSPACE_VERSION = ROOT_MANIFEST["version"] or ""

CRATES = {}
for entry in sorted(os.listdir("crates")):
    crate_dir = os.path.join("crates", entry)
    manifest = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.isfile(manifest):
        continue
    data = parse_manifest(manifest)
    name = data["name"] or entry
    version = data["version"]
    if version is None and data["version_workspace"]:
        version = WORKSPACE_VERSION
    CRATES[name] = {
        "dir": crate_dir,
        "version": version,
        "publish": data["publish"],
        "deps": data["deps"],
    }

CRATE_NAMES = set(CRATES)


def published_versions(name):
    """The versions a crate's own CHANGELOG.md records as released."""
    crate = CRATES.get(name)
    if not crate:
        return set()
    changelog = os.path.join(crate["dir"], "CHANGELOG.md")
    if not os.path.isfile(changelog):
        return set()
    return set(re.findall(r"^##\s+(\d+\.\d+\.\d+)\b", read(changelog), re.M))


def in_workspace_deps(text, exclude=()):
    """The workspace crate names a piece of text names as dependencies.

    A name that is part of a path (`crates/eggup-service/CHANGELOG.md`) names a
    file, not a dependency, so it is not a claim. The subject of the claim is
    never one of its own dependencies.
    """
    return {name for name in re.findall(r"(?<![/.])eggup-[a-z]+(?:-[a-z]+)*", text)
            if name in CRATE_NAMES and name not in exclude}


def split_row(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def is_separator(cells):
    return bool(cells) and all(c and set(c) <= set("-: ") for c in cells)


def paragraphs_with_headings(path):
    """(first line number, paragraph text, heading text) for every paragraph."""
    out = []
    heading = ""
    buffer = []
    start = 0
    for number, line in enumerate(lines(path), 1):
        stripped = line.strip()
        if stripped.startswith("#"):
            if buffer:
                out.append((start, "\n".join(buffer), heading))
                buffer = []
            heading = stripped
            continue
        if not stripped:
            if buffer:
                out.append((start, "\n".join(buffer), heading))
                buffer = []
            continue
        if not buffer:
            start = number
        buffer.append(line)
    if buffer:
        out.append((start, "\n".join(buffer), heading))
    return out


# ---------------------------------------------------------------- assertion 1
# No checked surface claims a version outside the known set. The known set is
# the registry history the repository already published, unioned with the
# version the manifests currently declare. A literal that the same clause
# explicitly disclaims ("not a `0.1.4`") is not a claim and is not a failure.
KNOWN_VERSIONS = {"0.1.0", "0.1.1", "0.1.2", "0.1.3"}
if WORKSPACE_VERSION:
    KNOWN_VERSIONS.add(WORKSPACE_VERSION)
DISCLAIMER = re.compile(
    r"\b(not|no|never|neither|instead of|rather than|unreleased|unpublished)\b", re.I)
CLAUSE_BREAK = re.compile(r"[.;!?]\s")


def disclaimed(text, start):
    """True when the clause that leads to `start` disclaims the literal.

    The window may cross a line break -- markdown wraps lines -- but not a
    sentence boundary, so a disclaimer cannot leak across sentences.
    """
    window = text[max(0, start - 120):start]
    parts = CLAUSE_BREAK.split(window)
    clause = parts[-1] if parts else window
    return bool(DISCLAIMER.search(clause))


for path in ALL_SURFACES:
    full = read(path)
    offset = 0
    for number, line in enumerate(lines(path), 1):
        for match in re.finditer(r"0\.1\.\d+", line):
            literal = match.group(0)
            if literal in KNOWN_VERSIONS:
                continue
            if disclaimed(full, offset + match.start()):
                continue  # the clause disclaims it, so it is not a claim
            fail("1", "%s:%d: version literal %s outside the known set %s"
                 % (path, number, literal, sorted(KNOWN_VERSIONS)))
        offset += len(line) + 1

# ---------------------------------------------------------------- assertion 2
# The published-state claims in the checked surfaces must name the current
# workspace version.
#   - a "workspace version" paragraph must not name any other version
#   - "published at `X`" must name the current version, except in docs/crates.md,
#     which is the one surface that records each crate's published history and
#     may name an older version as a crate's first publication
#   - "=X" (an exact pin) may survive only where it states the eggpack
#     exact-pin policy
#
# Re-based from the `0.1.2` literal when the `0.2.0` train landed, and made
# version-agnostic against WORKSPACE_VERSION so the next train needs no further
# edit of this clause. The shape of the rule is unchanged; only the version it
# is anchored to moved.
CURRENT = WORKSPACE_VERSION

if not CURRENT:
    fail("2", "the workspace manifest declares no version, so the published-state "
              "claims cannot be checked against one")

for path in DOC_FILES + ["README.md", "AGENTS.md"]:
    text = read(path)
    for paragraph in re.split(r"\n\s*\n", text):
        if "workspace version" not in paragraph:
            continue
        for literal in sorted(set(re.findall(r"0\.\d+\.\d+", paragraph))):
            if literal != CURRENT:
                fail("2", "%s: a 'workspace version' paragraph names %s, not the current %s"
                     % (path, literal, CURRENT))

published_hits = []
for path in DOC_FILES:
    for number, line in enumerate(lines(path), 1):
        for match in re.finditer(r"published at `(0\.\d+\.\d+)`", line):
            published_hits.append((path, number, line, match.group(1)))
for path, number, line, literal in published_hits:
    if literal == CURRENT:
        continue
    if path != "docs/crates.md":
        fail("2", "%s:%d: 'published at `%s`' names a version other than the current %s, "
                  "and only docs/crates.md may record an older published version"
             % (path, number, literal, CURRENT))
    if literal not in KNOWN_VERSIONS:
        fail("2", "%s:%d: 'published at `%s`' names a version outside the known set %s"
             % (path, number, literal, sorted(KNOWN_VERSIONS)))

for path in DOC_FILES + ["README.md", "AGENTS.md", "architecture/overview.md",
                         "architecture/tooling-governance.md"]:
    text = read(path)
    for paragraph in re.split(r"\n\s*\n", text):
        if ("=%s" % CURRENT) not in paragraph:
            continue
        if not any(subject in paragraph
                   for subject in ("eggup-eggpack", "eggpack-manifest",
                                   "eggpack-adapter")):
            fail("2", "%s: '=%s' survives outside the eggpack exact-pin policy"
                 % (path, CURRENT))

# ---------------------------------------------------------------- assertion 3
# No normative rule leaked into a non-owner document: a file that states a rule
# whose owner is architecture/ or crates/eggup-core/docs/ must link to that
# owner. The owning deep dive is exempt -- it is the owner, not a leak.
RULE_CONSTRUCTS = (
    "no escape hatch",
    "fail closed with a receipt",
    "never creates the destination",
    "requires every member",
    "under the lock",
    "never inferred",
    "integrity evidence only",
)
OWNER_LINKS = ("crates/eggup-core/docs/", "architecture/core-transaction.md")
RULE_OWNERS = ("architecture/core-transaction.md",)
for path in DOC_FILES + ARCH_FILES:
    if path in RULE_OWNERS:
        continue
    text = read(path)
    hits = [c for c in RULE_CONSTRUCTS if c in text]
    if not hits:
        continue
    if not any(owner in text for owner in OWNER_LINKS):
        fail("3", "%s: states a normative rule (%s) without linking its owner (%s)"
             % (path, ", ".join(hits), " / ".join(OWNER_LINKS)))

# ---------------------------------------------------------------- assertion 4
# The root CHANGELOG.md Unreleased section does not contradict the published
# sections above it.
#
# The published set is read from those sections, so it is version-agnostic: a
# crate published at any version is a crate that is on the registry, and a
# bullet in Unreleased that names it must carry a state marker. A gate that
# knew only one version would silently stop policing a crate the moment a
# newer version shipped, which is exactly the drift this assertion exists to
# prevent.
ALIAS = {"eggup-core": "Core", "eggup-acquisition": "Acquisition", "eggup-eggfetch": "Eggfetch",
         "eggup-curl": "Curl", "eggup-archive": "Archive", "eggup-eggpack": "Eggpack",
         "eggup-service": "Service"}
# A bullet that names a crate published at a version the bullet also states is
# correctly marked when it states that the published version still carries the
# defect it corrects.
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
    published = {}
    for name, version in re.findall(r"`(eggup-[a-z]+) (\d+\.\d+\.\d+)`", head):
        published.setdefault(name, set()).add(version)
    for chunk in re.split(r"\n(?=- )", unreleased):
        if not chunk.startswith("- "):
            continue
        # Each bullet is evaluated on its own paragraph. A bullet that is not
        # followed by another bullet would otherwise absorb the section's
        # trailing paragraph, and that paragraph's wording could satisfy -- or
        # defeat -- the state-marker check on the bullet's behalf.
        bullet = chunk.split("\n\n", 1)[0]
        names = [c for c in published if c in bullet or (ALIAS.get(c) and ALIAS[c] in bullet)]
        if not names:
            continue
        plain = squash(bullet)
        stated = set(re.findall(r"\b\d+\.\d+\.\d+\b", plain))
        on_registry = {v for c in names for v in published[c]}
        if stated & on_registry and ABSENT_FROM_PUBLISHED.search(plain):
            continue  # correctly marked: explicitly absent from a published version
        if re.search(r"\bunpublished\b", plain, re.I):
            fail("4", "published-as-unpublished: %s | %s" % (", ".join(names), bullet.split("\n")[0][:70]))
        else:
            fail("4", "published-with-no-state-marker: %s | %s" % (", ".join(names), bullet.split("\n")[0][:70]))

# ---------------------------------------------------------------- assertion 5
# No milestone appears with two different statuses in plans/registry.md, and the
# registry names its authoritative table. Three-cell tables are read too: the
# status cell is the third cell of every table the registry uses for status.


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
index = 0
while index < len(registry):
    line = registry[index]
    if not line.startswith("|"):
        index += 1
        continue
    cells = split_row(line)
    if len(cells) < 3 or is_separator(cells):
        index += 1
        continue
    # A header row is a row whose next line is the table separator and whose
    # previous line is not itself a table row. Any other row is data and is
    # never dropped, so a malformed table cannot hide a status conflict.
    following = registry[index + 1] if index + 1 < len(registry) else ""
    previous = registry[index - 1] if index > 0 else ""
    is_header = (following.startswith("|") and is_separator(split_row(following))
                 and not previous.startswith("|"))
    if is_header or cells[0] == "Subsystem":
        index += 1
        continue
    rows.setdefault((cells[0], cells[1]), []).append((index + 1, classify(cells[2])))
    index += 1
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
# runs only when a baseline ref is supplied. A crate build script is production
# source and is covered with the crate's src/ tree.
if baseline:
    try:
        changed = subprocess.run(
            ["git", "diff", "--name-only", baseline],
            check=True, capture_output=True, text=True).stdout.split("\n")
    except subprocess.CalledProcessError as error:
        fail("7", "git diff --name-only %s failed: %s" % (baseline, error.stderr.strip()))
        changed = []
    try:
        # A new file is untracked until it is added, and `git diff` does not
        # list untracked files. A build script that was just created must be
        # seen, so the untracked set is read as well.
        untracked = subprocess.run(
            ["git", "ls-files", "--others", "--exclude-standard"],
            check=True, capture_output=True, text=True).stdout.split("\n")
    except subprocess.CalledProcessError as error:
        fail("7", "git ls-files --others failed: %s" % error.stderr.strip())
        untracked = []
    for path in sorted(p for p in changed + untracked if p):
        if path.startswith("crates/") and ("/src/" in path or path.endswith("/build.rs")):
            fail("7", "production source changed: %s" % path)
        if path == "Cargo.lock":
            fail("7", "Cargo.lock changed")
        if re.match(r"crates/[^/]+/Cargo\.toml$", path):
            fail("7", "a crate manifest changed: %s" % path)
else:
    print("note 7: skipped — pass --baseline <ref> to scope the no-production-delta check")

# ---------------------------------------------------------------- assertion 8
# The documentation agrees with the manifests. The manifests are the source of
# truth for the workspace version, for which crates exist and may be published,
# and for which crates may depend on.
if not WORKSPACE_VERSION:
    fail("8", "Cargo.toml declares no [workspace.package] version")

# 8a -- a "workspace version" claim must name the version the manifests declare.
WORKSPACE_CLAIM = re.compile(r"workspace version\b[^.]{0,40}?`(\d+\.\d+\.\d+)`", re.I)
for path in ALL_SURFACES:
    text = " ".join(read(path).split())
    for match in WORKSPACE_CLAIM.finditer(text):
        if WORKSPACE_VERSION and match.group(1) != WORKSPACE_VERSION:
            fail("8", "%s: claims workspace version %s, but Cargo.toml declares %s"
                 % (path, match.group(1), WORKSPACE_VERSION))

# 8b -- a published-version claim must name a version the crate actually
# released, and a crate the manifests declare publish = false is never
# published. A claim whose subject is ambiguous is skipped, never guessed.
PUBLISHED_AT = re.compile(
    r"(?<!un)(?<!not )(?<!never )\bpublished at\s+`?(\d+\.\d+\.\d+)`?", re.I)
for path in ALL_SURFACES:
    for start, paragraph, heading in paragraphs_with_headings(path):
        claims = PUBLISHED_AT.findall(paragraph)
        if not claims:
            continue
        heading_subjects = set(re.findall(r"eggup-[a-z]+(?:-[a-z]+)*", heading))
        subjects = set(re.findall(r"eggup-[a-z]+(?:-[a-z]+)*", paragraph))
        if len(heading_subjects) == 1:
            subject = next(iter(heading_subjects))
        elif len(subjects) == 1:
            subject = next(iter(subjects))
        else:
            note("%s:%d: published-version claim with an ambiguous subject; skipped"
                 % (path, start))
            continue
        crate = CRATES.get(subject)
        if crate is None:
            continue
        released = published_versions(subject)
        for version in claims:
            if not crate["publish"]:
                fail("8", "%s:%d: %s is declared publish = false but is claimed as published at %s"
                     % (path, start, subject, version))
            elif version not in released:
                fail("8", "%s:%d: %s is claimed as published at %s, which its CHANGELOG.md "
                        "does not record as released (released: %s)"
                     % (path, start, subject, version, sorted(released) or "none"))

# 8c -- a dependency claim must name only crates the manifest declares. The
# claim may omit a dependency the manifest declares; it may not invent one.
if os.path.isfile("docs/crates.md"):
    for start, paragraph, heading in paragraphs_with_headings("docs/crates.md"):
        if "**Dependencies:**" not in paragraph:
            continue
        subjects = set(re.findall(r"eggup-[a-z]+(?:-[a-z]+)*", heading))
        if len(subjects) != 1:
            continue
        subject = next(iter(subjects))
        crate = CRATES.get(subject)
        if crate is None:
            continue
        claimed = in_workspace_deps(paragraph, exclude=(subject,))
        invented = claimed - crate["deps"]
        if invented:
            fail("8", "docs/crates.md:%d: %s claims dependency %s, which %s/Cargo.toml "
                    "does not declare" % (start, subject, ", ".join(sorted(invented)), crate["dir"]))

if os.path.isfile("architecture/tooling-governance.md"):
    governance = lines("architecture/tooling-governance.md")
    index = 0
    while index < len(governance):
        if governance[index].startswith("|"):
            cells = split_row(governance[index])
            if len(cells) >= 2 and cells[0] == "Crate" and "ependencies" in cells[1]:
                body = index + 2
                while body < len(governance) and governance[body].startswith("|"):
                    row = split_row(governance[body])
                    if len(row) >= 2 and row[0] in CRATES:
                        crate = CRATES[row[0]]
                        invented = in_workspace_deps(row[1], exclude=(row[0],)) - crate["deps"]
                        if invented:
                            fail("8", "architecture/tooling-governance.md:%d: %s claims dependency "
                                    "%s, which %s/Cargo.toml does not declare"
                                 % (body + 1, row[0], ", ".join(sorted(invented)), crate["dir"]))
                    body += 1
                index = body
                continue
        index += 1

if os.path.isfile("architecture/overview.md"):
    module = re.compile(r"^\d+\.\s+\*\*`?(eggup-[a-z]+(?:-[a-z]+)*)`?\*\*\s*(?:→|->)\s*(.*)$")
    for number, line in enumerate(lines("architecture/overview.md"), 1):
        match = module.match(line.strip())
        if not match:
            continue
        subject = match.group(1)
        crate = CRATES.get(subject)
        if crate is None:
            continue
        invented = in_workspace_deps(match.group(2), exclude=(subject,)) - crate["deps"]
        if invented:
            fail("8", "architecture/overview.md:%d: %s claims dependency %s, which %s/Cargo.toml "
                    "does not declare" % (number, subject, ", ".join(sorted(invented)), crate["dir"]))

# ---------------------------------------------------------------- assertion 9
# A tracked invariant has exactly one true polarity. A surface that states the
# opposite polarity is false wherever it lives -- including in the deep dive
# that owns the invariant, which is why this is separate from assertion 3.
POLARITY_RULES = (
    ("escape hatch", ("no ", "not ", "never ", "without ", "n't "),
     "a member declared NotRequired always fails; there is no escape hatch"),
)
for path in ALL_SURFACES + ["CHANGELOG.md"]:
    if not os.path.isfile(path):
        continue
    full = read(path)
    for phrase, negators, statement in POLARITY_RULES:
        for match in re.finditer(re.escape(phrase), full, re.I):
            window = full[max(0, match.start() - 40):match.start()].lower()
            if not any(negator in window for negator in negators):
                number = full.count("\n", 0, match.start()) + 1
                fail("9", "%s:%d: states the opposite of a tracked invariant: %s"
                     % (path, number, statement))

# ---------------------------------------------------------------------- report
for detail in notes:
    print("note %s" % detail)
if failures:
    for failure in failures:
        print("FAIL %s" % failure)
    print("documentation-truth checks failed: %d" % len(failures))
    sys.exit(1)
print("documentation-truth checks passed")
PY
