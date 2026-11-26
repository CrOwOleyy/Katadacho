# Contributing to Katadacho

Thanks for your interest in Katadacho. Bug reports, ideas and pull requests are all welcome.

## Before you start

- **Bugs:** open an issue with the bug report template. Include your OS and version, and on Linux your
  desktop or compositor and whether you use X11 or Wayland.
- **Features:** open an issue first to discuss the idea, so your time is not spent on something that does
  not fit the project.
- **Security issues:** do not open a public issue, follow [SECURITY.md](SECURITY.md).

Everyone taking part is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Setting up

Follow [Building from source](README.md#building-from-source) in the README, then:

```bash
pnpm install
```

```bash
pnpm tauri dev
```

Only one instance of Katadacho runs at a time. If an installed copy is already running, the development
build hands over to it and exits: quit the installed copy from its tray icon first.

## Making a change

1. Create a branch from `main`: `feature/short-name` for a feature, `fix/short-name` for a bug fix.
2. Make small commits, one per change, with a short message in English in the imperative mood, without a
   prefix: `Add a mute button to the mixer`, not `[ADD] mute button` or `added mute button`.
3. Keep the code consistent with the files around it: naming, structure and comment style.
4. Run the checks below, then open a pull request.

### Code guidelines

- **User-facing text is in English.** This includes labels, messages and the tray menu.
- **Comments explain why, not what.** Some existing comments are in French; write new ones in English.
- **Platform code stays behind a shared interface.** A feature that calls system APIs has a Windows
  implementation and a Linux implementation selected with `#[cfg(...)]`, and other systems return the
  explicit error from `platform::unsupported`. See `src-tauri/src/controls/` for an example.
- **Pure logic is separated from system calls** so it can be tested on any OS: parsers, grouping,
  classification and state machines live in their own functions with unit tests.
- **Long or blocking work never runs on the UI thread.** Tauri commands are `async` and move blocking calls
  to `spawn_blocking`.
- **No new network calls** without discussing them in an issue first. Katadacho keeps everything local.

## Checks

Run these before opening a pull request:

```bash
pnpm test
```

```bash
pnpm build
```

```bash
cd src-tauri && cargo test --lib
```

If you change Linux code from another OS, the Fedora container runs the same Rust tests on Linux:

```bash
docker build -t katadacho-fedora -f packaging/linux/fedora.Dockerfile packaging/linux
```

```bash
docker run --rm -v "$PWD:/repo:ro" katadacho-fedora bash /repo/packaging/linux/build-in-container.sh test
```

If you change the interface, update the README screenshots: run `pnpm dev`, open
`http://localhost:1420/screenshots.html?scene=controls` (the scenes are listed in
`src/screenshots/main.tsx`), and capture the island at 900 by 520 pixels. The page uses fictional data,
so screenshots never show anything from your own machine.

The same checks run automatically on every pull request, on Windows and on Linux (see
`.github/workflows/ci.yml`).

## Releasing

1. Update `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`, and merge
   that change into `main`.
2. Tag the merge commit with the same version and push the tag:

   ```bash
   git tag v0.2.0
   ```

   ```bash
   git push origin v0.2.0
   ```

3. The release workflow builds the Windows installer and the Linux packages and attaches them to a draft
   release. It stops early if the tag does not match the version in `tauri.conf.json`.
4. On GitHub, open the draft under **Releases**, check the files, write the notes, and publish it.

## Pull requests

- Describe what changes for the user and why.
- Say which platforms you tested on, and what you could not test.
- Add screenshots for visible changes.
- Keep a pull request focused on one change. Unrelated fixes go in their own pull request.

## License

By contributing, you agree that your contributions are released under the [MIT License](LICENSE).
