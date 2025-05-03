// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::Parser;


/// A program/daemon sending notifications when MPD plays a new song.
#[derive(Debug, Parser)]
#[command(version = env!("VERSION"))]
pub struct Args {}
