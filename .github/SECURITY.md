# Security Policy

## Reporting a Vulnerability

Do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.

Use GitHub Private Vulnerability Reporting for this repository:

1. Open the repository on GitHub.
2. Go to the `Security` tab.
3. Choose `Report a vulnerability`.
4. Include reproduction details, impact, and any known mitigations.

If private vulnerability reporting has not been enabled yet, do not post
exploit details publicly. Wait for the repository maintainers to publish
the private reporting path before sharing sensitive details.

## Scope

The following areas are in scope for security reports:

- local workspace file handling
- markdown preview and diagram rendering
- bundled runtimes and packaged assets
- AI provider integrations
- GitHub Actions and release automation
- dependency and supply-chain risks

## Supported Versions

Katana is pre-1.0. Security support applies to the latest development
state until versioned releases are published.

## Disclosure Expectations

- Allow maintainers time to investigate and remediate before public
  disclosure.
- Avoid including secrets, tokens, or private user data unless they are
  required to reproduce the issue.
- If a report depends on unsafe sample content, provide the smallest
  proof of concept that demonstrates the issue.
