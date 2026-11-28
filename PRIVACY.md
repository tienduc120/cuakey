# Privacy policy

This policy covers CuaKey, the Vietnamese typing app for Windows, whether it is installed from the Microsoft Store or from the zip on the Releases page.

## What CuaKey reads

- **The keys you press.** To turn what you type into Vietnamese, CuaKey sees each key through a Windows keyboard hook and replaces the text it changes.
- **The kind of control that has focus.** Through UI Automation, CuaKey tells a text box from a password box or a game, so Vietnamese applies only where text is typed. It never changes what you type into a password box.
- **The name of the app in front**, when you use the always-on list or per-app memory, to apply the language you set for that app.
- **The clipboard**, only when you run quick convert.

## What CuaKey keeps

Keys are held in memory only while CuaKey works out the word being typed. They are never written to disk and never logged.

CuaKey stores its settings, the apps you add to its lists, and your macros in `%APPDATA%\CuaKey` on your PC. Uninstalling the Store app deletes them; with the zip, delete that folder.

## What CuaKey sends

Nothing. CuaKey has no network code, no telemetry, no ads, and no accounts. A link in the About window opens in your browser only when you click it.

## Changes and contact

Changes to this policy are made in this file, in the project's public repository, and its history shows each one. For questions, open an issue at <https://github.com/tienduc120/cuakey/issues>.
