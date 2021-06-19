/* SPDX-License-Identifier: (Apache-2.0 OR MIT OR Zlib) */
/* Copyright © 2021 Violet Leonard */

use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum ProgressBar {
    None,
    IntPctChild(std::process::Child),
}

impl ProgressBar {
    pub fn none() -> Self {
        Self::None
    }

    pub fn best(label: &str) -> Self {
        if cfg!(target_os = "linux") {
            if let Ok(zenity) = Command::new("zenity")
                .args(&[
                    "--progress",
                    "--title",
                    "Suzy Build Tools",
                    "--text",
                    label,
                    "--no-cancel",
                    "--auto-close",
                    "--width=400",
                ])
                .stdin(Stdio::piped())
                .spawn()
            {
                Self::IntPctChild(zenity)
            } else if let Ok(whiptail) = Self::whiptail(label) {
                Self::IntPctChild(whiptail)
            } else {
                Self::None
            }
        } else {
            Self::None
        }
    }

    pub fn update(&mut self, items: usize, total: usize) {
        match self {
            Self::None => (),
            Self::IntPctChild(child) => {
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ign = writeln!(stdin, "{}", items * 100 / total,);
                }
            }
        }
    }

    fn whiptail(label: &str) -> std::io::Result<std::process::Child> {
        let tty = std::fs::OpenOptions::new().write(true).open("/dev/tty")?;
        Command::new("whiptail")
            .args(&["--gauge", label, "6", "60", "0"])
            .stdin(Stdio::piped())
            .stdout(tty)
            .spawn()
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        match self {
            Self::None => (),
            Self::IntPctChild(child) => {
                let _ign = child.kill();
            }
        }
    }
}
