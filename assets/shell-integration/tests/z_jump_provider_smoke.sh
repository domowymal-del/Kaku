#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "zoxide_jump_provider: starting (zsh=$(command -v zsh 2>/dev/null || echo MISSING), bash=$BASH_VERSION)" >&2

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/kaku-zoxide-jump-provider.XXXXXX")"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

HOME="$tmp_dir/home"
ZDOTDIR="$HOME"
mkdir -p "$HOME"

vendor_dir="$tmp_dir/vendor"
mkdir -p "$vendor_dir/fast-syntax-highlighting" \
         "$vendor_dir/zsh-autosuggestions" \
         "$vendor_dir/zsh-completions"

# Minimal fast-syntax-highlighting stub
cat >"$vendor_dir/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh" <<'EOF'
typeset -g KAKU_TEST_FAST_SH_SOURCED=1
_zsh_highlight() { :; }
EOF

echo "zoxide_jump_provider: running setup_zsh.sh" >&2
setup_out=""
setup_status=0
setup_out="$(
  HOME="$HOME" \
  ZDOTDIR="$ZDOTDIR" \
  KAKU_INIT_INTERNAL=1 \
  KAKU_SKIP_TOOL_BOOTSTRAP=1 \
  KAKU_SKIP_TERMINFO_BOOTSTRAP=1 \
  KAKU_VENDOR_DIR="$vendor_dir" \
  bash "$REPO_ROOT/assets/shell-integration/setup_zsh.sh" --update-only 2>&1
)" || setup_status=$?
if [[ "$setup_status" -ne 0 ]]; then
  echo "zoxide_jump_provider: setup_zsh.sh failed (exit $setup_status):" >&2
  echo "$setup_out" >&2
  exit 1
fi

kaku_zsh="$HOME/.config/kaku/zsh/kaku.zsh"
if [[ ! -f "$kaku_zsh" ]]; then
  echo "zoxide_jump_provider: kaku.zsh not created at $kaku_zsh" >&2
  exit 1
fi

# Test 1: when zoxide is already initialized by user, Kaku must not re-init
with_existing_provider=""
if ! with_existing_provider="$(
  TERM=xterm-256color \
  HOME="$HOME" \
  ZDOTDIR="$ZDOTDIR" \
  zsh -f -c '
# Simulate user having already run zoxide init
__zoxide_z() { :; }
typeset -g KAKU_TEST_ZOXIDE_DOUBLE_INIT=0
eval_orig=$(builtin command -v eval 2>/dev/null || true)
# Track if zoxide init is called again
zoxide() {
  if [[ "${1:-}" == "init" ]]; then
    KAKU_TEST_ZOXIDE_DOUBLE_INIT=1
  fi
}
source "$HOME/.config/kaku/zsh/kaku.zsh"
print -r -- "__KAKU_NO_DOUBLE_INIT__:${KAKU_TEST_ZOXIDE_DOUBLE_INIT}"
' 2>&1
)"; then
  echo "zoxide_jump_provider: zsh with existing provider exited non-zero:" >&2
  echo "$with_existing_provider" >&2
  exit 1
fi

case "$with_existing_provider" in
  *__KAKU_NO_DOUBLE_INIT__:0* ) ;;
  * )
    echo "zoxide_jump_provider: zoxide re-initialized despite existing __zoxide_z provider:" >&2
    echo "$with_existing_provider" >&2
    exit 1
    ;;
esac

# Test 2: when zoxide is not available, no errors should occur (graceful degradation)
without_zoxide=""
if ! without_zoxide="$(
  TERM=xterm-256color \
  HOME="$HOME" \
  ZDOTDIR="$ZDOTDIR" \
  PATH="/usr/bin:/bin" \
  zsh -f -c '
source "$HOME/.config/kaku/zsh/kaku.zsh"
print -r -- "__KAKU_NO_ZOXIDE_OK__:0"
' 2>&1
)"; then
  echo "zoxide_jump_provider: zsh without zoxide exited non-zero:" >&2
  echo "$without_zoxide" >&2
  exit 1
fi

case "$without_zoxide" in
  *__KAKU_NO_ZOXIDE_OK__:0* ) ;;
  * )
    echo "zoxide_jump_provider: kaku.zsh errored when zoxide is absent:" >&2
    echo "$without_zoxide" >&2
    exit 1
    ;;
esac

echo "zoxide_jump_provider smoke test passed"
