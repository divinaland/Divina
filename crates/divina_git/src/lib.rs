// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

//! Much help from:
//!
//! - <https://github.com/rust-lang/git2-rs/blob/master/examples/clone.rs>

#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code
)]
#![deny(clippy::all, clippy::nursery, clippy::pedantic)]
#![recursion_limit = "128"]
#![doc(
  html_logo_url = "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/160/twitter/282/ribbon_1f380.png",
  html_favicon_url = "https://emojipedia-us.s3.dualstack.us-west-1.amazonaws.com/thumbs/160/twitter/282/ribbon_1f380.png"
)]

use std::{io::Write, path};

struct State {
  progress: Option<git2::Progress<'static>>,
  total:    usize,
  current:  usize,
  path:     Option<path::PathBuf>,
  newline:  bool,
}

fn print(state: &mut State) {
  let git_stats = state.progress.as_ref().unwrap();
  let network_pct = (100 * git_stats.received_objects()) / git_stats.total_objects();
  let index_pct = (100 * git_stats.indexed_objects()) / git_stats.total_objects();
  let co_pct = if state.total > 0 {
    (100 * state.current) / state.total
  } else {
    0
  };
  let kbytes = git_stats.received_bytes() / 1024;
  if git_stats.received_objects() == git_stats.total_objects() {
    if !state.newline {
      println!();
      state.newline = true;
    }
    print!(
      ":: resolving deltas {}/{}\r",
      git_stats.indexed_deltas(),
      git_stats.total_deltas()
    );
  } else {
    print!(
      ":: net {:3}% ({:4} kb, {:5}/{:5}) / idx {:3}% ({:5}/{:5}) / chk {:3}% ({:4}/{:4}) {}\r",
      network_pct,
      kbytes,
      git_stats.received_objects(),
      git_stats.total_objects(),
      index_pct,
      git_stats.indexed_objects(),
      git_stats.total_objects(),
      co_pct,
      state.current,
      state.total,
      state
        .path
        .as_ref()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
    );
  }
  std::io::stdout().flush().unwrap();
}

/// # Errors
/// never
pub fn clone(url: &str, path: &str) -> Result<(), git2::Error> {
  let state = std::cell::RefCell::new(State {
    progress: None,
    total:    0,
    current:  0,
    path:     None,
    newline:  false,
  });
  let mut cb = git2::RemoteCallbacks::new();
  cb.transfer_progress(|stats| {
    let mut state = state.borrow_mut();
    state.progress = Some(stats.to_owned());
    print(&mut *state);
    true
  });

  let mut co = git2::build::CheckoutBuilder::new();
  co.progress(|path, cur, total| {
    let mut state = state.borrow_mut();
    state.path = path.map(path::Path::to_path_buf);
    state.current = cur;
    state.total = total;
    print(&mut *state);
  });

  let mut fo = git2::FetchOptions::new();
  fo.remote_callbacks(cb);
  git2::build::RepoBuilder::new()
    .fetch_options(fo)
    .with_checkout(co)
    .clone(url, path::Path::new(path))?;
  println!();

  Ok(())
}
