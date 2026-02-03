# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# jlif demo recording
#
# Records a terminal session demonstrating jlif filtering JSON logs
# from a simulated kubectl command.
#
# Usage: vhs demo.tape
# (Prefer using record.sh which handles setup and cleanup)

Output demo.webm
Output demo.mp4

Set Shell "zsh"
Set FontFamily "JetBrainsMono NFM SemiBold"
Set FontSize 28
Set Width 1800
Set Height 600
Set Theme { "name": "Jlif", "black": "#0f0f14", "red": "#ef6487", "green": "#5eca89", "yellow": "#f59e0b", "blue": "#65aef7", "magenta": "#a78bfa", "cyan": "#43c1be", "white": "#e5e5e5", "brightBlack": "#9ca3af", "brightRed": "#ef6487", "brightGreen": "#5eca89", "brightYellow": "#fcd34d", "brightBlue": "#65aef7", "brightMagenta": "#a78bfa", "brightCyan": "#43c1be", "brightWhite": "#ffffff", "background": "#0f0f14", "foreground": "#e5e5e5", "selection": "#2d2620", "cursor": "#f59e0b" }
Set Padding 20

# Hidden setup - prepare environment
Set TypingSpeed 0
Hide
Type "cd /tmp/jlif-demo"
Enter
Sleep 10ms
Type "PROMPT='%F{#f59e0b}> %f'"
Enter
Sleep 10ms
Type "kubectl() { ./mock-logs.sh; }"
Enter
Sleep 10ms
Type "clear"
Enter
Sleep 30ms
Show
Set TypingSpeed 35ms

# == VISIBLE RECORDING STARTS ==

Sleep 500ms

Type "kubectl logs my-service | jlif -f sunshine"
Enter

Sleep 6s
