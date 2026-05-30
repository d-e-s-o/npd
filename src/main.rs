// Copyright (C) 2025-2026 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Now Playing Daemon (`npd`) is a program for sending `DBus`
//! notifications when the song currently played by MPD changes.

mod args;

use std::env::args_os;
use std::process::ExitCode;

use clap::Parser as _;

use npd::run;


fn main() -> ExitCode {
  let _args = match args::Args::try_parse_from(args_os()) {
    Ok(args) => args,
    Err(err) => {
      let _result = err.print();
      return u8::try_from(err.exit_code())
        .map(ExitCode::from)
        .unwrap_or(ExitCode::FAILURE)
    },
  };

  run()
    .map(|_| ExitCode::SUCCESS)
    .map_err(|e| eprintln!("{e:?}"))
    .unwrap_or(ExitCode::FAILURE)
}
