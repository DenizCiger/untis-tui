# untis-tui

Rust terminal UI for Untis.

## Install

```bash
npm install -g untis-tui
untis
```

The npm package installs the `untis` command and downloads a prebuilt binary for the current platform from GitHub Releases.

## Demo Mode

Run the bundled portfolio demo without WebUntis, saved credentials, or local cache:

```bash
untis --demo
```

If you are developing locally:

```bash
cargo run -- --demo
```

Demo mode uses curated mock timetable and absence data, keeps all interaction inside the real TUI, and never reads or writes your real profile state.

## Browser Demo

Build the dedicated browser-demo container:

```bash
docker build -f Dockerfile.demo -t untis-tui-demo .
docker run --rm -p 7681:7681 untis-tui-demo
```

Then open [http://localhost:7681](http://localhost:7681).
