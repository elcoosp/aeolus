#!/bin/bash
# Clone all §2 repos at latest commit on default branch (shallow).
set -u
BASE=/home/z/my-project/repos
mkdir -p "$BASE"
LOG=$BASE/clone.log
: > "$LOG"

clone() {
  local url="$1" dir="$2"
  if [ -d "$BASE/$dir/.git" ]; then echo "SKIP $dir (exists)" >> "$LOG"; return; fi
  echo "CLONE $dir ..." >> "$LOG"
  GIT_TERMINAL_PROMPT=0 git clone --depth 1 --single-branch "$url" "$BASE/$dir" >> "$LOG" 2>&1 \
    && echo "OK   $dir $(git -C $BASE/$dir rev-parse HEAD) branch=$(git -C $BASE/$dir rev-parse --abbrev-ref HEAD)" >> "$LOG" \
    || echo "FAIL $dir" >> "$LOG"
}

clone https://github.com/linebender/xilem        xilem &
clone https://github.com/lapce/floem             floem &
clone https://github.com/iced-rs/iced            iced &
clone https://github.com/vizia-rs/vizia          vizia &
clone https://github.com/slint-ui/slint          slint &
clone https://github.com/DioxusLabs/dioxus       dioxus &
clone https://github.com/leptos-rs/leptos        leptos &
clone https://github.com/emilk/egui              egui &
clone https://github.com/RibirX/Ribir            ribir &
clone https://github.com/longbridge/gpui-ce      gpui-ce &
clone https://github.com/marc2332/freya          freya &
clone https://github.com/andaja/icss             icss &
wait
echo "ALL DONE" >> "$LOG"
