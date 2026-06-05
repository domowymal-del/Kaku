#!/usr/bin/env python3
"""Build-time syntax highlighting for docs code blocks.

Walks the docs HTML, finds <pre><code class="language-X">...</code></pre>
blocks, and rewrites their inner text with compact <span class="tok-*">
tokens via Pygments. Zero runtime JS is shipped: the spans are baked in.

Idempotent and re-runnable: existing tok-* spans are stripped before each
pass, so the source of truth stays the plain code and the script can run
again after any doc edit.

Usage:
    python3 scripts/highlight.py            # rewrite in place
    python3 scripts/highlight.py --check    # fail if anything is unhighlighted
"""
from __future__ import annotations

import html
import re
import sys
from pathlib import Path

from pygments.lexers import get_lexer_by_name
from pygments.util import ClassNotFound
from pygments.token import Comment, Keyword, Name, Number, String

ROOT = Path(__file__).resolve().parent.parent
DOC_DIRS = [ROOT / "docs", ROOT / "zh" / "docs"]

# Site language alias -> Pygments lexer name.
LEXER_ALIASES = {"zsh": "bash", "fish": "fish", "applescript": "applescript"}

BLOCK_RE = re.compile(
    r'(<pre><code class="language-)([a-z]+)(">)(.*?)(</code></pre>)', re.DOTALL
)
SPAN_RE = re.compile(r'</?span[^>]*>')


def token_class(ttype) -> str | None:
    """Collapse Pygments token types into a small, restrained palette."""
    if ttype in Comment:
        return "c"
    if ttype in Keyword.Constant:
        return "b"
    if ttype in Keyword:
        return "k"
    if ttype in String:
        return "s"
    if ttype in Number:
        return "n"
    if ttype in Name.Function:
        return "f"
    if ttype in Name.Builtin or ttype in Name.Constant:
        return "b"
    return None


def esc(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def highlight(code: str, lang: str) -> str:
    name = LEXER_ALIASES.get(lang, lang)
    try:
        lexer = get_lexer_by_name(name)
    except ClassNotFound:
        lexer = get_lexer_by_name("text")
    # Coalesce consecutive tokens sharing a class to keep the markup compact.
    runs: list[tuple[str | None, str]] = []
    for ttype, value in lexer.get_tokens(code):
        if not value:
            continue
        cls = token_class(ttype)
        if runs and runs[-1][0] == cls:
            runs[-1] = (cls, runs[-1][1] + value)
        else:
            runs.append((cls, value))
    out = [
        f'<span class="tok-{cls}">{esc(text)}</span>' if cls else esc(text)
        for cls, text in runs
    ]
    return "".join(out).rstrip("\n")


def render_block(m: re.Match) -> str:
    pre, lang, mid, inner, tail = m.groups()
    plain = html.unescape(SPAN_RE.sub("", inner)).rstrip("\n")
    return f"{pre}{lang}{mid}{highlight(plain, lang)}{tail}"


def process(check: bool) -> int:
    files = sorted(f for d in DOC_DIRS for f in d.rglob("*.html"))
    changed = 0
    blocks = 0
    for path in files:
        text = path.read_text(encoding="utf-8")
        new, n = BLOCK_RE.subn(render_block, text)
        blocks += n
        if new != text:
            if check:
                print(f"unhighlighted: {path.relative_to(ROOT)}")
                changed += 1
            else:
                path.write_text(new, encoding="utf-8")
                changed += 1
    if check:
        print(f"{blocks} blocks checked, {changed} file(s) need highlighting")
        return 1 if changed else 0
    print(f"highlighted {blocks} blocks across {changed} file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(process("--check" in sys.argv))
