# Install

## When to use this

Use this page when you want to install and run the Rust runner binary as an
operator/integrator.

## What you will do

1. Install `dc-runner` via Cargo or GitHub release assets.
2. Verify the binary is on `PATH`.
3. Run a first command.

## Step-by-step

### 1) Cargo install (recommended)

```sh
cargo install dc-runner-cli --locked
```

### 2) GitHub release binary install (macOS/Linux)

```sh
PLATFORM="darwin-arm64" # or linux-x86_64
VERSION="v0.2.0"        # set release tag
BASE="https://github.com/jonruttan/dc-runner-rust/releases/download/${VERSION}"
curl -fL "${BASE}/dc-runner-${PLATFORM}" -o dc-runner
curl -fL "${BASE}/dc-runner-${PLATFORM}.sha256" -o dc-runner.sha256
shasum -a 256 -c dc-runner.sha256
chmod +x dc-runner
sudo mv dc-runner /usr/local/bin/dc-runner
```

### 3) Verify

```sh
dc-runner --help
dc-runner governance --help
dc-runner governance critical --help
```

## Common failure signals

- `dc-runner: command not found`
  - Cause: install location is not on `PATH`.
  - Fix: add install directory to shell `PATH` and restart shell.

- `checksum mismatch`
  - Cause: corrupted download or wrong release/tag.
  - Fix: re-download both binary and `.sha256`, verify tag/platform.

- `permission denied`
  - Cause: binary missing execute bit.
  - Fix: run `chmod +x dc-runner` before moving.

## Upgrade and uninstall

Upgrade with Cargo:

```sh
cargo install dc-runner-cli --locked --force
```

Pin version with Cargo:

```sh
cargo install dc-runner-cli --locked --version 0.2.0
```

Uninstall:

```sh
cargo uninstall dc-runner-cli
```

## Normative refs

- `/README.md`
- `/docs/commands.md`
- `/docs/release.md`
- `/dc-runner-cli/Cargo.toml`
