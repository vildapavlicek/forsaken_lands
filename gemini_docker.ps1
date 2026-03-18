docker run --rm -it `
  -v "${env:USERPROFILE}\.gemini:/root/.gemini" `
  -v "${PWD}:/workspace" `
  -e TERM="xterm-256color" `
  -e FORCE_COLOR="3" `
  -e COLORTERM="truecolor" `
  gemini-rust-sandbox
