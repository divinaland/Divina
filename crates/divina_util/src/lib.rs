// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

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

#[macro_export]
macro_rules! exit_with {
  ($exit_code:expr) => {
    std::process::exit($exit_code);
  };
  ($exit_code:expr, $($message:tt)*) => {
    println!($($message)*);

    std::process::exit($exit_code);
  };
}
