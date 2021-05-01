#!/bin/bash
# [NOTE] This script is from: https://github.com/denoland/rusty_v8
# Copyright 2018-2019 the Deno authors. All rights reserved. MIT license.
# LICENSE: https://github.com/denoland/rusty_v8/blob/main/LICENSE

for REL in v0.13.0 v0.12.0; do
  mkdir -p $RUSTY_V8_MIRROR/$REL
  for FILE in \
    librusty_v8_debug_x86_64-unknown-linux-gnu.a \
    librusty_v8_release_x86_64-unknown-linux-gnu.a \
  ; do
    if [ ! -f $RUSTY_V8_MIRROR/$REL/$FILE ]; then
      wget -O $RUSTY_V8_MIRROR/$REL/$FILE \
        https://github.com/denoland/rusty_v8/releases/download/$REL/$FILE
    fi
  done
done