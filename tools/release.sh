#!/bin/bash
# 在本機（Apple Silicon Mac）打包並發佈版本，規則見 spec/20261101-release.md。
#
#   tools/release.sh build              檢查、跑測試、打包四個平台到 dist/v<版本>/
#   tools/release.sh publish <說明檔>    推 tag、建立 GitHub Release、上傳、版本號加一並推送
set -euo pipefail
cd "$(dirname "$0")/.."

REPO=supacode-org/supa-live
TARGETS_MAC="aarch64-apple-darwin x86_64-apple-darwin"
TARGETS_WIN="x86_64-pc-windows-msvc aarch64-pc-windows-msvc"

die() { echo "錯誤：$*" >&2; exit 1; }

# [workspace.package] 底下的第一個 version
version() { awk '/^\[workspace.package\]/{p=1;next} /^\[/{p=0} p&&/^version *=/{gsub(/[" ]/,"",$0);split($0,a,"=");print a[2];exit}' Cargo.toml; }

VER=$(version)
TAG="v$VER"
DIST="dist/$TAG"
[ -n "$VER" ] || die "讀不到 Cargo.toml 的版本號"

# 發佈用的檔名，build 產生、publish 檢查
artifacts() {
  echo "Supa-Live_${VER}_aarch64.dmg Supa-Live_${VER}_x64.dmg Supa-Live_${VER}_x64-setup.exe Supa-Live_${VER}_arm64-setup.exe"
  for os in macos-aarch64 macos-x86_64 windows-x86_64 windows-aarch64; do echo "supa-live-cli_${VER}_$os.zip"; done
}

