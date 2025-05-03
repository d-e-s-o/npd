// Copyright (C) 2025 Daniel Mueller <deso@posteo.net>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr as _;

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;

use dirs::config_dir;
use dirs::home_dir;


/// Find the MPD configuration file in a known set of locations.
pub(crate) fn find_config() -> Result<PathBuf> {
  macro_rules! return_if_exists {
    ($path:expr) => {
      if $path.try_exists()? {
        return Ok($path)
      }
    };
  }

  if let Some(config_dir) = config_dir() {
    return_if_exists!(config_dir.join("mpd").join("mpd.conf"));
  }

  if let Some(home_dir) = home_dir() {
    return_if_exists!(home_dir.join(".mpdconf"));
    return_if_exists!(home_dir.join(".mpd").join("mpd.conf"));
  }

  return_if_exists!(PathBuf::from("/etc/mpd.conf"));

  bail!("failed to find MPD configuration file")
}


fn parse_config<R>(mut reader: R) -> Result<HashMap<String, String>>
where
  R: BufRead,
{
  let mut line = String::new();
  let mut values = HashMap::new();
  while let Ok(len) = reader.read_line(&mut line) {
    if len == 0 {
      break
    }
    let s = line.trim();
    // TODO: Note that this would erroneously remove a `#` inside a
    //       comment. We ignore this case for now as it's pretty
    //       unlikely to be encountered.
    let s = if let Some(idx) = s.find('#') {
      &s[0..idx]
    } else {
      s
    };

    if let Some((key, mut value)) = s.split_once(|c: char| c.is_ascii_whitespace()) {
      // Could use `str::trim_matches` here, but it removes stuff
      // repeatedly.
      value = value.trim();
      value = value.strip_prefix('"').unwrap_or(value);
      value = value.strip_suffix('"').unwrap_or(value);
      let _prev = values.insert(key.to_string(), value.to_string());
    }
    let () = line.clear();
  }
  Ok(values)
}

/// Parse the MPD configuration.
pub(crate) fn parse_config_file(path: &Path) -> Result<HashMap<String, String>> {
  let file =
    File::open(path).with_context(|| format!("failed to open file `{}`", path.display()))?;
  parse_config(BufReader::new(file))
}


fn parse_state<R>(mut reader: R) -> Result<String>
where
  R: BufRead,
{
  let mut line = String::new();
  // The index of the currently playing song.
  let mut current_prefix = None;
  while let Ok(len) = reader.read_line(&mut line) {
    if len == 0 {
      break
    }

    match &current_prefix {
      // If we don't have a current song index yet, keep looking for it.
      None => {
        if let Some(current) = line.strip_prefix("current:") {
          let current = usize::from_str(current.trim())
            .with_context(|| format!("failed to parse current song index `{current}`"))?;
          current_prefix = Some(format!("{current}:"));
        }
      },
      // If we have a prefix then check each line for a match.
      Some(current_prefix) => {
        if let Some(current) = line.strip_prefix(current_prefix) {
          // Once we found the current song we can stop immediately.
          return Ok(current.trim().to_string())
        }
      },
    }
    let () = line.clear();
  }

  bail!("failed to find currently playing song in MPD state file contents")
}


/// Parse the MPD state file, retrieving the current song.
pub(crate) fn parse_state_file_current(path: &Path) -> Result<String> {
  let file =
    File::open(path).with_context(|| format!("failed to open file `{}`", path.display()))?;
  parse_state(BufReader::new(file))
}


#[cfg(test)]
mod tests {
  use super::*;

  use std::io::Cursor;


