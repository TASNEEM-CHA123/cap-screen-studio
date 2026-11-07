<p align="center">
	<img width="150" height="150" src="https://github.com/CapSoftware/Cap/blob/main/apps/desktop-gpui/icons/Square310x310Logo.png" alt="Cap logo">
</p>

<h1 align="center">Cap</h1>

<p align="center">
	Beautiful, shareable screen recordings. Open source, fast, and built for teams that want to own their data.
</p>

> **Fork notice — Cap Test:** This repository is a downstream fork of [CapSoftware/Cap](https://github.com/CapSoftware/Cap) (the original Cap desktop app). The upstream Cap code is used here under its AGPLv3 license (see [LICENSE](LICENSE)); this fork adds a separated **Mic / System** audio workflow plus fixes and a distinct **Cap Test** identity so it can run side-by-side with an installed original Cap without sharing data or crashing into it. All Cap trademarks, logos, and upstream branding remain the property of their respective owners. See [Fork changes](#fork-changes-cap-test) and [Running Cap Test next to Cap](#running-cap-test-next-to-cap) below.

<p align="center">
	<a href="https://cap.so">Website</a>
	 |
	<a href="https://cap.so/download">Download</a>
	 |
	<a href="https://cap.so/docs">Docs</a>
	 |
	<a href="https://cap.so/pricing">Pricing</a>
	 |
	<a href="https://cap.link/discord">Discord</a>
</p>

<p align="center">
	<a href="https://console.algora.io/org/CapSoftware/bounties?status=open">
		<img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2FCapSoftware%2Fbounties%3Fstatus%3Dopen" alt="Open bounties">
	</a>
</p>

<img src="https://raw.githubusercontent.com/CapSoftware/Cap/refs/heads/main/apps/web/public/landing-cover.png" alt="Cap app preview">

## Fork changes (Cap Test)

This fork is derived from the upstream **Cap** project (Copyright © Cap Software, Inc.; AGPLv3 — see `LICENSE`). In addition to the upstream Cap desktop app, it adds:

- **Separated Mic / System timeline.** The timeline shows a video row plus mirrored, read-only Mic and System waveform rows, each with its own mute and volume. Rows only appear when the recording actually has that audio. Preview, MP4 export, and audio-only export all honor per-source mute (`mic_muted` / `system_muted`).
- **Audio-source choices.** MP4 export offers **Both / Mic / System**. Audio-only export offers **MP3** and **WAV** per selected source (WAV always; MP3 via libmp3lame/libshine/mp3_mf).
- **Export fixes.** Planar S16 MP3 encoding with `frame_size()` chunking and `set_stream`/`write_interleaved`; audio-only export applies the source inside `AudioExportSettings::export` and renders only `sample_windows` for estimates; audio progress units match the UI total.
- **Serde fix.** `AudioExportSettings.format` renamed to `audio_format` (`camelCase`) so `ExportSettings` (`#[serde(tag = "format")]`) JSON round-trips.
- **Side-by-side identity.** The Windows build ships as **Cap Test / `CapTest.exe`** with its own app-data, logs, URL scheme, tray ID, and disabled updater endpoint so it does not overwrite or steal from an installed Cap. See next section.

---

## Running Cap Test next to Cap

### Build prerequisites (Windows)

- **Rust** via rustup (pinned toolchain in `rust-toolchain.toml`; GPUI needs 1.95.0 for release bundles).
- **Bun 1.4.4**, **Node.js 24+**.
- **FFmpeg 7.1** shared build + DXC + ONNX Runtime — handled by `bun run cap-setup` (downloads into `target/ffmpeg`, `target/native-deps`).
- **Visual Studio 2022 BuildTools** (MSVC + CMake + Windows SDK) — `cap-setup` locates `libclang.dll` under `VC\Tools\LLVM` and writes `.cargo/config.toml` (`FFMPEG_DIR`, `LIBCLANG_PATH`, `CMAKE`, `ORT_DYLIB_PATH`).
- **`libclang` compatible with `bindgen 0.70.1`** (LLVM 19.x). If you use a newer LLVM (e.g. 23), FFmpeg bindings come out opaque and fail; use the portable 19.1.7 tarball:

  ```powershell
  curl.exe -L -o "$env:LOCALAPPDATA\llvm-19.1.7\tar.xz" "https://github.com/llvm/llvm-project/releases/download/llvmorg-19.1.7/clang+llvm-19.1.7-x86_64-pc-windows-msvc.tar.xz"
  tar -xf "$env:LOCALAPPDATA\llvm-19.1.7\tar.xz" -C "$env:LOCALAPPDATA\llvm-19.1.7"
  ```

- **Patched GPUI checkout** at `../zed-cap` (relative to this repo) — fetched/applied by `scripts/prepare-gpui-dependency.sh`.

### Build & run

```powershell
bun install
bun run env-setup
bun run cap-setup
Set-Location apps\desktop-gpui
cargo build -p cap-desktop-gpui --release   # -> target\release\cap-gpui.exe
```

Stage a runnable folder (exe + DLLs from the bundle mapping in `scripts/bundle-gpui.mjs`):

```powershell
Copy-Item target\release\cap-gpui.exe ..\..\mine\CapTest.exe
Copy-Item ..\..\target\native-deps\dxc\*.dll ..\..\mine\
Copy-Item ..\..\target\native-deps\onnxruntime\lib\*.dll ..\..\mine\
Copy-Item ..\..\target\debug\av*.dll, "..\..\target\debug\postproc-58.dll", "..\..\target\debug\sw*.dll" ..\..\mine\
```

Then double-click `mine\CapTest.exe`. The main window title is **“Cap Test”** (tray: **“Cap Test”**).

### How it avoids interfering with the original Cap

|  | Original Cap (installed) | Cap Test (this fork) |
|---|---|---|
| Exe / product | `Cap.exe` / “Cap” | `CapTest.exe` / “Cap Test” |
| App data (settings, recordings, PID, single-instance mutex) | `%APPDATA%\so.cap.desktop` | `%APPDATA%\so.cap.desktop.test` |
| Logs / transcription / thumbnails / update staging | `%LOCALAPPDATA%\so.cap.desktop\…` | `%LOCALAPPDATA%\so.cap.desktop.test\…` |
| Deep-link scheme | `cap-desktop://` | `cap-test://` (only `CapTest.exe` claims it) |
| Tray / window titles | “Cap …” | “Cap Test …” |
| Auto-updater | Cap CDN | disabled (`https://localhost/cap-test-update`) |

**Caveat:** global hotkeys (`Ctrl+Shift+F9`) are OS-global — only one app can hold a combo. If both apps fight over a hotkey, change it in one app (Shortcuts settings). Everything else (recordings, settings, deep links, single-instance) is fully separate.

Cap is the open source alternative to Loom. It gives you fast screen recording, polished local editing, instant share links, comments, transcripts, analytics, team workspaces, custom domains, custom S3 storage, and full self-hosting when you need complete control.

Use Cap for product demos, bug reports, onboarding, tutorials, design reviews, engineering walkthroughs, async standups, client updates, and any moment where showing the work is faster than scheduling another call.

## Why Cap

- **Record, edit, share.** Capture your screen, camera, and microphone, then share a link or export a finished video.
- **Instant Mode for speed.** Upload while recording and get a shareable link the moment you stop.
- **Studio Mode for polish.** Record locally, edit with backgrounds, zooms, trimming, captions, and export controls.
- **A native desktop app.** Cap runs on macOS, Windows, and Linux, with a web dashboard for viewing, sharing, and managing recordings.
- **Own your storage.** Use Cap Cloud, connect your own S3-compatible bucket, keep recordings local, or self-host the full platform.
- **Privacy by default.** Share publicly or privately, add passwords, use your own domain, or keep sensitive recordings off hosted infrastructure.
- **Async collaboration.** Comments, reactions, transcripts, viewer analytics, and team workspaces keep feedback attached to the video.
- **Cap AI.** Generate titles, summaries, clickable chapters, captions, and transcripts automatically.
- **Move from Loom.** Import existing Loom videos into Cap and keep your library in one place.

## Recording Modes

| Mode | Best for | How it works |
| --- | --- | --- |
| Instant Mode | Fast feedback, bug reports, async updates | Cap uploads while you record, then gives you a share link as soon as recording stops. |
| Studio Mode | Product demos, tutorials, launches, client work | Cap records locally, opens the editor, and lets you export or share a polished video. |

## Data Ownership

Cap is designed for people and teams who do not want their recording workflow locked inside a black box.

- Use Cap Cloud for the fastest hosted experience.
- Connect AWS S3, Cloudflare R2, Backblaze B2, MinIO, Wasabi, or another S3-compatible provider.
- Serve share pages from your own domain.
- Self-host Cap Web, the API, database, media server, and object storage with Docker Compose.
- Point Cap Desktop at your self-hosted instance from `Settings > Cap Server URL`.

## Get Started

For most users, the fastest path is:

1. Download Cap for macOS, Windows, or Linux from [cap.so/download](https://cap.so/download).
2. Sign in or create an account.
3. Choose Instant Mode or Studio Mode.
4. Record your first Cap.
5. Share the link, export the file, or keep it local.

The full product docs live at [cap.so/docs](https://cap.so/docs).

## Self-Hosting

The fastest way to self-host Cap Web is Docker Compose:

```bash
git clone https://github.com/CapSoftware/Cap.git
cd Cap
docker compose up -d
```

Cap will be available at `http://localhost:3000`.

Login links appear in the service logs when email is not configured:

```bash
docker compose logs cap-web
```

### Deployment Options

| Method | Best for |
| --- | --- |
| Docker Compose | VPS, home servers, and any Docker-capable host |
| [Railway](https://railway.com/new/template/PwpGcf) | One-click managed hosting |
| Coolify | Self-hosted PaaS deployments with `docker-compose.coolify.yml` |

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/new/template/PwpGcf)

For production, configure public URLs and replace the default secrets before exposing the deployment to the internet:

```bash
CAP_URL=https://cap.yourdomain.com
S3_PUBLIC_URL=https://s3.yourdomain.com
```

See the [self-hosting guide](https://cap.so/docs/self-hosting) for email setup, AI providers, SSL, storage, production hardening, and troubleshooting.

For single-owner instances behind an authenticating reverse proxy, see
[trusted-proxy authentication](docs/trusted-proxy-auth.md) for the required shared
secret, proxy hardening, and how to trigger sign-in without a built-in login button.

## Local Development

Cap is a Turborepo monorepo. The desktop app is native Rust with a [GPUI](https://www.gpui.rs/) interface, the web app is Next.js with Drizzle and MySQL, and recording, rendering, and export live in shared Rust crates.

Requirements:

- Node.js 24 or newer
- Bun 1.4.4
- Rust via [rustup](https://rustup.rs). The workspace pins 1.88; the desktop app pins its own toolchain, which rustup installs on first build.
- Docker for MySQL, MinIO, and local services

Install and set up the repo:

```bash
bun install
bun run env-setup
bun run cap-setup
```

Common commands:

| Command | Purpose |
| --- | --- |
| `bun run dev` | Start the full local development stack |
| `bun run dev:web` | Start the web app without the desktop app |
| `bun run dev:desktop` | Build and run the desktop app, rebuilding on save on macOS |
| `bun run build` | Build the workspace |
| `bun run gpui:build <target>` | Build and package a desktop release for a Rust target triple |
| `bun run lint` | Run Biome linting |
| `bun run format` | Format with Biome |
| `bun run typecheck` | Run TypeScript project references |
| `cargo test -p <crate>` | Run Rust tests for a crate |

Database commands:

| Command | Purpose |
| --- | --- |
| `bun run db:generate` | Generate database artifacts |
| `bun run db:push` | Push schema changes |
| `bun run db:studio` | Open Drizzle Studio |

## Repository Map

| Path | What lives there |
| --- | --- |
| `apps/desktop-gpui` | The Cap desktop app: native Rust and GPUI for macOS, Windows, and Linux |
| `apps/web` | Next.js web app for marketing, docs, dashboard, sharing, API routes, and auth |
| `apps/cli` | Rust CLI |
| `apps/media-server` | Media processing service used by the web app |
| `apps/discord-bot` | Discord integration |
| `packages/database` | Drizzle schema and database access |
| `packages/ui` | Shared React UI |
| `packages/web-backend` | Backend service layer |
| `packages/web-domain` | Web domain models and types |
| `packages/env` | Environment validation |
| `packages/sdk-embed` | Embed SDK |
| `packages/sdk-recorder` | Recorder SDK |
| `crates/*` | Recording, capture, camera, audio, encoding, rendering, muxing, export, and test crates |
| `scripts/*` | Setup, analytics, build, and maintenance tooling |
| `infra/*` | Infrastructure configuration |

The web API uses Effect and `@effect/platform` HTTP APIs. Desktop capture and export paths are backed by Rust crates for fast recording, rendering, and platform-specific media access.

## Analytics

Cap uses [Tinybird](https://www.tinybird.co) for viewer telemetry dashboards. Set `TINYBIRD_ADMIN_TOKEN` or `TINYBIRD_TOKEN` before running analytics commands.

| Command | Purpose |
| --- | --- |
| `bun run analytics:setup` | Deploy Tinybird datasources and pipes from `scripts/analytics/tinybird` |
| `bun run analytics:check` | Validate that the Tinybird workspace matches the app expectations |

`analytics:setup` can remove Tinybird resources outside the checked-in analytics configuration. Use it only against the workspace you intend to manage from this repo.

## Contributing

Cap is built in public. Issues, pull requests, design feedback, bug reports, docs fixes, and bounties are welcome.

- Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
- Join the community on [Discord](https://cap.link/discord).
- Check open bounties on [Algora](https://console.algora.io/org/CapSoftware/bounties?status=open).

## License

Portions of this software are licensed as follows:

- Code in the `cap-camera*` and `scap-*` crate families is licensed under the MIT License. See [licenses/LICENSE-MIT](https://github.com/CapSoftware/Cap/blob/main/licenses/LICENSE-MIT).
- Third-party components are licensed under the original license provided by their owner.
- All other content not mentioned above is available under the AGPLv3 license as defined in [LICENSE](https://github.com/CapSoftware/Cap/blob/main/LICENSE).
# cap-screen-studio
