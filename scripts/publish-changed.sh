#!/usr/bin/env bash
# publish-changed: publish-on-version-change for cargo workspaces.
#
# For each given workspace root: read the workspace's publishable crates
# from `cargo metadata` (respecting each crate's `publish` field and
# `registry` fields — never a hand-maintained list), compute their
# topological dependency order (deps before dependents — the coldsteel
# registry rejects a crate whose in-workspace dep versions aren't present
# yet), and for each crate compare its Cargo.toml version against the
# coldsteel kellnr sparse index. A version already on the index is skipped;
# an absent version is reported (dry run, the default) or published
# (--publish).
#
# This is the version-diff mechanism, not a path/tag trigger: it is safe to
# run on every push to master, including pushes that touch nothing
# publishable, and a missed or failed publish self-heals on the next run
# instead of being lost.
#
# Publishing must be opted into explicitly, and only to this registry: every
# crate whose manifest is not `publish = false` must say
# `publish = ["coldsteel"]` exactly. An absent `publish` field — and the
# equivalent `publish = true` — means "publishable to crates.io", so a
# workspace root added to this job would carry every such crate onto the
# registry by default. That is a hard error here, not a skip: see
# `docs/agent_guide/publishing.md`.
#
# Requires on PATH: cargo (the workspace's own toolchain — run this inside
# that workspace's devenv shell so `cargo publish`'s verify build has the
# right native deps), curl, jq.
#
# Usage:
#   publish-changed.sh [--publish] [--] <workspace-root> [<workspace-root> ...]
#
# Options:
#   --publish   Actually run `cargo publish` for crates found absent from
#               the index. Requires CARGO_REGISTRIES_COLDSTEEL_TOKEN.
#               Without this flag the script only reports what it would do
#               (dry run) and never touches the registry beyond reading
#               the sparse index.
#   -h, --help  Show this help.
#
# A publish failure aborts the script immediately (no silent skip) — the
# job that runs this must go red, because a red publish job means every
# downstream CI build resolving these crates is about to be wrong.
#
# Exit status: 0 if every configured workspace was processed without a
# publish error; 1 on any hard failure (bad workspace, registry
# unreachable, publish failed, dependency cycle among publishable crates,
# a crate that is publishable without an explicit `publish = ["coldsteel"]`).

set -euo pipefail

REGISTRY_NAME="coldsteel"
INDEX_BASE="https://cargo.coldsteel.casa/api/v1/crates"

usage() {
    sed -n '2,40p' "$0" | sed 's/^# \{0,1\}//'
}

publish=0
roots=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --publish)
            publish=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            break
            ;;
        -*)
            echo "Error: Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

roots+=("$@")

if [[ ${#roots[@]} -eq 0 ]]; then
    echo "Error: at least one workspace root is required." >&2
    usage >&2
    exit 1
fi

if [[ "$publish" -eq 1 && -z "${CARGO_REGISTRIES_COLDSTEEL_TOKEN:-}" ]]; then
    echo "Error: --publish requires CARGO_REGISTRIES_COLDSTEEL_TOKEN to be set." >&2
    exit 1
fi

for bin in cargo curl jq; do
    if ! command -v "$bin" >/dev/null 2>&1; then
        echo "Error: '$bin' not found on PATH." >&2
        exit 1
    fi
done

# Sparse index path for a crate name, per Cargo's registry index format:
# https://doc.rust-lang.org/cargo/reference/registries.html#index-format
index_path() {
    local name="$1" len
    len=${#name}
    case "$len" in
        1) echo "1/$name" ;;
        2) echo "2/$name" ;;
        3) echo "3/${name:0:1}/$name" ;;
        *) echo "${name:0:2}/${name:2:2}/$name" ;;
    esac
}

# version_published NAME VERSION
# Returns success (0) if VERSION is present on the index, failure (1) if
# the crate/version is absent. Hard-exits the whole script on any
# registry-index error other than a clean 404 (unknown crate) — an
# unreachable registry must not be misread as "nothing published yet".
version_published() {
    local name="$1" version="$2" path url status tmp found
    path=$(index_path "$name")
    url="$INDEX_BASE/$path"
    tmp=$(mktemp)

    if ! status=$(curl -sS -o "$tmp" -w '%{http_code}' "$url"); then
        echo "Error: failed to reach registry index for $name: $url" >&2
        rm -f "$tmp"
        exit 1
    fi

    if [[ "$status" == "404" ]]; then
        rm -f "$tmp"
        return 1
    fi

    if [[ "$status" != "200" ]]; then
        echo "Error: unexpected HTTP $status from registry index for $name: $url" >&2
        cat "$tmp" >&2
        rm -f "$tmp"
        exit 1
    fi

    found=$(jq -s --arg v "$version" '[.[] | select(.vers == $v)] | length' "$tmp")
    rm -f "$tmp"
    [[ "$found" -gt 0 ]]
}

# wait_for_index NAME VERSION
# Poll the index after a publish until the new version is visible, so the
# next crate in topological order doesn't try to resolve it too early.
# Hard-exits if it never shows up — a publish that "succeeded" but never
# reached the index is exactly the failure mode this job exists to catch.
wait_for_index() {
    local name="$1" version="$2" attempt
    for ((attempt = 0; attempt < 12; attempt++)); do
        if version_published "$name" "$version"; then
            return 0
        fi
        sleep 5
    done
    echo "Error: $name $version did not appear on the registry index after publishing (timed out)." >&2
    exit 1
}

