# GitHub Desktop / desktop.ini

- Google Drive 同期配下では `.git/` に `desktop.ini` が大量混入し、GitHub Desktop が `fatal: git show-ref: bad ref refs/tags/desktop.ini` で壊れることがある。
- 共有手順として `install_git_hooks.ps1` で `core.hooksPath=.githooks` を設定し、`.git/**/desktop.ini` を自動掃除する。
- 復旧手順は `cleanup_git_desktop_ini.ps1` または `repair_git_metadata.bat` 実行後に GitHub Desktop 再起動。
