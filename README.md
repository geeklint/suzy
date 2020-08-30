
This is an early-stage attempt to write a GUI framework in Rust based on the
observer pattern.  Inspired heavily by the [kivy](https://kivy.org/#home)
framework for Python.

## Project Goals:
* Productivity - One of the biggest priorities to emulate from kivy is ease
  of use for rapid development.
* Explicit layout by default - the first-class layout support is to **write
  declaritive code** which describes the relationship between GUI elements.
  This is the author's personal preference over alternitives like flex-box.
* Multitouch by default - also inherited from kivy, mouse and touchscreen
  input is unified, unless explicitly distinguished by the application.

## Lesser Project Goals
* BYO - Although the declaritive, observer pattern style is prescribed, the
  intention is that very little else will be: non-optional dependencies will
  be minimized in favor of "glue" that allows applications to choose their own
  set of utilities for tasks such as image loading, windowing, etc.
* Embeddebility - Optimistically, you should be able to use this as a library
  within e.g. a game engine.  The details of this are not fully explored yet.
* Performance - you should be able to write a light-weight GUI using this
  library.  Being "the fastest" is not a priority for the author; it is assumed
  to be unlikely that GUI code is a common bottleneck for real-world
  applications.
