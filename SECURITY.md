# Security Policy

## Reporting Vulnerabilities

Report security vulnerabilities to **security@agentralabs.tech**.

**DO NOT** open public issues for security vulnerabilities.

## Responsible Disclosure

We follow a responsible disclosure process:

1. Report the vulnerability via email to security@agentralabs.tech
2. Include a clear description, reproduction steps, and impact assessment
3. We will acknowledge receipt within 48 hours
4. We will provide an initial assessment within 7 business days
5. We will coordinate a fix and disclosure timeline with you

## Scope

We care about:

- **Validation bypass** -- circumventing streaming validation to produce unvalidated output
- **Shadow execution escape** -- sandbox breakout during shadow code execution
- **Prompt injection bypass** -- injection attacks evading detection
- **MCP server sandbox bypasses** -- unauthorized access to system resources via MCP tools
- **PII leak** -- PII detection failing to catch sensitive data in output
- **Memory safety** -- buffer overflows, use-after-free, or other memory corruption
- **Authentication bypass** -- circumventing AGENTIC_TOKEN auth on server profiles
- **Rate limiter bypass** -- circumventing rate limiting protections

## Out of Scope

- Issues in dependencies (report directly to the dependency maintainer)
- Social engineering attacks
- Denial of service via expected high-load scenarios

## Supported Versions

| Version | Supported |
|---|---|
| 0.1.x | Yes |
| < 0.1.0 | No |
