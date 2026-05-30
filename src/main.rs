// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Now Playing Daemon (`npd`) is a program for sending `DBus`
//! notifications when the song currently played by MPD changes.

use std::process::ExitCode;

use npd::run;


fn main() -> ExitCode {
  run()
    .map(|_| ExitCode::SUCCESS)
    .map_err(|e| eprintln!("{e:?}"))
    .unwrap_or(ExitCode::FAILURE)
}