# process_workspace ROOT
process_workspace() {
    local root="$1"
    local manifest="$root/Cargo.toml"

    echo "=== workspace: $root ==="

    if [[ ! -f "$manifest" ]]; then
        echo "Error: no Cargo.toml at $manifest" >&2
        exit 1
    fi

    local meta
    if ! meta=$(cargo metadata --no-deps --format-version 1 --manifest-path "$manifest"); then
        echo "Error: cargo metadata failed for $root" >&2
        exit 1
    fi

    # Opt-in gate. Publishing is a deliberate, per-crate, single-registry
    # decision, so the only two manifest states this script accepts are
    # `publish = false` (an unpublishable crate — `[]` in cargo metadata)
    # and `publish = ["coldsteel"]` exactly. Everything else is a hard
    # error:
    #
    #   - an absent `publish` field, and the identical-in-metadata
    #     `publish = true`, both mean "publishable to crates.io". That is
    #     cargo's default, which makes the dangerous state the one you get
    #     by writing nothing — a new crate in a configured workspace would
    #     publish itself on the next master push, and a human running a
    #     bare `cargo publish` would send it to the public registry;
    #   - an array naming any other registry, or naming coldsteel among
    #     others, is a crate whose blast radius is wider than this job.
    #
    # No skip, no "publish the coldsteel ones and ignore the rest": a
    # manifest this script cannot vouch for stops the whole workspace, per
    # `docs/agent_guide/no_silent_fallback_values.md`. The fix is always
    # one line in the named crate's Cargo.toml.
    local unvetted
    unvetted=$(jq -r --arg reg "$REGISTRY_NAME" '
        .packages[]
        | select(.publish != [] and .publish != [$reg])
        | "  \(.name) \(.version): publish = " +
          (if .publish == null then "absent (or `true`)" else (.publish | tojson) end)
    ' <<<"$meta")

    if [[ -n "$unvetted" ]]; then
        echo "Error: crate(s) in $root are publishable without an explicit \`publish = [\"$REGISTRY_NAME\"]\`:" >&2
        echo "$unvetted" >&2
        echo "Set \`publish = [\"$REGISTRY_NAME\"]\` to publish it here, or \`publish = false\` to keep it out of every registry." >&2
        exit 1
    fi

    # Publishable crates: exactly those that named this registry above.
    # `publish = false` comes back from cargo metadata as an empty array,
    # which the index() check naturally excludes. Never a hand-maintained
    # crate list — this reads straight off the manifests, same as cargo
    # itself would.
    local names_raw
    names_raw=$(jq -r --arg reg "$REGISTRY_NAME" '
        .packages[]
        | select(.publish | index($reg) != null)
        | .name
    ' <<<"$meta" | sort)

    if [[ -z "$names_raw" ]]; then
        echo "No publishable crates in $root."
        return 0
    fi

    local -a names=()
    while IFS= read -r line; do
        [[ -n "$line" ]] && names+=("$line")
    done <<<"$names_raw"

    # version[name], indeg[name] (in-degree for Kahn's algorithm), and
    # adj[name] (space-separated dependents) — all local to this function
    # call, so processing multiple workspace roots in one invocation never
    # cross-contaminates.
    local -A version=()
    local -A indeg=()
    local -A adj=()

    local n
    for n in "${names[@]}"; do
        version[$n]=$(jq -r --arg n "$n" '.packages[] | select(.name == $n) | .version' <<<"$meta")
        indeg[$n]=0
        adj[$n]=""
    done

    # Edges: for each publishable crate, each in-workspace path-dependency
    # that is itself publishable becomes an edge dep -> dependent. A path
    # dependency on a non-publishable sibling (e.g. an examples crate) is
    # not a publish-ordering concern and is ignored.
    local dep deps_raw
    for n in "${names[@]}"; do
        deps_raw=$(jq -r --arg n "$n" '
            .packages[] | select(.name == $n) | .dependencies[]
            | select(.path != null) | .name
        ' <<<"$meta")
        while IFS= read -r dep; do
            [[ -z "$dep" ]] && continue
            if [[ -n "${version[$dep]+x}" ]]; then
                adj[$dep]="${adj[$dep]} $n"
                indeg[$n]=$((indeg[$n] + 1))
            fi
        done <<<"$deps_raw"
    done

    # Kahn's algorithm. The frontier is re-sorted each pass so the order is
    # deterministic (alphabetical among crates at the same dependency tier)
    # rather than dependent on associative-array iteration order.
    local -a queue=() order=()
    for n in "${names[@]}"; do
        [[ "${indeg[$n]}" -eq 0 ]] && queue+=("$n")
    done

    local cur dependent
    while [[ ${#queue[@]} -gt 0 ]]; do
        mapfile -t queue < <(printf '%s\n' "${queue[@]}" | sort)
        cur="${queue[0]}"
        queue=("${queue[@]:1}")
        order+=("$cur")
        for dependent in ${adj[$cur]}; do
            indeg[$dependent]=$((indeg[$dependent] - 1))
            if [[ "${indeg[$dependent]}" -eq 0 ]]; then
                queue+=("$dependent")
            fi
        done
    done

    if [[ ${#order[@]} -ne ${#names[@]} ]]; then
        echo "Error: dependency cycle detected among publishable crates in $root" >&2
        exit 1
    fi

    local v
    for n in "${order[@]}"; do
        v="${version[$n]}"
        if version_published "$n" "$v"; then
            echo "  [skip]    $n $v (already on registry)"
            continue
        fi

        if [[ "$publish" -eq 0 ]]; then
            echo "  [would publish] $n $v"
            continue
        fi

        echo "  [publish] $n $v"
        if ! (cd "$root" && cargo publish --registry "$REGISTRY_NAME" -p "$n" --locked); then
            echo "Error: cargo publish failed for $n $v in $root" >&2
            exit 1
        fi
        wait_for_index "$n" "$v"
    done
}

for root in "${roots[@]}"; do
    process_workspace "$root"
done

echo "Done."
