// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Now Playing Daemon (`npd`) is a program for sending `DBus`
//! notifications when the song currently played by MPD changes.

mod args;
mod mpd;

use std::collections::HashMap;
use std::env::args_os;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use anyhow::ensure;
use anyhow::Context as _;
use anyhow::Result;

use clap::error::ErrorKind;
use clap::Parser as _;

use inotify::Inotify;
use inotify::WatchMask;

use zbus::blocking::connection::Builder as ConnectionBuilder;
use zbus::names::WellKnownName;
use zbus::zvariant::Value;
use zbus::Address;

use crate::args::Args;


fn send_notification(summary: &str) -> Result<()> {
  let appname = env!("CARGO_PKG_NAME");
  let replaces_id = 1u32;
  let icon = "";
  let body = "";
  let hints = HashMap::<&str, Value>::new();
  // 5s.
  let timeout = 5000i32;

  let address = Address::session().context("failed to get D-Bus session address")?;
  let connection = ConnectionBuilder::address(address.clone())
    .with_context(|| format!("failed to create connection builder for address {address}"))?
    .build()
    .with_context(|| format!("failed to establish D-Bus session connection to {address}"))?;

  let bus = WellKnownName::from_static_str_unchecked("org.freedesktop.Notifications");
  let destination = Some(bus);
  let path = "/org/freedesktop/Notifications";
  let interface = "org.freedesktop.Notifications";
  let method = "Notify";

  let _msg_id = connection
    .call_method(
      destination.clone(),
      path,
      Some(interface),
      method,
      &(
        appname,
        replaces_id,
        icon,
        summary,
        body,
        [""; 0].as_slice(),
        &hints,
        timeout,
      ),
    )
    .with_context(|| format!("failed to call {method} method on {interface}"))?
    .body()
    .deserialize::<u32>()
    .context("failed to deserialize D-Bus message body")?;
  Ok(())
}

/// Run the program.
pub fn run() -> Result<()> {
  let _args = match Args::try_parse_from(args_os()) {
    Ok(args) => args,
    Err(err) => match err.kind() {
      ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
        print!("{err}");
        return Ok(())
      },
      _ => return Err(err.into()),
    },
  };

  let config_path = mpd::find_config()?;
  let config = mpd::parse_config_file(&config_path).context("failed to parse MPD config file")?;
  let state_file = config
    .get("state_file")
    .context("MPD configuration does not specify `state_file`")?;

  let mut inotify = Inotify::init().context("failed to create file watcher")?;
  let mut buffer = [0u8; 1024];
  let mut previous = None;
  loop {
    let _descriptor = inotify
      .watches()
      .add(state_file, WatchMask::CREATE)
      .with_context(|| format!("failed to add file watch for `{state_file}`"))?;

    let mut events = inotify
      .read_events_blocking(&mut buffer)
      .with_context(|| format!("failed to wait for inotify event on `{state_file}`"))?;

    if events.next().is_some() {
      let path = Path::new(state_file);
      let mut i = 0;
      // TODO: It is unclear why the file would not exist shortly after
      //       we receive a creation event, but that is what we see
      //       frequently. There shouldn't be any races, assuming it's
      //       only written a single time. Need to figure out what is
      //       going on.
      while !path.exists() {
        i += 1;
        ensure!(
          i < 500,
          "failed to find MPD state file at `{}`",
          path.display()
        );
        let () = sleep(Duration::from_millis(1));
      }

      let current =
        mpd::parse_state_file_current(path).context("failed to parse MPD state file")?;
      if current != previous {
        if let Some(current) = &current {
          let () = send_notification(current).context("failed to send DBus notification")?;
        }
      }
      previous = current;
    }
  }
}