preflight() {
  [ "$(git rev-parse --abbrev-ref HEAD)" = main ] || die "要在 main 分支上"
  [ -z "$(git status --porcelain)" ] || die "工作區有未提交的改動"
  git fetch -q origin main --tags
  [ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || die "main 和 origin/main 不同步"
  ! git rev-parse -q --verify "refs/tags/$TAG" >/dev/null || die "tag $TAG 已經存在"
  for t in $TARGETS_MAC $TARGETS_WIN; do
    rustup target list --installed | grep -qx "$t" || die "缺少 Rust target $t，見 spec 的「本機需要的工具」"
  done
  command -v makensis >/dev/null || die "缺少 makensis（brew install nsis）"
  command -v lld-link >/dev/null || die "缺少 lld-link（brew install lld）"
  command -v cargo-xwin >/dev/null || die "缺少 cargo-xwin（cargo install --locked cargo-xwin）"
  cargo tauri --version >/dev/null 2>&1 || die "缺少 tauri-cli（cargo install tauri-cli --version \"^2\" --locked）"
}

# 執行檔裡會留下原始碼路徑（panic 訊息用），把家目錄和專案目錄換掉，不洩漏使用者名稱。
# 用各 target 自己的 RUSTFLAGS：設全域 RUSTFLAGS 會蓋掉 cargo-xwin 給的連結設定。
remap_paths() {
  local v; v="CARGO_TARGET_$(echo "$1" | tr 'a-z-' 'A-Z_')_RUSTFLAGS"
  export "$v=--remap-path-prefix=$HOME=~ --remap-path-prefix=$PWD=."
}

# 確認執行檔裡沒有家目錄
check_paths() { # <執行檔>...
  local f
  for f in "$@"; do
    ! LC_ALL=C grep -q -a -F "$HOME" "$f" || die "$f 裡有本機路徑 $HOME"
  done
}

# Tauri 用 productName 當安裝檔檔名，裡面有空格；GitHub 會把空格換成點，所以複製時改成連字號
copy_bundle() { # <檔案>...
  local f
  for f in "$@"; do cp "$f" "$DIST/$(basename "$f" | sed 's/^Supa Live_/Supa-Live_/')"; done
}

zip_cli() { # <target> <os 名稱> <執行檔名>
  local out="$PWD/$DIST/supa-live-cli_${VER}_$2.zip"
  (cd "target/$1/release" && zip -q -X "$out" "$3")
}

build() {
  preflight
  echo "== 檢查 $TAG"
  cargo fmt --all --check
  cargo clippy -q --workspace --all-targets -- -D warnings
  cargo test -q --workspace

  rm -rf "$DIST" && mkdir -p "$DIST"
  local bundle=apps/desktop/src-tauri

  for t in $TARGETS_MAC; do
    echo "== 打包 $t"
    # ad-hoc 簽名，沒有 Developer ID
    remap_paths "$t"
    (cd "$bundle" && APPLE_SIGNING_IDENTITY=- cargo tauri build --target "$t" --bundles dmg)
    copy_bundle target/"$t"/release/bundle/dmg/"Supa Live_$VER"_*.dmg
    cargo build -q --release -p supa-live --target "$t"
    check_paths target/"$t"/release/supa-live-desktop target/"$t"/release/supa-live
    zip_cli "$t" "macos-${t%%-*}" supa-live
  done

  # cargo-xwin 要用 Homebrew 的 llvm（clang-cl）；只在打 Windows 時放進 PATH，不影響 macOS 的建置。
  # makensis 在 C locale 下會 bad_alloc 當掉，明確給 UTF-8 locale。
  # ring 在 ARM64 Windows 上用 clang 編譯，不吃 clang-cl 的參數，所以 cargo-xwin 改用 clang 模式。
  local llvm; llvm="$(brew --prefix llvm)/bin"
  export XWIN_CROSS_COMPILER=clang
  for t in $TARGETS_WIN; do
    echo "== 打包 $t"
    remap_paths "$t"
    (cd "$bundle" && PATH="$llvm:$PATH" LC_ALL=en_US.UTF-8 cargo tauri build --runner cargo-xwin --target "$t" --bundles nsis)
    copy_bundle target/"$t"/release/bundle/nsis/"Supa Live_$VER"_*-setup.exe
    PATH="$llvm:$PATH" cargo xwin build -q --release -p supa-live --target "$t"
    check_paths target/"$t"/release/supa-live-desktop.exe target/"$t"/release/supa-live.exe
    zip_cli "$t" "windows-${t%%-*}" supa-live.exe
  done

  for f in $(artifacts); do [ -f "$DIST/$f" ] || die "缺少 $DIST/$f"; done
  echo "== 完成，產物在 $DIST/"
  ls -lh "$DIST"
}

publish() {
  local notes=${1:-}
  [ -f "$notes" ] || die "要給 Release 說明檔：tools/release.sh publish <說明檔>"
  preflight
  local files=()
  for f in $(artifacts); do [ -f "$DIST/$f" ] || die "缺少 $DIST/$f，先跑 tools/release.sh build"; files+=("$DIST/$f"); done

  echo "== 發佈 $TAG"
  git tag -a "$TAG" -m "Supa Live $VER"
  git push -q origin "$TAG"
  gh release create "$TAG" "${files[@]}" --repo "$REPO" --title "Supa Live $VER" --notes-file "$notes" --verify-tag

  # 修訂號加一
  local next; next=$(echo "$VER" | awk -F. '{print $1"."$2"."$3+1}')
  echo "== 版本號 $VER → $next"
  awk -v v="$next" '/^\[workspace.package\]/{p=1} /^\[/&&!/^\[workspace.package\]/{p=0} p&&/^version *=/&&!d{print "version = \"" v "\"";d=1;next} {print}' Cargo.toml > Cargo.toml.new
  mv Cargo.toml.new Cargo.toml
  cargo update -q --workspace
  git add Cargo.toml Cargo.lock
  git commit -q -m "版本號改為 $next" -m "$VER 已發佈，下一次發佈用這個版本號。"
  git push -q origin main
  echo "== 完成：https://github.com/$REPO/releases/tag/$TAG"
}

case "${1:-}" in
  build) build ;;
  publish) shift; publish "$@" ;;
  *) echo "用法：tools/release.sh build | publish <說明檔>" >&2; exit 2 ;;
esac
