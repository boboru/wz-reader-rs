# wz-reader-rs 知識庫

> **最後更新**: 2026 年 1 月 18 日
> **版本**: 0.0.16
> **授權**: MIT

## 概述

**wz-reader-rs** 是一個執行緒安全的 Rust 函式庫，用於讀取 MapleStory `.wz` 和 `.ms` 遊戲資料檔案。它提供記憶體高效的解析、延遲載入、自動版本偵測，並支援多個 MapleStory 區域版本（GMS、EMS、BMS）。

本函式庫是從 [WzComparerR2.WzLib](https://github.com/Kagamia/WzComparerR2/tree/master/WzComparerR2.WzLib) 和 [MapleLib](https://github.com/lastbattle/MapleLib) 移植的 Rust 版本，作為 Rust 學習專案而創建。

## 快速連結

- **[架構總覽](./ARCHITECTURE.md)** - 系統架構、設計模式和組件關係
- **[API 參考](./API.md)** - 完整的 API 文件與範例
- **[組件指南](./COMPONENTS.md)** - 詳細的組件文件
- **[設定指南](./SETUP.md)** - 開發環境設定與建置說明
- **[使用範例](./EXAMPLES.md)** - 常見使用模式與程式碼範例
- **[格式規範](./FORMATS.md)** - 檔案格式技術規範

## 專案資訊

- **儲存庫**: https://github.com/spd789562/wz-reader-rs
- **文件**: https://docs.rs/wz_reader
- **Crate**: https://crates.io/crates/wz_reader
- **最低 Rust 版本**: 1.70.0
- **作者**: Leo Lin
- **授權**: MIT

## 核心功能

### 主要能力
- **執行緒安全解析**: 所有操作使用 `Arc<RwLock<>>` 實現執行緒安全
- **記憶體高效**: 記憶體映射檔案 I/O 配合延遲解析
- **自動版本偵測**: 自動偵測加密 IV 和補丁版本
- **多種格式支援**: 支援 `.wz`（經典格式）和 `.ms`（MapleStoryM）格式

### 支援的內容類型
- 圖片（PNG 自訂壓縮格式）
- 聲音（MP3/二進位格式）
- 影片
- Lua 腳本
- 原始資料
- 結構化屬性

### 可選功能
- `rayon`: 平行處理支援
- `serde`: JSON 序列化/反序列化
- `json`: JSON 匯出功能
- `zlib-ng`: 優化的 zlib 解壓縮（預設啟用）

## 快速開始

### 安裝

將以下內容加入你的 `Cargo.toml`：

```toml
[dependencies]
wz_reader = "0.0.16"
```

### 基本使用

```rust
use wz_reader::util::{resolve_base, walk_node};
use wz_reader::NodeCast;

fn main() {
    // 載入 .wz 檔案（自動偵測版本）
    let base_node = resolve_base(r"path/to/Base.wz", None).unwrap();

    // 遍歷所有節點
    walk_node(&base_node, true, &|node| {
        let node_read = node.read().unwrap();
        println!("{}", node_read.get_full_path());
    });
}
```

## 文件結構

```
docs/
├── INDEX.md                 # 本檔案 - 主要入口
├── ARCHITECTURE.md          # 系統架構與設計
├── API.md                   # 完整 API 參考
├── COMPONENTS.md            # 組件文件
├── SETUP.md                 # 開發設定指南
├── EXAMPLES.md              # 使用範例與模式
└── FORMATS.md               # 檔案格式規範
```

## 技術堆疊

### 核心依賴
- **memmap2**: 記憶體映射檔案 I/O
- **scroll**: 二進位資料解析
- **hashbrown**: 高效能 HashMap
- **thiserror**: 錯誤處理
- **aes + ecb**: AES 加密/解密
- **flate2**: Zlib 解壓縮
- **image**: 圖片處理
- **c2-chacha**: ChaCha20 加密（MS 格式）

### 開發依賴
- **criterion**: 效能基準測試
- **tempfile**: 臨時檔案處理
- **axum + tokio**: Web 伺服器範例

## 專案結構

```
wz-reader-rs/
├── src/
│   ├── lib.rs                    # 函式庫入口
│   ├── file.rs                   # WzFile 實作
│   ├── node.rs                   # WzNode 核心類型
│   ├── wz_image.rs               # WzImage 解析
│   ├── directory.rs              # 目錄結構
│   ├── reader.rs                 # 二進位讀取器實作
│   ├── object.rs                 # 物件類型定義
│   ├── version.rs                # 版本偵測
│   ├── header.rs                 # WZ 檔案標頭解析
│   ├── property/                 # 屬性類型
│   │   ├── mod.rs
│   │   ├── png.rs                # PNG 圖片處理
│   │   ├── sound.rs              # 聲音資料處理
│   │   ├── lua.rs                # Lua 腳本處理
│   │   ├── video.rs              # 影片資料處理
│   │   ├── string.rs             # 字串處理
│   │   ├── vector.rs             # Vector2D 類型
│   │   └── raw_data.rs           # 原始資料處理
│   ├── ms/                       # MapleStoryM 格式支援
│   │   ├── mod.rs
│   │   ├── file.rs               # MS 檔案格式
│   │   ├── ms_image.rs           # MS 圖片格式
│   │   ├── header.rs             # MS 標頭解析
│   │   ├── snow2_reader.rs       # Snow2 解密
│   │   ├── snow2_decryptor.rs    # Snow2 演算法
│   │   ├── chacha20_reader.rs    # ChaCha20 解密
│   │   └── utils.rs              # MS 工具函式
│   ├── util/                     # 工具模組
│   │   ├── mod.rs
│   │   ├── walk.rs               # 樹走訪工具
│   │   ├── resolver.rs           # 路徑解析
│   │   ├── node_util.rs          # 節點工具
│   │   ├── parse_property.rs     # 屬性解析
│   │   ├── wz_mutable_key.rs     # 加密金鑰管理
│   │   ├── maple_crypto_constants.rs  # 加密常數
│   │   └── color.rs              # 顏色工具
│   └── node_cast.rs              # 類型轉換 trait
├── examples/                     # 使用範例
├── tests/                        # 整合測試
├── benches/                      # 效能基準測試
└── docs/                         # 文件（本目錄）
```

## 常見使用案例

### 1. 提取圖片
```rust
use wz_reader::property::get_image;

walk_node(&base_node, true, &|node| {
    if let Some(png) = node.read().unwrap().try_as_png() {
        let image = get_image(&node).unwrap();
        image.save("output.png").unwrap();
    }
});
```

### 2. 解析特定節點
```rust
let node = base_node.read().unwrap()
    .at_path_parsed("Character/00002000.img")?;
```

### 3. JSON 匯出
```rust
#[cfg(feature = "json")]
let json = node.read().unwrap().to_simple_json()?;
```

## 效能考量

- 使用記憶體映射 I/O 實現高效檔案讀取
- 延遲解析 - 僅在存取時解析節點
- 可選的 Rayon 整合以實現平行處理
- 預設使用優化的 zlib-ng 壓縮
- 執行緒安全設計允許並發存取

## 已知限制

1. **學習專案**: 效能可能不如 C# 實作
2. **唯讀**: 不支援寫入/修改 .wz 檔案
3. **版本支援**: 主要在 BMS、GMS 和 EMS 版本上測試
4. **文件**: 這是學習專案，某些邊緣案例可能未記錄

## 貢獻

這主要是一個學習專案。請參閱儲存庫以了解貢獻指南。

## 相關專案

- [WzComparerR2](https://github.com/Kagamia/WzComparerR2) - 原始 C# 實作
- [MapleLib](https://github.com/lastbattle/MapleLib) - MapleStory 檔案的 C# 函式庫
- [WzDumper](https://github.com/lastbattle/Harepacker-resurrected) - .wz 檔案編輯器

## 支援

- **問題回報**: https://github.com/spd789562/wz-reader-rs/issues
- **文件**: https://docs.rs/wz_reader
- **範例**: 請參閱儲存庫中的 `examples/` 目錄

---

*於 2026-01-18 生成，作為深度儲存庫研究的一部分*
