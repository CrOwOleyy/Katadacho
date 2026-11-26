# Security Policy

## Supported versions

Katadacho is in early development. Only the latest code on the `main` branch receives security fixes.

## Reporting a vulnerability

Please report vulnerabilities privately, not in a public issue.

1. On the repository page, open the **Security** tab and choose **Report a vulnerability**.
2. Describe the issue, the affected platform, and the steps to reproduce it.

If private reporting is not available, open an issue that only asks for a private contact, without any
details about the vulnerability.

You should get a first answer within a week. Once a fix is ready, you will be credited in the release
notes unless you prefer to stay anonymous.

## Scope

Areas that deserve particular care:

- **Clipboard history:** what is stored, and how secrets and password manager copies are excluded.
- **The Claude Desktop bridge:** the local server listens on `127.0.0.1` and requires a secret token.
- **Elevated actions:** the deep RAM cleaner asks for administrator rights on Windows, and uses `pkexec`
  on Linux.
