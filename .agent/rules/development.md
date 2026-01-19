# Development Rules

## Compilation
To check that the application compiles, use the command:
`cargo c`

**IMPORTANT**:
- We are running on Windows with PowerShell.
- Unix-style command chaining or redirection (e.g., `cargo c &2>1`) will **NOT** work.
- Do not attempt to use Unix shell syntax.

## Formatting
Every task should be finished by formatting the code with:
`cargo +nightly fmt`
