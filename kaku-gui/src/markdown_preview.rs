use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn selection_text_for_preview(text: &str) -> Option<&str> {
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn open_selection_preview(text: &str) -> Result<PathBuf> {
    let source = selection_text_for_preview(text).context("no selected text")?;
    let path = write_preview_file(source)?;
    open_preview_file(&path)?;
    Ok(path)
}

fn write_preview_file(source: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join("kaku-markdown-preview");
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    restrict_dir_permissions(&dir);

    let path = dir.join(format!("preview-{}.html", preview_file_suffix()));
    fs::write(&path, render_preview_html(source))
        .with_context(|| format!("write {}", path.display()))?;
    restrict_file_permissions(&path);
    Ok(path)
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) {}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) {}

fn preview_file_suffix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}-{millis}", std::process::id())
}

fn open_preview_file(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(path);
        command
    };

    #[cfg(not(target_os = "macos"))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };

    command
        .spawn()
        .with_context(|| format!("open {}", path.display()))?;
    Ok(())
}

pub fn render_preview_html(source: &str) -> String {
    let source_json = json_for_script(source);
    let rendered_json = json_for_script(&markdown_to_html(source));
    let escaped_source = escape_html(source);
    let mathjax_script = if contains_math_delimiter(source) {
        r#"
<script>
window.MathJax = {
  startup: { typeset: false },
  tex: {
    inlineMath: [['$', '$'], ['\\(', '\\)']],
    displayMath: [['$$', '$$'], ['\\[', '\\]']]
  }
};
</script>
<script defer src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-chtml.js"></script>
"#
    } else {
        ""
    };

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kaku Markdown Preview</title>
<style>
:root {{
  color-scheme: light dark;
  --bg: #f7f7f4;
  --fg: #1d1d1f;
  --muted: #686868;
  --border: rgba(0, 0, 0, 0.12);
  --code: rgba(0, 0, 0, 0.06);
}}
@media (prefers-color-scheme: dark) {{
  :root {{
    --bg: #171717;
    --fg: #f4f2ec;
    --muted: #aaa59a;
    --border: rgba(255, 255, 255, 0.16);
    --code: rgba(255, 255, 255, 0.08);
  }}
}}
body {{
  margin: 0;
  background: var(--bg);
  color: var(--fg);
  font: 16px/1.58 -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Arial, sans-serif;
}}
main {{
  box-sizing: border-box;
  max-width: 880px;
  min-height: 100vh;
  margin: 0 auto;
  padding: 40px 28px 56px;
}}
#preview > :first-child {{
  margin-top: 0;
}}
h1, h2, h3 {{
  line-height: 1.2;
  margin: 1.5em 0 0.55em;
}}
p, ul, ol, blockquote, pre, table {{
  margin: 0 0 1em;
}}
code, pre {{
  font-family: "SF Mono", Menlo, Consolas, monospace;
}}
pre {{
  overflow: auto;
  padding: 14px;
  border-radius: 8px;
  background: var(--code);
}}
code {{
  padding: 0.12em 0.3em;
  border-radius: 4px;
  background: var(--code);
}}
pre code {{
  padding: 0;
  background: transparent;
}}
blockquote {{
  padding-left: 1em;
  border-left: 3px solid var(--border);
  color: var(--muted);
}}
table {{
  width: 100%;
  border-collapse: collapse;
}}
th, td {{
  padding: 8px 10px;
  border: 1px solid var(--border);
}}
details {{
  margin-top: 32px;
  color: var(--muted);
}}
details pre {{
  color: var(--fg);
  white-space: pre-wrap;
}}
.error {{
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--fg);
}}
</style>
</head>
<body>
<main>
<noscript><pre>{escaped_source}</pre></noscript>
<article id="preview"></article>
<details>
  <summary>Source</summary>
  <pre id="source"></pre>
</details>
</main>
{mathjax_script}
<script>
const source = {source_json};
const renderedHtml = {rendered_json};
const preview = document.getElementById('preview');
const sourceNode = document.getElementById('source');
sourceNode.textContent = source;

function escapeHtml(value) {{
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}}

function renderFallback(message) {{
  preview.innerHTML =
    '<div class="error">' + escapeHtml(message) + '</div><pre>' + escapeHtml(source) + '</pre>';
}}

window.addEventListener('load', async () => {{
  try {{
    preview.innerHTML = renderedHtml;
    if (window.MathJax && window.MathJax.typesetPromise) {{
      await window.MathJax.typesetPromise([preview]);
    }} else if ({has_math}) {{
      renderFallback('Math renderer failed to load. Showing the original text.');
    }}
  }} catch (error) {{
    renderFallback('Preview failed. Showing the original text.');
  }}
}});
</script>
</body>
</html>
"#,
        has_math = contains_math_delimiter(source)
    )
}

