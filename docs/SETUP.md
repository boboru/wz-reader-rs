# 開發環境設置指南

> **最後更新**: January 18, 2026
> **版本**: 0.0.16

## 目錄

- [需求](#需求)
- [安裝](#安裝)
- [功能標誌](#功能標誌)
- [建置](#建置)
- [測試](#測試)
- [效能測試](#效能測試)
- [範例](#範例)
- [開發工作流程](#開發工作流程)
- [IDE 設置](#ide-設置)
- [疑難排解](#疑難排解)

---

## 需求

### 最低 Rust 版本

**wz-reader-rs** 需要 Rust **1.70.0** 或更新版本。

```bash
# 檢查您的 Rust 版本
rustc --version

# 如需要請更新 Rust
rustup update
```

### 系統需求

- **作業系統**: Windows、macOS 或 Linux
- **記憶體**: 最少 4GB（建議 8GB+ 用於大型 .wz 檔案）
- **磁碟空間**: 函式庫與相依套件約需 ~50MB

### 相依套件

所有相依套件都透過 Cargo 管理。無需手動安裝外部函式庫。

**核心相依套件:**
- `memmap2` - 記憶體映射檔案 I/O
- `scroll` - 二進位資料解析
- `hashbrown` - 高效能 HashMap
- `aes` + `ecb` - AES 加密/解密
- `flate2` - Zlib 解壓縮
- `image` - 影像處理
- `c2-chacha` - ChaCha20 加密（MS 格式）
- `thiserror` - 錯誤處理

**可選相依套件:**
- `rayon` - 平行處理（可選功能）
- `serde` + `serde_json` - JSON 序列化（可選功能）

---

## 安裝

### 作為函式庫相依套件

在您的 `Cargo.toml` 中加入:

```toml
[dependencies]
wz_reader = "0.0.16"
```

或使用 `cargo add`:

```bash
cargo add wz_reader
```

### 從原始碼安裝

複製程式庫:

```bash
git clone https://github.com/spd789562/wz-reader-rs.git
cd wz-reader-rs
```

建置專案:

```bash
cargo build --release
```

---

## 功能標誌

wz-reader-rs 使用 Cargo 功能來啟用可選功能。

### 預設功能

```toml
default = ["rayon", "zlib-ng"]
```

預設情況下，**平行處理**和**最佳化 zlib** 已啟用。

### 可用功能

#### `rayon`（預設）

啟用使用 Rayon 的平行處理支援。

**優勢:**
- 平行樹遍歷
- 並行影像解析
- 更快的批次操作

**加入至 Cargo.toml:**
```toml
wz_reader = { version = "0.0.16", features = ["rayon"] }
```

**如不需要可停用:**
```toml
wz_reader = { version = "0.0.16", default-features = false }
```

#### `serde`

為所有型別啟用 Serde 序列化/反序列化。

**使用場景:**
- 將節點樹序列化到磁碟
- 網路傳輸
- 狀態持久化

**加入至 Cargo.toml:**
```toml
wz_reader = { version = "0.0.16", features = ["serde"] }
```

**範例:**
```rust
use wz_reader::WzNode;

let node = WzNode::from_str("test", 42, None);
let json = serde_json::to_string(&node)?;
```

#### `json`

啟用 JSON 匯出功能。自動包含 `serde`。

**使用場景:**
- 將 .wz 資料匯出為 JSON 檔案
- API 回應
- 資料檢查

**加入至 Cargo.toml:**
```toml
wz_reader = { version = "0.0.16", features = ["json"] }
```

**範例:**
```rust
let json = node.read().unwrap().to_simple_json()?;
println!("{}", serde_json::to_string_pretty(&json)?);
```

#### `zlib-ng`（預設）

使用最佳化的 zlib-ng 而非標準 zlib。

**優勢:**
- 解壓縮速度約快 2-3 倍
- PNG 提取效能更好

**為相容性停用:**
```toml
wz_reader = { version = "0.0.16", default-features = false, features = ["rayon"] }
```

### 功能組合

#### 最小配置（無功能）:
```toml
wz_reader = { version = "0.0.16", default-features = false }
```

#### 全部功能:
```toml
wz_reader = { version = "0.0.16", features = ["rayon", "serde", "json", "zlib-ng"] }
```

#### 自訂組合:
```toml
wz_reader = { version = "0.0.16", default-features = false, features = ["json"] }
```

---

## 建置

### 開發建置

快速編譯用於開發:

```bash
cargo build
```

### 發布建置

為正式環境最佳化建置:

```bash
cargo build --release
```

**發布最佳化:**
- 完整最佳化（`opt-level = 3`）
- 連結時最佳化（LTO）
- 更小的二進位檔案大小
- 約比除錯建置快 10-20 倍

### 使用特定功能建置

```bash
# 啟用 JSON 支援
cargo build --features json

# 不使用預設功能
cargo build --no-default-features

# 多個功能
cargo build --features "json,serde"
```

### 檢查而不建置

驗證程式碼可編譯但不產生二進位檔案:

```bash
cargo check
```

比 `cargo build` 更快，適合開發時使用。

---

## 測試

### 執行所有測試

```bash
cargo test
```

### 執行特定測試模組

```bash
cargo test wz_file_test
```

### 執行時顯示輸出

```bash
cargo test -- --nocapture
```

### 使用特定功能執行

```bash
cargo test --features json
```

### 整合測試

位於 `tests/` 目錄:

```bash
# 執行所有整合測試
cargo test --test '*'

# 執行特定測試檔案
cargo test --test wz_file_test
```

**主要測試檔案**: `tests/wz_file_test.rs`
- 全面的 WZ 檔案解析測試
- 版本檢測驗證
- 屬性解析驗證

### 測試資料需求

部分測試需要實際的 .wz 檔案。如果您沒有測試資料:

1. 測試將被跳過（不會失敗）
2. 或從 MapleStory 安裝目錄下載範例 .wz 檔案
3. 放置於 `test_data/` 目錄（如已配置）

---

## 效能測試

### 執行效能測試

```bash
cargo bench
```

**效能測試類別:**
- 檔案解析效能
- 屬性提取速度
- 版本檢測開銷
- 記憶體映射 I/O 效率

### 檢視效能測試結果

結果儲存在 `target/criterion/`:

```bash
# 檢視 HTML 報告
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
start target/criterion/report/index.html  # Windows
```

### 效能測試配置

位於 `benches/bench_main.rs`。

**效能測試範例:**
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse_wz_file", |b| {
        b.iter(|| {
            // 效能測試程式碼
        });
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

---

## 範例

### 列出所有範例

```bash
cargo run --example
```

**可用範例:**
- `single_file` - 載入並遍歷單一 .wz 檔案
- `extracting_pngs` - 提取所有 PNG 影像
- `extracting_sounds` - 提取所有音訊檔案
- `extracting_lua` - 提取 Lua 腳本
- `wz_to_json` - 將 .wz 轉換為 JSON
- `parallel_parse_wz_image` - 平行解析示範
- `with_axum` - 網頁伺服器整合
- `parse_ms_file` - 解析 MapleStoryM 檔案
- 以及更多...

### 執行範例

基本語法:

```bash
cargo run --example <name> -- <args>
```

#### 單一檔案範例

```bash
cargo run --example single_file -- "path/to/Base.wz"
```

#### 提取 PNG

```bash
# 單一檔案
cargo run --example extracting_pngs --features "image/png" -- single "Base.wz" "./output"

# 使用 Base.wz 解析度
cargo run --example extracting_pngs --features "image/png" -- base "Base.wz" "./output"

# 整個資料夾
cargo run --example extracting_pngs --features "image/png" -- folder "Data/" "./output"
```

#### 轉換為 JSON

```bash
cargo run --example wz_to_json --features json -- "UI.wz" "./output"
```

#### 平行解析

```bash
cargo run --example parallel_parse_wz_image --features rayon -- "Base.wz"
```

#### 網頁伺服器（Axum）

```bash
cargo run --example with_axum --features "json,image/default-formats"
```

然後造訪: `http://localhost:3000`

---

## 開發工作流程

### 建議的工作流程

1. **修改**原始碼
2. **檢查編譯**: `cargo check`
3. **執行測試**: `cargo test`
4. **測試範例**: `cargo run --example single_file -- <test_file>`
5. **格式化程式碼**: `cargo fmt`
6. **執行檢查工具**: `cargo clippy`
7. **提交變更**: `git commit`

### 程式碼格式化

```bash
# 格式化所有程式碼
cargo fmt

# 檢查格式但不修改
cargo fmt -- --check
```

### 程式碼檢查

```bash
# 執行 Clippy 檢查工具
cargo clippy

# 使用所有功能執行
cargo clippy --all-features

# 將警告視為錯誤
cargo clippy -- -D warnings
```

### 文件

```bash
# 建置文件
cargo doc

# 建置並在瀏覽器中開啟
cargo doc --open

# 包含私有項目
cargo doc --document-private-items
```

### 監視模式（需要 cargo-watch）

```bash
# 安裝 cargo-watch
cargo install cargo-watch

# 儲存時自動執行測試
cargo watch -x test

# 儲存時自動檢查
cargo watch -x check
```

---

## IDE 設置

### Visual Studio Code

**建議的擴充套件:**

1. **rust-analyzer** - 語言伺服器
   - 安裝: `ext install rust-lang.rust-analyzer`
   - 提供: IntelliSense、跳轉定義、重構

2. **CodeLLDB** - 除錯器
   - 安裝: `ext install vadimcn.vscode-lldb`

3. **crates** - 相依套件管理
   - 安裝: `ext install serayuzgur.crates`

**設定** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

**啟動配置** (`.vscode/launch.json`):
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug example",
      "cargo": {
        "args": [
          "build",
          "--example=single_file"
        ]
      },
      "args": ["path/to/test.wz"]
    }
  ]
}
```

### IntelliJ IDEA / CLion

1. 安裝 **IntelliJ Rust** 外掛
2. 開啟專案目錄
3. 在設定中啟用 Cargo 功能:
   - Settings → Languages & Frameworks → Rust
   - 勾選 "Use all features"

### Vim / Neovim

使用 **rust-analyzer** 配合您的 LSP 客戶端:

**For vim-lsp:**
```vim
if executable('rust-analyzer')
  au User lsp_setup call lsp#register_server({
    \   'name': 'rust-analyzer',
    \   'cmd': {server_info->['rust-analyzer']},
    \   'whitelist': ['rust'],
    \ })
endif
```

**For coc.nvim:**
```vim
:CocInstall coc-rust-analyzer
```

---

## 疑難排解

### 常見問題

#### 1. "Failed to compile" - 缺少 Rust

**問題**: 未安裝 Rust 或版本過舊。

**解決方法**:
```bash
# 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新 Rust
rustup update
```

#### 2. "Cannot find feature `rayon`"

**問題**: 功能標誌拼字錯誤或版本錯誤。

**解決方法**:
```toml
# 正確拼寫
wz_reader = { version = "0.0.16", features = ["rayon"] }
```

#### 3. Windows 上的 "Linking failed"

**問題**: 缺少 MSVC 建置工具。

**解決方法**:
1. 安裝 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. 選擇 "Desktop development with C++"
3. 重新啟動終端機並重試

或使用 GNU 工具鏈:
```bash
rustup default stable-gnu
```

#### 4. 解析大型檔案時 "Out of memory"

**問題**: 大型 .wz 檔案的 RAM 不足。

**解決方法**:
- 使用延遲解析（不解析整個樹）
- 使用後釋放節點:
  ```rust
  node.write().unwrap().unparse();
  ```
- 增加系統交換空間
- 以較小的區塊處理檔案

#### 5. "Version detection failed"

**問題**: 不支援或損壞的 .wz 檔案。

**解決方法**:
```rust
// 嘗試明確指定版本
let node = WzNode::from_wz_file_full(
    "Base.wz",
    Some(WzMapleVersion::GMS),
    Some(83),  // 已知的補丁版本
    None,
    None
)?;
```

#### 6. 範例無法編譯

**問題**: 缺少必要功能。

**解決方法**:
```bash
# PNG 提取需要 image/png 功能
cargo run --example extracting_pngs --features "image/png"

# JSON 需要 json 功能
cargo run --example wz_to_json --features json
```

#### 7. 測試失敗並顯示 "No such file"

**問題**: 測試資料不可用。

**解決方法**:
- 需要實際 .wz 檔案的測試將被跳過
- 這是預期行為，不是失敗
- 若要執行所有測試，請提供測試 .wz 檔案

---

## 平台特定注意事項

### Windows

- 使用 PowerShell 或 CMD 執行命令
- 含有空格的檔案路徑需要引號:
  ```bash
  cargo run --example single_file -- "C:\Program Files\MapleStory\Base.wz"
  ```

### macOS

- 在 Intel 和 Apple Silicon（M1/M2）上都可運作
- 可能需要 Xcode 命令列工具:
  ```bash
  xcode-select --install
  ```

### Linux

- 已在 Ubuntu、Debian、Fedora、Arch 上測試
- 需要 `build-essential` 或同等套件:
  ```bash
  # Debian/Ubuntu
  sudo apt install build-essential

  # Fedora
  sudo dnf install gcc

  # Arch
  sudo pacman -S base-devel
  ```

---

## 效能調校

### 編譯時最佳化

**Cargo.toml 設定檔配置:**

```toml
[profile.release]
opt-level = 3           # 最大最佳化
lto = true              # 連結時最佳化
codegen-units = 1       # 更好的最佳化，較慢編譯
panic = 'abort'         # 更小的二進位檔案
strip = true            # 移除除錯符號

[profile.dev]
opt-level = 1           # 更快的開發建置
```

### 執行時最佳化

1. **實際使用時使用發布建置**:
   ```bash
   cargo build --release
   cargo run --release --example single_file
   ```

2. **啟用 zlib-ng**（預設）:
   ```toml
   wz_reader = { version = "0.0.16", features = ["zlib-ng"] }
   ```

3. **適當時使用平行功能**:
   ```rust
   #[cfg(feature = "rayon")]
   use wz_reader::util::walk_node_parallel;
   ```

---

## 取得協助

### 文件

- **API 文件**: https://docs.rs/wz_reader
- **本指南**: `docs/` 目錄
- **範例**: `examples/` 目錄

### 社群

- **問題回報**: https://github.com/spd789562/wz-reader-rs/issues
- **討論區**: GitHub Discussions
- **程式庫**: https://github.com/spd789562/wz-reader-rs

### 回報錯誤

錯誤回報應包含:
1. Rust 版本（`rustc --version`）
2. 作業系統和版本
3. wz_reader 版本
4. 最小重現程式碼
5. .wz 檔案版本（如相關）
6. 錯誤訊息或堆疊追蹤

---

## 下一步

設置完成後:

1. **閱讀範例**: `examples/` 目錄
2. **查看 API 文件**: [API.md](./API.md)
3. **了解架構**: [ARCHITECTURE.md](./ARCHITECTURE.md)
4. **嘗試範例**: 從 `single_file.rs` 開始
5. **建構您的應用程式**: 在您的專案中使用此函式庫

---

*使用範例請參閱 [EXAMPLES.md](./EXAMPLES.md)*
*API 參考請參閱 [API.md](./API.md)*
*元件詳情請參閱 [COMPONENTS.md](./COMPONENTS.md)*
