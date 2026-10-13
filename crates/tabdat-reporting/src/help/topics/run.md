# run

How to invoke:
`run <script-path>`

What it does:
Execute a TabDat script file in the current session. Interactive runs display successful
leaf-command results in execution order, including nested scripts, then report the total
executed-command count for that run. On error, prior results and completed state changes remain; the error identifies
the script path/line and the shell remains usable. Scripts are sequential, not atomic.
An interactive output failure is fatal and prevents subsequent data/file operations; it does not roll back
an already completed command.

What problem it answers:
How do I replay a workflow deterministically?

Examples:
- `run analysis.td`

For batch or script execution, `tabdat --json -f analysis.td` emits one compact versioned JSON
result envelope per successful command. JSON mode suppresses script metadata and command echoes;
failures also emit one error envelope with a stable type/message and script location when available;
interactive shell sessions remain terminal-only. Use `tabdat --json --list-commands` for a single
sorted command catalog without starting a session or reading data; discovery cannot be combined with
command or script execution. Use `tabdat --json --help-topic summarize` to retrieve one existing
help topic as a single JSON envelope; topic names are case-insensitive and unknown topics emit the
existing JSON error envelope. Use `tabdat --json --explain -c "summarize age"` for a syntax-only
preview that reports the normalized command name with `execution: "not_run"`; exactly one `-c` is
required and the command is not executed. Standard `--help` keeps argparse help precedence.
Use `tabdat --json --list-command-effects` for the finite declared command-level effect catalog; it
does not start a session, read data, or plan execution. Categories are ordered `read`, `write`,
`control`, `plot`, `unknown` and include possible delegated/output effects.
