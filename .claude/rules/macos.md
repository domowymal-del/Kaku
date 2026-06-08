# Kaku macOS Platform Rules

> macOS 26 上 AppKit 自动注入是 PAC-fault / 崩溃的高频来源。改 menubar / window handling 前必读本文。

## AppKit Menu 自动注入是地雷

NSApplication 默认会向 Window menu 和 Help menu 注入自己的项（Tile / Move & Resize / Bring All to Front / Help Search），并把 NSWindowRepresentingMenuItem 挂在 Windows menu 下做 dangling reference 。在 macOS 26 上，这些注入项可能 PAC-fault：

- `-[NSMenu _performKeyEquivalentWithDelegate:]` 在 Ctrl+Cmd+F 路由时崩
- Windows menu 关闭窗口后 NSWindowRepresentingMenuItem dangling
- Help Search field 触发 menu 重新计算时崩

## 必须执行的兜底

- 不要调 `setWindowsMenu:` 让 AppKit 接管 Windows menu。Kaku 有自己的 tab/window 切换器。
- 不要调 `setHelpMenu:` 让 AppKit 注入 Help Search。Kaku 没有 help book。
- 窗口创建时调 `setExcludedFromWindowsMenu:` 让 AppKit 看不见我们的 NSWindow，避免它把窗口加到 Windows menu。

参考历史 commit：
- `1ec3823` - 第一次排查 NSWindowRepresentingMenuItem dangling
- `4b15bab` - 进一步 stop AppKit from injecting into Window/Help menus
- `fc2dfa9` - redirect AppKit Spotlight for Help to an orphan menu
- `6ba7381` - restore synchronous menubar init to prevent key event loss

## Menubar 初始化时序

Menubar **必须同步初始化**。延后到 async block 会丢早期按键事件（Cmd+Q 在启动后立刻按下会被吞）。见 `6ba7381`。

## Key Equivalent

新增 KeyAssignment 前确认走 Kaku 自己的 `command_for_key` 路径，不要走 AppKit menu 的 `performKeyEquivalent`。后者会经过被 PAC-fault 的注入项。

### menu keyEquivalent 会拦 keyDown，吞掉 raw-mode TUI 的 Ctrl+letter

macOS 26 上 NSMenu 的 keyEquivalent modifier 匹配并不严格相等。给 menu item 装 `Ctrl+letter` 形式的快捷键，即使带额外 modifier（如 `NSFunctionKeyMask` 想做 `Fn+Ctrl+letter`），仍会拦截 plain `Ctrl+letter` 的 keyDown。后果：

- AppKit 把 keyDown 路由给 menu item action，NSWindow 收不到。
- keyUp 不走 `performKeyEquivalent:`，所以 keyUp 照常进 Kaku。
- `termwiz/src/input.rs` 的 `encode` 对 `is_down=false` 直接返回空字符串，PTY 上一个字节都不会到。
- 普通 shell `cat -v` 测试看到 `^C` 不代表 raw-mode 也工作；启动时机和 cooked vs raw 都会让 menu 拦截表现不一致。**必须**在 raw-mode TUI (claude / codex / vim / htop) 里实测。

兜底：任何"模仿系统快捷键"的 menu 项宁可 `keys: vec![]` 不设快捷键，让用户从菜单点。`kaku-gui/src/commands.rs` 1000 行附近的 keyEquivalent 装配逻辑应该统一从 `candidate[0]` 取 modifiers，不要走 `forced_equiv_mods` 这种特例化路径。

参考历史：`a14b26d` 之后的 CenterWindow 修复（删掉给 Window > Center 装 `Fn+Ctrl+C` keyEquivalent 的特例，因为 macOS 26 把它当 plain Ctrl+C 拦了，导致 raw-mode 下 Ctrl+C 不退 claude/codex）。

### 排查 keyDown 丢失

`config.debug_key_events = true` 重启 Kaku，复现按键，看 `~/.local/share/kaku/kaku-gui-log-<pid>.txt`：

- grep `key_event.*CTRL`。
- 如果只有 `key_is_down: false`、没有 `key_is_down: true`，就是 menu keyEquivalent 拦了 keyDown。
- 这种情况下不要去查 termwiz / PTY / termios，先找 `set_key_equiv_modifier_mask` 装配点。

## 调试方式

- 怀疑 menu 注入相关崩溃时：`defaults read com.apple.dt.Xcode` 找最近 crash log。
- 看 `NSMenu _performKeyEquivalentWithDelegate:` 栈帧是 AppKit 注入项的标志。
- 主要复现环境是一台 macOS 26 实机。
