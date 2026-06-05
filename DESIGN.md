# Kaku Static Website Design

## Visual Theme

Kaku shares the Kami parchment system used across the family sites (mole.fit and kin) so the apps read as one series. Warm parchment page surface, ink-blue accents, serif type, restrained hairline rules, and dark terminal/screenshot surfaces only where the product needs them. Avoid decorative grids, heavy shadows, oversized marketing type, and card-heavy composition.

## Palette

- `#f5f4ed` parchment page background (`--paper`)
- `#faf9f5` elevated and card surface (`--paper-elev` / `--paper-card`)
- `#141413` primary ink text (`--ink`); `#3d3d3a` dim, `#6b6a64` muted
- `#1B365D` ink-blue brand and actions (`--brand`); `#2D5A8A` hover (`--brand-light`); `#EEF2F7` tint
- `#0d7d4d` green for command-success accents only
- `#141318` terminal canvas (`--term`); `#e7e4d9` terminal text
- `#d8d5c8` warm line/divider (`--line`); `#e5e3d8` soft line (`--line-soft`)

## Typography

Serif-first, matching the family. English pages use Charter with Georgia / Palatino fallbacks; Chinese pages use `TsangerJinKai02`, `Source Han Serif SC`, then `Songti SC`. Body and headings share the serif. System sans (`--ui`) is reserved for small uppercase labels; monospace (`JetBrains Mono`) is for terminal prompts, code snippets, and the version tag. Do not add a font CDN by default.

## Components

Buttons are pill-shaped (999px radius), at least 44px tall, with a filled ink-blue primary or an ink-blue outline secondary. Cards are parchment blocks with hairline borders and no decorative shadows. Terminal windows keep dark chrome but no extra visual effects.

## Layout

The chrome aligns with the mole-mac (Kami) site so the two feel like one series:

- A top eyebrow strip, not a sticky nav bar: wordmark plus version (mono) on the left, Docs plus the language toggle plus GitHub/X icons on the right, in small uppercase serif.
- A left-aligned big-serif hero (`Kaku` with the `書` orbit glyph and a blur-in rise), a serif tagline, and a single dotted trust row in place of stat and tag clutter.
- Sections separated by generous vertical rhythm (96px) rather than divider lines; only the hero keeps a bottom rule.
- A footer that carries the `Kaku · 書` serif wordmark and an italic ethos line. Because Kaku is a multi-page doc site, the footer keeps its Product / Docs / Community / Project link columns rather than collapsing to mole's single-page colophon.

Inner content widths stay close to the blog: about 760px for prose and 1120px for page sections.

## Verification

Verify with a static server, desktop screenshot, 375px mobile screenshot, and link checking across Chinese and English pages before pushing.
