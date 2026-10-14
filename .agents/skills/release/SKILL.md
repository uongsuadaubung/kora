---
name: release
description: >-
  Automates the end-to-end release workflow for Kora: runs tests, bumps the version in Cargo.toml,
  builds Kora.exe in release mode, commits, creates and pushes the git tag, generates Vietnamese
  release notes matching Kora's standard format, and publishes the GitHub Release with Kora.exe
  attached via the GitHub CLI (gh). Use this skill whenever the user asks to create a release,
  cut a new version/tag, publish a release on GitHub, or invokes /release.
---

# Kora Release Workflow

Follow these exact steps whenever the user asks to release a new version of Kora (e.g., "tạo release v0.8.3", "release bản mới", or `/release`).

## 1. Determine Target Version
- If the user specified a version (e.g., `v0.8.3` or `0.8.3`), normalize the Cargo version as `X.Y.Z` and the Git tag as `vX.Y.Z`.
- If the user did not specify a version number, inspect `Cargo.toml` and `git tag --sort=-v:refname` and increment the patch version by default (or ask if unclear).

## 2. Verify Test Suite
- Run `cargo test` and ensure all unit and integration tests pass with 0 failures before proceeding.

## 3. Update Version & Build Release Binary
1. Update `version = "X.Y.Z"` in `Cargo.toml` (if not already updated).
2. Build the production binary:
   ```powershell
   cargo build --release --bin Kora
   ```
   Verify that `target/release/Kora.exe` exists and compiled cleanly.

## 4. Commit, Tag, and Push
1. Stage all changes and commit (if there are uncommitted changes):
   ```powershell
   git add -A
   git commit -m "vX.Y.Z: <concise summary of changes>"
   ```
2. Create the annotated/lightweight tag `vX.Y.Z` (if not already created) and push both `main` and the tag to `origin`:
   ```powershell
   git tag vX.Y.Z
   git push origin main --tags
   ```

## 5. Draft Release Notes & Publish GitHub Release via `gh`
1. Inspect changes since the previous tag (`git log <prev_tag>..HEAD --oneline` or recent session changes) and write Vietnamese release notes to a temporary UTF-8 markdown file in the conversation `scratch/` directory (e.g., `release_notes_vX.Y.Z.md`) following Kora's standard format:
   ```markdown
   # Kora vX.Y.Z — <Tiêu đề tóm tắt điểm nhấn của bản phát hành>

   ## 🚀 Tính năng mới / Cải tiến (New Features & Improvements)
   - **<Tên hạng mục>:**
     - <Chi tiết thay đổi rõ ràng, súc tích>

   ## 🐛 Sửa lỗi Bộ gõ (Bug Fixes)
   - **<Tên lỗi đã sửa>:**
     - <Mô tả nguyên nhân và cách khắc phục>

   ## ⚡ Tối ưu Hiệu năng & Kiến trúc (Performance & Architecture)
   - **<Tên tối ưu>:**
     - <Mô tả kỹ thuật ngắn gọn>
   ```
   *(Only include sections that apply to the release.)*
2. Create the GitHub Release and upload `target/release/Kora.exe`:
   ```powershell
   gh release create vX.Y.Z target/release/Kora.exe --title "Version X.Y.Z" --notes-file "<path/to/release_notes_vX.Y.Z.md>"
   ```
3. Return the published GitHub Release URL, version, tag, and attached asset confirmation to the user.

