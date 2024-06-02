#!/bin/bash

set -e

find suzy/ -name '*.rs' -print0 |
    xargs -0 grep -lZ 'with_default_render_platform!' |
    xargs -0 sed -e 's/with_default_render_platform!/mod tmp_for_fmt/' |
    rustfmt --edition 2018 --check |
    { ! grep .; }