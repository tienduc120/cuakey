# 0016 - Signed releases through SignPath Foundation

- **Status**: accepted
- **Date**: 2026-11-27

## Context

The release zip holds an unsigned `cuakey.exe`. SmartScreen warns when a downloaded unsigned program is run, and Smart App Control blocks it with no way to allow it. Smart App Control trusts a signature only from a certificate that chains to Microsoft's trusted root program, and it also weighs the program's reputation. A self-signed certificate is trusted only on a PC that imports it, so it does not help a person who downloads a release.

SignPath Foundation signs open source projects for free, with a certificate issued to the foundation. It asks for an OSI-approved licence, a project that is released and maintained, a build made by CI from the repository, multi-factor authentication for the team, a code signing policy on the project's home page, and a manual approval for each release.

## Decision

1. **Releases are signed through SignPath Foundation.** The publisher on the signature is "SignPath Foundation", not the maintainer.
2. **Only `cuakey.exe` is signed**, inside the release workflow, after the build that made it and before the zip is packed. `.signpath/artifact-configuration.xml` states what is signed; the same text is entered in SignPath.
3. **The signing steps run only when the repository variable `SIGNPATH_ORGANIZATION_ID` is set.** Without it the workflow builds and publishes the unsigned zip as before, so a fork or a clone needs nothing. The API token is the repository secret `SIGNPATH_API_TOKEN`; the project slug is `cuakey` and the signing policy slug is `release-signing`.
4. **The workflow refuses a release whose signature does not verify.** It checks `Get-AuthenticodeSignature` is `Valid` before packing.
5. **The signing request waits for the approver in SignPath**, up to an hour; the job's limit is raised to cover it.
6. **The README carries the code signing policy** SignPath requires, and its Credits name SignPath and the action.
7. **Both actions are pinned by commit, like the others**: `signpath/github-action-submit-signing-request` v3.0 and `actions/upload-artifact` v7.0.2.

## Consequences

- Positive: a release binary with a signature Smart App Control can accept and SmartScreen can attach reputation to, at no cost.
- Negative: the signature names SignPath Foundation. Every release waits for a manual approval, and a release cannot be signed while SignPath is down. Reputation builds with downloads, so the first signed releases may still show a SmartScreen warning. The signing action runs in the job that holds `contents: write`, which pinning by commit limits but does not remove. One person holds every role in the policy.
- Neutral: SignPath's side (roles, multi-factor authentication, approval, certificate) is outside this repository, and no tool here checks it; the workflow checks only that the signature verifies. The action's repository states no licence, so it is run by the workflow and never copied, which `NOTICE.md` records.

## Alternatives considered

- **A self-signed certificate**: free and instant, but trusted only where it is imported.
- **The Microsoft Store (MSIX)**: Microsoft signs the package, but the app would be repackaged and pass store certification.
- **Azure Artifact Signing**: Microsoft's preferred route for Smart App Control, but a paid subscription.
- **A purchased code signing certificate**: works for everyone, but costs money every year.
