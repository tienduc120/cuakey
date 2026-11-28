# 0017 - Releases through the Microsoft Store

- **Status**: accepted; supersedes 0016
- **Date**: 2026-11-28

## Context

SignPath Foundation declined to sign CuaKey: the project does not yet have the reputation it asks for. Without a signature, SmartScreen warns about the downloaded `cuakey.exe`, and Smart App Control blocks it with no way to allow it, however the file arrived (winget included).

What else signs it for an individual developer outside the US and Canada: Azure Artifact Signing accepts individuals only there, and has paused new individual onboarding; a purchased certificate costs money every year. The Microsoft Store registers individual developers for free, and Microsoft signs the MSIX package the Store delivers, which SmartScreen and Smart App Control both accept.

The Store signs only MSIX packages, and a packaged app runs differently: its writes under `HKCU` and new files under `%APPDATA%` go to a private per-package copy, the folder it is installed in changes with every version, and it may elevate only when its manifest declares so.

## Decision

1. **CuaKey is published in the Microsoft Store as an MSIX package**, from the maintainer's individual Partner Center account, which the Store names as the publisher.
2. **The release workflow builds the package and does not sign it.** It packs the `cuakey.exe` it built, with `services/cuakey/desktop/package/AppxManifest.xml` and the logos beside it, into `cuakey-<tag>-x64.msix` with `makeappx` from the Windows SDK on the runner, and keeps it as a workflow artifact. The maintainer uploads it in Partner Center. An unsigned package installs nowhere else, so it is not a release asset.
3. **The package version is the workspace version with a fourth part of 0**, as the Store requires. The identity name and publisher in the manifest are the values Partner Center assigns.
4. **The zip release stays, unsigned**, for people who would rather not use the Store. The SignPath steps, `.signpath/`, and the README's code signing policy go.
5. **The app knows whether it runs from the package** (`GetCurrentPackageFullName`), and when it does:
   - starting with Windows uses the package's startup task (`windows.startupTask`) through the `StartupTask` API, since Windows never reads the private copy of the `Run` key. When the user has turned the task off in Task Manager, the app cannot turn it on again and says so;
   - starting elevated keeps the scheduled task;
   - everything that names the program to start it (the scheduled task, the desktop shortcut, the restart as administrator) names the app execution alias `%LOCALAPPDATA%\Microsoft\WindowsApps\cuakey.exe`, which survives updates and starts the program inside its package;
   - the manifest declares the restricted capability `allowElevation`, without which a packaged app cannot run as administrator, with the reason given at submission.
6. **An update never closes a running CuaKey.** The manifest sets `uap17:UpdateWhileInUse` to `defer`, so an update lands the next time CuaKey starts or Windows restarts. Windows 11 24H2 and later honour it; older Windows ignores the element.
7. **The privacy policy the Store asks for is `PRIVACY.md`**: keystrokes are handled in memory and never stored or sent, and the settings file stays on the PC.
8. **`platform-windows` turns on the `windows` features it needs** (`ApplicationModel`, `Foundation`, `Win32_Storage_Packaging_Appx`); no dependency is added.

## Consequences

- Positive: free. A Store install is trusted by SmartScreen and Smart App Control, updates itself, and is also what `winget install --source msstore` installs. The publisher is the maintainer, not a foundation, and no one approves each signature by hand.
- Negative: every release waits for Store certification, from hours to days. The packaged paths run only in an installed package, so the tests cover which path is chosen, not what Windows does with it; each is checked by registering the package by hand (`Add-AppxPackage -Register` in Developer Mode). Microsoft may refuse `allowElevation`, or the app itself, since it reads every keystroke. Uninstalling the Store app deletes its settings. The zip still warns. Whether a Store update closes a running CuaKey on Windows before 11 24H2 is not known.
- Neutral: Partner Center (identity, listing, certification) is outside this repository, and no tool here checks it.

## Alternatives considered

- **SignPath Foundation** (ADR-0016): free, and signs the zip too, but declined the project for now; it can be asked again once the project has a longer history.
- **Azure Artifact Signing**: not open to an individual outside the US and Canada.
- **A purchased certificate** (Certum's open source certificate, about €69 a year): signs the zip as well, but costs money every year and signs awkwardly from CI.
- **winget alone**: easier to install and update, but signs nothing, so Smart App Control still blocks it.
- **A self-signed certificate**: trusted only where it is imported.
