[![pipeline](https://github.com/d-e-s-o/npd/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/d-e-s-o/npd/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/npd.svg)](https://crates.io/crates/npd)

npd
===

`npd` (Now Playing Daemon) is program/daemon sending a DBus notification
when a new song is played in [MPD][mpd]. It uses `inotify` to not waste
CPU resources polling MPD's state -- an approach taken by other
solutions.

[mpd]: https://www.musicpd.org/
