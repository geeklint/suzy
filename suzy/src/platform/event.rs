/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

/// A type which implements EventLoopState with a single boolean flag.
#[derive(Clone, Copy, Debug)]
pub struct SimpleEventLoopState {
    /// A flag indicating if the event loop should keep running.
    pub running: bool,
}

impl Default for SimpleEventLoopState {
    fn default() -> Self {
        Self { running: true }
    }
}
