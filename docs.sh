#!/bin/sh
# Serve, build and deploy the OpenInterceptor implementation site (the `docs` branch).
#
#   ./docs.sh serve    live preview at http://127.0.0.1:8000 (PORT=… to change)
#   ./docs.sh build    strict build into site/
#   ./docs.sh deploy   strict build, then publish to the forge's pages branch
#                      (on GitHub this is automatic: pushing the docs branch runs .github/workflows/pages.yml,

#   ./docs.sh help
#
# The toolchain lives in .venv/ beside this script, created on first use from
# requirements-docs.txt. Set MKDOCS=/path/to/mkdocs to use another install.
# Deploy follows the publish-pages rules: the forge in the main branch's
# .specitout/sdd.json picks the pages branch (github → gh-pages, gitea/forgejo → pages).
set -eu
cd "$(dirname "$0")"

venv_mkdocs() {
    if [ -n "${MKDOCS:-}" ]; then echo "$MKDOCS"; return; fi
    if [ ! -x .venv/bin/mkdocs ]; then
        echo "docs.sh: creating .venv and installing requirements-docs.txt" >&2
        python3 -m venv .venv
        .venv/bin/pip install --quiet --upgrade pip
        .venv/bin/pip install --quiet -r requirements-docs.txt
    fi
    echo .venv/bin/mkdocs
}

forge() {
    if [ -n "${PAGES_FORGE:-}" ]; then echo "$PAGES_FORGE"; return; fi
    # The manifest is on main; read it from git without switching branches.
    git show main:.specitout/sdd.json 2>/dev/null \
        | python3 -c 'import json,sys; print(json.load(sys.stdin).get("forge","none"))' 2>/dev/null \
        || echo none
}

cmd="${1:-help}"
[ $# -gt 0 ] && shift
case "$cmd" in
    serve)
        mk=$(venv_mkdocs)
        exec "$mk" serve -a "127.0.0.1:${PORT:-8000}" "$@"
        ;;
    build)
        mk=$(venv_mkdocs)
        exec "$mk" build --strict "$@"
        ;;
    deploy)
        mk=$(venv_mkdocs)
        "$mk" build --strict
        if ! git remote get-url origin >/dev/null 2>&1; then
            echo "docs.sh: no 'origin' remote is configured, so nothing was published."
            echo "docs.sh: the strict build is in site/. Add a remote, then run ./docs.sh deploy again."
            exit 0
        fi
        if [ -n "$(git status --porcelain -- docs mkdocs.yml requirements-docs.txt)" ]; then
            echo "docs.sh: warning: uncommitted documentation changes will be published" >&2
        fi
        case "$(forge)" in
            gitea|forgejo) branch=pages ;;
            *) branch=gh-pages ;;
        esac
        exec "$mk" gh-deploy --force --remote-branch "$branch" "$@"
        ;;
    help|-h|--help)
        sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
        ;;
    *)
        echo "docs.sh: unknown command '$cmd' (try ./docs.sh help)" >&2
        exit 2
        ;;
esac
