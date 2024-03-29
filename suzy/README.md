
[![crates.io](https://img.shields.io/crates/v/suzy.svg)](https://crates.io/crates/suzy)
[![docs.rs](https://docs.rs/suzy/badge.svg)](https://docs.rs/suzy/)
[![Build Status](https://github.com/geeklint/suzy/workflows/Rust/badge.svg)](https://github.com/geeklint/suzy/actions)
![License](https://img.shields.io/crates/l/suzy?color=blueviolet)

Suzy is a GUI framework in Rust based on the observer pattern.
Inspired heavily by the [kivy](https://kivy.org/#home) framework for Python.

## Project Goals:
* Explicit layout by default - the first-class layout support is to **write
  declaritive code** which describes the relationship between GUI elements.
* Productivity - The automatic observer patern enables rapid prototyping.
* Multitouch by default - mouse and touchscreen input is unified, unless
  explicitly distinguished by the application.

## Secondary Project Goals
* BYO - Although the declaritive, observer pattern style is prescribed, very
  little else is: non-optional dependencies are minimized in favor of "glue"
  that allows applications to choose their own set of utilities for tasks
  such as image loading, windowing, etc.
* Embeddebility - You should be able to use Suzy as a library within
  e.g. a game engine, as long as the graphics interface is compatible.
* Performance - GUIs made using Suzy are light-weight.
  It is uncommon that GUI code is a bottleneck for real-world
  applications.
