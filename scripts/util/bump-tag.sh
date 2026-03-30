#!/usr/bin/env bash
# Bumps the latest path-be-v* tag and optionally pushes it.
#
# Usage: bump-tag.sh [COMPONENT] [--push]
#
# COMPONENT (default: patch):
#   major        — bump major, reset minor/patch           e.g. v1.2.3 → v2.0.0
#   minor        — bump minor, reset patch                 e.g. v1.2.3 → v1.3.0
#   patch        — bump patch                              e.g. v1.2.3 → v1.2.4
#   alpha        — bump alpha patch (or start alpha0.0.1)  e.g. v1.2.3-alpha0.0.1 → v1.2.3-alpha0.0.2
#   alpha-minor  — bump alpha minor, reset alpha patch     e.g. v1.2.3-alpha0.0.1 → v1.2.3-alpha0.1.0
#   alpha-major  — bump alpha major, reset alpha minor/patch
#
# Bumping major/minor/patch always drops any alpha suffix (promotes to stable).
#
# --push: create and push the tag to origin in one step.

set -euo pipefail

BUMP="patch"
PUSH=false

for arg in "$@"; do
    case "$arg" in
        --push) PUSH=true ;;
        major|minor|patch|alpha|alpha-major|alpha-minor) BUMP="$arg" ;;
        *) echo "Unknown argument: $arg" >&2; exit 1 ;;
    esac
done

# ── Find latest tag ──────────────────────────────────────────────────────────

LATEST=$(git tag --list "path-be-v*" --sort=-version:refname | head -1)

if [[ -z "$LATEST" ]]; then
    NEW_TAG="path-be-v0.0.1"
    echo "No existing path-be-v* tags found — starting at $NEW_TAG"
else
    echo "Latest: $LATEST"
    VERSION="${LATEST#path-be-v}"

    # Parse stable or alpha variant
    # Accepts both: -alpha1.2.3 and -alpha-1.2.3
    if [[ "$VERSION" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)-alpha-?([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
        MAJOR="${BASH_REMATCH[1]}"
        MINOR="${BASH_REMATCH[2]}"
        PATCH="${BASH_REMATCH[3]}"
        AM="${BASH_REMATCH[4]}"
        AN="${BASH_REMATCH[5]}"
        AP="${BASH_REMATCH[6]}"
        IS_ALPHA=true
    elif [[ "$VERSION" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
        MAJOR="${BASH_REMATCH[1]}"
        MINOR="${BASH_REMATCH[2]}"
        PATCH="${BASH_REMATCH[3]}"
        AM=0; AN=0; AP=0
        IS_ALPHA=false
    else
        echo "error: cannot parse version '$VERSION'" >&2
        exit 1
    fi

    # ── Compute new tag ──────────────────────────────────────────────────────

    case "$BUMP" in
        major)
            NEW_TAG="path-be-v$((MAJOR+1)).0.0"
            ;;
        minor)
            NEW_TAG="path-be-v${MAJOR}.$((MINOR+1)).0"
            ;;
        patch)
            NEW_TAG="path-be-v${MAJOR}.${MINOR}.$((PATCH+1))"
            ;;
        alpha)
            if [[ "$IS_ALPHA" == true ]]; then
                AP=$((AP+1))
            else
                AM=0; AN=0; AP=1
            fi
            NEW_TAG="path-be-v${MAJOR}.${MINOR}.${PATCH}-alpha${AM}.${AN}.${AP}"
            ;;
        alpha-minor)
            if [[ "$IS_ALPHA" == true ]]; then
                AN=$((AN+1)); AP=0
            else
                AM=0; AN=1; AP=0
            fi
            NEW_TAG="path-be-v${MAJOR}.${MINOR}.${PATCH}-alpha${AM}.${AN}.${AP}"
            ;;
        alpha-major)
            if [[ "$IS_ALPHA" == true ]]; then
                AM=$((AM+1)); AN=0; AP=0
            else
                AM=1; AN=0; AP=0
            fi
            NEW_TAG="path-be-v${MAJOR}.${MINOR}.${PATCH}-alpha${AM}.${AN}.${AP}"
            ;;
    esac
fi

echo "New tag: $NEW_TAG"
git tag "$NEW_TAG"

if [[ "$PUSH" == true ]]; then
    git push origin "$NEW_TAG"
    echo "Pushed $NEW_TAG"
else
    echo "Run 'git push origin $NEW_TAG' to trigger CI, or rerun with --push"
fi