fn json_for_script(source: &str) -> String {
    serde_json::to_string(source)
        .expect("serializing a string to JSON cannot fail")
        .replace("</", "<\\/")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

pub fn markdown_to_html(source: &str) -> String {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum ListKind {
        Ordered,
        Unordered,
    }

    fn flush_paragraph(out: &mut String, paragraph: &mut Vec<String>) {
        if paragraph.is_empty() {
            return;
        }
        out.push_str("<p>");
        out.push_str(&render_inline_markdown(&paragraph.join(" ")));
        out.push_str("</p>\n");
        paragraph.clear();
    }

    fn close_list(out: &mut String, list: &mut Option<ListKind>) {
        match list.take() {
            Some(ListKind::Ordered) => out.push_str("</ol>\n"),
            Some(ListKind::Unordered) => out.push_str("</ul>\n"),
            None => {}
        }
    }

    fn open_list(out: &mut String, list: &mut Option<ListKind>, kind: ListKind) {
        if *list == Some(kind) {
            return;
        }
        close_list(out, list);
        match kind {
            ListKind::Ordered => out.push_str("<ol>\n"),
            ListKind::Unordered => out.push_str("<ul>\n"),
        }
        *list = Some(kind);
    }

    let mut out = String::new();
    let mut paragraph = Vec::new();
    let mut list = None;
    let mut code_fence: Option<String> = None;

    for raw_line in source.lines() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim();

        if code_fence.is_some() {
            if trimmed.starts_with("```") {
                out.push_str("</code></pre>\n");
                code_fence = None;
            } else {
                out.push_str(&escape_html(line));
                out.push('\n');
            }
            continue;
        }

        if let Some(lang) = trimmed.strip_prefix("```") {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            let lang = lang.trim();
            if lang.is_empty() {
                out.push_str("<pre><code>");
            } else {
                out.push_str("<pre><code class=\"language-");
                out.push_str(&escape_html(lang));
                out.push_str("\">");
            }
            code_fence = Some(lang.to_string());
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            continue;
        }

        if let Some((level, text)) = heading(trimmed) {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            out.push_str(&format!(
                "<h{level}>{}</h{level}>\n",
                render_inline_markdown(text)
            ));
            continue;
        }

        if let Some(text) = trimmed.strip_prefix("> ") {
            flush_paragraph(&mut out, &mut paragraph);
            close_list(&mut out, &mut list);
            out.push_str("<blockquote><p>");
            out.push_str(&render_inline_markdown(text));
            out.push_str("</p></blockquote>\n");
            continue;
        }

        if let Some(text) = unordered_list_item(trimmed) {
            flush_paragraph(&mut out, &mut paragraph);
            open_list(&mut out, &mut list, ListKind::Unordered);
            out.push_str("<li>");
            out.push_str(&render_inline_markdown(text));
            out.push_str("</li>\n");
            continue;
        }

        if let Some(text) = ordered_list_item(trimmed) {
            flush_paragraph(&mut out, &mut paragraph);
            open_list(&mut out, &mut list, ListKind::Ordered);
            out.push_str("<li>");
            out.push_str(&render_inline_markdown(text));
            out.push_str("</li>\n");
            continue;
        }

        close_list(&mut out, &mut list);
        paragraph.push(trimmed.to_string());
    }

    if code_fence.is_some() {
        out.push_str("</code></pre>\n");
    }
    flush_paragraph(&mut out, &mut paragraph);
    close_list(&mut out, &mut list);

    if out.is_empty() {
        "<p></p>\n".to_string()
    } else {
        out
    }
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = line.get(level..)?;
    rest.strip_prefix(' ').map(|text| (level, text.trim()))
}

fn unordered_list_item(line: &str) -> Option<&str> {
    ["- ", "* ", "+ "]
        .iter()
        .find_map(|prefix| line.strip_prefix(prefix))
}

fn ordered_list_item(line: &str) -> Option<&str> {
    let (digits, rest) = line.split_at(line.chars().take_while(|ch| ch.is_ascii_digit()).count());
    if digits.is_empty() {
        return None;
    }
    rest.strip_prefix(". ")
}

fn render_inline_markdown(source: &str) -> String {
    let mut out = String::new();
    let mut rest = source;
    while let Some(start) = rest.find('`') {
        let (before, after_start) = rest.split_at(start);
        out.push_str(&render_emphasis(&escape_html(before)));

        let after_start = &after_start[1..];
        if let Some(end) = after_start.find('`') {
            let (code, after_end) = after_start.split_at(end);
            out.push_str("<code>");
            out.push_str(&escape_html(code));
            out.push_str("</code>");
            rest = &after_end[1..];
        } else {
            out.push('`');
            rest = after_start;
            break;
        }
    }
    out.push_str(&render_emphasis(&escape_html(rest)));
    out
}

