#!/bin/bash
# Clone all audit targets at HEAD (shallow), record manifest with hashes.
set -u
REPOS=/home/z/my-project/repos
MANIFEST=$REPOS/MANIFEST.txt
mkdir -p "$REPOS"

clone() {
  local name="$1" url="$2"
  if [ "$#" -gt 0 ] && [ -n "$FILTER" ]; then
    case " $FILTER " in *" $name "*) ;; *) return 0;; esac
  fi
  if [ -d "$REPOS/$name/.git" ] && git -C "$REPOS/$name" rev-parse HEAD >/dev/null 2>&1; then echo "SKIP $name"; return; fi
  rm -rf "$REPOS/$name"
  if git clone --depth 1 --single-branch --quiet "$url" "$REPOS/$name" 2>/dev/null; then
    echo "OK $name"
  else
    echo "FAIL $name $url"
  fi
}
FILTER="${1:-}"

# --- primary targets (v1 scope) ---
clone xilem          https://github.com/linebender/xilem &
clone floem          https://github.com/lapce/floem &
clone iced           https://github.com/iced-rs/iced &
clone icss           https://github.com/andaja/icss &
# --- secondary targets (v2 scope) ---
clone vizia          https://github.com/vizia/vizia &
clone slint          https://github.com/slint-ui/slint &
clone gpui-ce        https://github.com/gpui-ce/gpui-ce &
clone freya          https://github.com/marc2332/freya &
clone dioxus         https://github.com/DioxusLabs/dioxus &
clone egui           https://github.com/emilk/egui &
# --- parser & prior-art crates ---
clone ironframe      https://github.com/finitefield-org/ironframe &
clone tailwind-rs    https://github.com/oovm/tailwind-rs &
clone tailwind-fuse  https://github.com/gaucho-labs/tailwind-fuse &
clone tw_merge       https://github.com/rust-ui/ui &
clone encre-css      https://gitlab.com/encre-org/encre-css.git &
clone crepuscularity https://github.com/tschk/crepuscularity &
clone twill          https://github.com/FerrisMind/twill &
clone gpui-rsx       https://github.com/wsafight/gpui-rsx &
clone floem-tailwind https://github.com/lapce/floem-tailwind &
clone floem-shadcn   https://github.com/lapce/floem-shadcn &
clone shadcn-ui      https://github.com/RustForWeb/shadcn-ui &
clone leptos-shadcn  https://github.com/cloud-shuttle/leptos-shadcn-ui &
wait

: > "$MANIFEST"
for d in "$REPOS"/*/; do
  name=$(basename "$d")
  if [ -d "$d/.git" ]; then
    h=$(git -C "$d" rev-parse HEAD)
    dt=$(git -C "$d" log -1 --format=%cI)
    echo "$name HEAD $h $dt" >> "$MANIFEST"
  fi
done
sort -o "$MANIFEST" "$MANIFEST"
echo "ALL_CLONE_ATTEMPTS_DONE"
