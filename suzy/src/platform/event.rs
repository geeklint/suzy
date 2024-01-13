/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

/// A trait which the event handler can use to shutdown the event loop.
pub trait EventLoopState {
    /// Signal that the event loop should stop.
    fn request_shutdown(&mut self);
}

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

impl EventLoopState for SimpleEventLoopState {
    fn request_shutdown(&mut self) {
        self.running = false;
    }
}