fn render_emphasis(escaped: &str) -> String {
    replace_inline_pair(
        &replace_inline_pair(escaped, "**", "<strong>", "</strong>"),
        "*",
        "<em>",
        "</em>",
    )
}

fn replace_inline_pair(source: &str, marker: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = source;
    let mut open_pair = false;

    while let Some(pos) = rest.find(marker) {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + marker.len()..];
        if after.find(marker).is_none() && !open_pair {
            out.push_str(marker);
        } else if open_pair {
            out.push_str(close);
            open_pair = false;
        } else {
            out.push_str(open);
            open_pair = true;
        }
        rest = after;
    }

    out.push_str(rest);
    if open_pair {
        out.push_str(close);
    }
    out
}

pub fn escape_html(source: &str) -> String {
    source
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn contains_math_delimiter(source: &str) -> bool {
    has_unescaped_pair(source, "$$")
        || has_bracket_math(source)
        || unescaped_single_dollar_count(source) >= 2
}

fn has_bracket_math(source: &str) -> bool {
    source.contains("\\[") && source.contains("\\]")
}

fn has_unescaped_pair(source: &str, delimiter: &str) -> bool {
    let mut matches = 0;
    let mut index = 0;
    while let Some(offset) = source[index..].find(delimiter) {
        let absolute = index + offset;
        if !is_escaped_at(source, absolute) {
            matches += 1;
            if matches >= 2 {
                return true;
            }
        }
        index = absolute + delimiter.len();
    }
    false
}

fn unescaped_single_dollar_count(source: &str) -> usize {
    source
        .char_indices()
        .filter(|(idx, ch)| {
            if *ch != '$' || is_escaped_at(source, *idx) {
                return false;
            }
            let before = source[..*idx].chars().next_back();
            let after = source[*idx + 1..].chars().next();
            before != Some('$') && after != Some('$')
        })
        .count()
}

fn is_escaped_at(source: &str, byte_index: usize) -> bool {
    let mut slash_count = 0;
    for ch in source[..byte_index].chars().rev() {
        if ch == '\\' {
            slash_count += 1;
        } else {
            break;
        }
    }
    slash_count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{
        contains_math_delimiter, escape_html, markdown_to_html, render_preview_html,
        selection_text_for_preview,
    };

    #[test]
    fn missing_selection_returns_none() {
        assert_eq!(selection_text_for_preview(" \n\t"), None);
        assert_eq!(selection_text_for_preview("hello"), Some("hello"));
    }

    #[test]
    fn preview_html_escapes_fallback_source() {
        let html = render_preview_html("<script>alert('x')</script> & text");
        assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt; &amp; text"));
    }

    #[test]
    fn preview_html_escapes_script_breakout() {
        let html = render_preview_html("</script><script>alert(1)</script>");
        assert!(html.contains("<\\/script><script>alert(1)<\\/script>"));
        assert!(!html.contains("const source = \"</script><script>alert(1)</script>\";"));
    }

    #[test]
    fn html_escape_covers_special_characters() {
        assert_eq!(
            escape_html("<tag a=\"b\">Tom & 'Jerry'</tag>"),
            "&lt;tag a=&quot;b&quot;&gt;Tom &amp; &#39;Jerry&#39;&lt;/tag&gt;"
        );
    }

    #[test]
    fn detects_markdown_math_delimiters() {
        assert!(contains_math_delimiter("inline $x + y$ math"));
        assert!(contains_math_delimiter("block\n$$\nx^2\n$$"));
        assert!(contains_math_delimiter("\\[x^2\\]"));
        assert!(!contains_math_delimiter("one price is $5 only"));
        assert!(!contains_math_delimiter(r"escaped \$x\$ only"));
    }

    #[test]
    fn renders_basic_markdown_locally() {
        let html = markdown_to_html("# Title\n\n- **one**\n- `two`\n\n```sh\necho <ok>\n```");

        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<li><strong>one</strong></li>"));
        assert!(html.contains("<li><code>two</code></li>"));
        assert!(html.contains("echo &lt;ok&gt;"));
    }

    #[test]
    fn preview_without_math_uses_no_remote_renderer() {
        let html = render_preview_html("# Local only");

        assert!(!html.contains("marked.min.js"));
        assert!(!html.contains("mathjax@3"));
    }

    #[test]
    fn preview_with_math_loads_mathjax() {
        let html = render_preview_html("Area is $x^2$.");

        assert!(html.contains("mathjax@3"));
    }
}