  /// Make sure that we can parse an MPD configuration file.
  #[test]
  fn config_parsing() {
    let conf = r##"
# An example configuration file for MPD.
# Read the user manual for documentation: http://www.musicpd.org/doc/user/


# Files and directories #######################################################
#
# This setting controls the top directory which MPD will search to discover the
# available audio files and add them to the daemon's online database. This
# setting defaults to the XDG directory, otherwise the music directory will be
# be disabled and audio files will only be accepted over ipc socket (using
# file:// protocol) or streaming files over an accepted protocol.
#
music_directory                 "/var/lib/mpd/music"
#
# This setting sets the MPD internal playlist directory. The purpose of this
# directory is storage for playlists created by MPD. The server will use
# playlist files not created by the server but only if they are in the MPD
# format. This setting defaults to playlist saving being disabled.
#
playlist_directory              "/var/lib/mpd/playlists"
#
# This setting sets the location of the MPD database. This file is used to
# load the database at server start up and store the database while the
# server is not up. This setting defaults to disabled which will allow
# MPD to accept files over ipc socket (using file:// protocol) or streaming
# files over an accepted protocol.
#
db_file                 "/var/lib/mpd/database"

# These settings are the locations for the daemon log files for the daemon.
#
# The special value "syslog" makes MPD use the local syslog daemon. This
# setting defaults to logging to syslog.
#
# If you use systemd, do not configure a log_file.  With systemd, MPD
# defaults to the systemd journal, which is fine.
#
log_file "/var/log/mpd.log"

# This setting sets the location of the file which stores the process ID
# for use of mpd --kill and some init scripts. This setting is disabled by
# default and the pid file will not be stored.
#
# If you use systemd, do not configure a pid_file.
#
pid_file "/tmp/mpd.pid"

# This setting sets the location of the file which contains information about
# most variables to get MPD back into the same general shape it was in before
# it was brought down. This setting is disabled by default and the server
# state will be reset on server start up.
#
state_file                      "/var/lib/mpd/state"
#
# The location of the sticker database.  This is a database which
# manages dynamic information attached to songs.
#
#sticker_file                   "~/.mpd/sticker.sql"
#
###############################################################################

# Input #######################################################################
#
input {
        plugin "curl"
#       proxy "proxy.isp.com:8080"
#       proxy_user "user"
#       proxy_password "password"
}
"##;
    let reader = BufReader::new(Cursor::new(conf));
    let values = parse_config(reader).unwrap();
    assert_eq!(values.get("state_file").unwrap(), "/var/lib/mpd/state");
  }

  /// Check that we can extract the name of the currently playing file
  /// from an MPD state file.
  #[test]
  fn state_file_parsing() {
    let state = r#"
sw_volume: 80
audio_device_state:1:My ALSA EQ
state: play
current: 6
time: 18.372000
random: 1
repeat: 1
single: 0
consume: 0
crossfade: 0
mixrampdb: 0.000000
mixrampdelay: -1.000000
playlist_begin
0:by-artist/various/21ror_-_talk_about.opus
1:by-artist/various/24kgoldn_-_mood_(feat._iann_dior).opus
2:by-artist/various/3_doors_down_-_kryptonite.opus
3:by-artist/various/ace_frehley_-_new_york_groove.opus
4:by-artist/various/adele_-_hello.opus
5:by-artist/various/adele_-_rolling_in_the_deep.m4a
6:by-artist/various/adele_-_someone_like_you.opus
7:by-artist/various/afroman_-_because_i_got_high.opus
8:by-artist/various/akon_-_i_wanna_love_you_feat._snoop_dogg.opus
9:by-artist/various/akon_-_smack_that_feat._eminem.opus
10:by-artist/various/alan_walker_-_faded.opus
11:by-artist/various/alessia_cara_-_scars_to_your_beautiful.opus
12:by-artist/various/alesso_-_heroes_(we_could_be)_(ft._tove_lo).aac
13:by-artist/various/alexandra_stan_-_mr._saxobeat.opus
14:by-artist/various/alex_metric_&_jacques_lu_cont_-_safe_with_you_(feat_malin).aac
15:by-artist/various/alicia_keys_-_girl_on_fire.opus
16:by-artist/various/all_about_she_-_higher_(free).aac
17:by-artist/various/alvyn_&_jstn_dmnd_-_sky_bri.opus
18:by-artist/various/arcando_&_maazel_-_to_be_loved_(feat._salvo).opus
19:by-artist/various/ariana_grande_-_7_rings.opus
20:by-artist/various/ariana_grande_-_side_to_side_(feat._nicki_minaj).opus
playlist_end
"#;

    let reader = BufReader::new(Cursor::new(state));
    let current = parse_state(reader).unwrap();
    assert_eq!(current, "by-artist/various/adele_-_someone_like_you.opus");
  }
}
