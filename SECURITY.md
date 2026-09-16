# Security policy

## Supported versions

docgraph is pre-1.0 and compatibility is best effort. Security fixes are made
against the latest release; pin the release used by automation and update promptly
when a new release addresses a security issue.

## Reporting a vulnerability

Please report suspected vulnerabilities privately rather than in a public issue.
Use GitHub's
[private vulnerability reporting](https://github.com/JTarasovic/docgraph/security/advisories/new)
for this repository. Include the affected version, a description of the issue, and
the minimal steps needed to reproduce it.

Please do not include working exploit code or a step-by-step extraction path in the
initial report; a description of the class of problem and its impact is enough to
begin triage.

We will acknowledge the report, investigate, and coordinate a fix and disclosure
timeline with you.

## Supply-chain guarantees

Every supported release artifact is published with:

- an adjacent SHA-256 checksum, and
- a producer attestation tying the artifact to this repository's build.

Verify both before running a downloaded build:

```text
gh attestation verify <archive> --repo JTarasovic/docgraph
```

The GitHub Actions installers (`JTarasovic/docgraph/install` and
`install-runtime`) require valid producer attestations and SHA-256 checksums, so CI
consumers get the same guarantee automatically. See the
[README](README.md#security-and-attestations) for the consumer-facing summary.
