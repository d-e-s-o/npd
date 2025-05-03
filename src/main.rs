// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Now Playing Daemon (`npd`) is a program for sending `DBus`
//! notifications when the song currently played by MPD changes.

use anyhow::Result;

use npd::run;


fn main() -> Result<()> {
  run()
}
