# 元件指南

> **最後更新**: 2026 年 1 月 18 日
> **版本**: 0.0.16

## 目錄

- [概述](#概述)
- [檔案層元件](#檔案層元件)
  - [WzFile 模組](#wzfile-模組)
  - [MsFile 模組](#msfile-模組)
  - [WzImage 模組](#wzimage-模組)
  - [Directory 模組](#directory-模組)
- [核心節點系統](#核心節點系統)
  - [Node 模組](#node-模組)
  - [NodeCast 模組](#nodecast-模組)
  - [NodeName 模組](#nodename-模組)
- [物件系統](#物件系統)
  - [Object 模組](#object-模組)
- [讀取器元件](#讀取器元件)
  - [Reader 模組](#reader-模組)
  - [Header 模組](#header-模組)
- [屬性元件](#屬性元件)
  - [PNG 屬性](#png-屬性)
  - [Sound 屬性](#sound-屬性)
  - [String 屬性](#string-屬性)
  - [Lua 屬性](#lua-屬性)
  - [Video 屬性](#video-屬性)
  - [RawData 屬性](#rawdata-屬性)
  - [Vector 屬性](#vector-屬性)
- [MS 格式元件](#ms-格式元件)
  - [MS File](#ms-file)
  - [MS Image](#ms-image)
  - [Snow2 Reader](#snow2-reader)
  - [ChaCha20 Reader](#chacha20-reader)
- [工具元件](#工具元件)
  - [Walk 模組](#walk-模組)
  - [Resolver 模組](#resolver-模組)
  - [Node 工具](#node-工具)
  - [Property 解析器](#property-解析器)
- [版本系統](#版本系統)
- [元件依賴關係](#元件依賴關係)

---

## 概述

本文件提供 wz-reader-rs 中每個元件的詳細文件。元件按層次和職責組織。

**元件層次:**
1. **檔案層**: 不同檔案格式的入口點
2. **核心節點**: 統一的樹狀結構
3. **讀取器層**: 帶加密的二進位 I/O
4. **屬性層**: 型別特定的資料容器
5. **工具層**: 輔助函數和演算法

---

## 檔案層元件

### WzFile 模組

**位置**: `src/file.rs` (298 行)

**用途**: 解析 MapleStory .wz 封存檔案,包含版本偵測和驗證。

**關鍵結構:**

```rust
pub struct WzFile {
    pub reader: Arc<WzReader>,      // 記憶體映射讀取器
    pub offset: usize,              // 檔案起始偏移量
    pub block_size: usize,          // 總檔案大小
    pub is_parsed: bool,            // 解析狀態
    pub wz_file_meta: WzFileMeta,   // 元資料
}

pub struct WzFileMeta {
    pub path: String,                           // 檔案路徑
    pub patch_version: i32,                     // 遊戲版本 (例如 83, 176)
    pub wz_version_header: i32,                 // 加密版本
    pub wz_with_encrypt_version_header: bool,   // 標頭格式旗標
    pub hash: usize,                            // 用於偏移量計算的雜湊值
}
```

**職責:**
1. 記憶體映射 .wz 檔案
2. 偵測加密 IV (初始化向量)
3. 自動偵測補丁版本 (1-2000 範圍)
4. 透過檢查映像標頭驗證版本
5. 建立目錄結構

**版本偵測演算法:**
```rust
// src/file.rs:139-166
if patch_version == -1 {
    let guess_range = if wz_with_encrypt_version_header {
        1..2000  // 嘗試所有版本
    } else {
        770..780  // 64 位元客戶端範圍
    };

    for ver in guess_range {
        wz_file_meta.hash = check_and_get_version_hash(ver);
        if try_decode_with_version(ver).is_ok() {
            // 找到版本!
            break;
        }
    }
}
```

**版本雜湊值計算:**
```rust
// src/file.rs:270-297
fn check_and_get_version_hash(encver: i32, patch_version: i32) -> i32 {
    let mut version_hash: i32 = 0;

    // 雜湊補丁版本的每個數字
    for char in patch_version.to_string().chars() {
        let char_code = char.to_ascii_lowercase() as i32;
        version_hash = version_hash * 32 + char_code + 1;
    }

    // 驗證加密版本
    let enc = 0xff ^ (version_hash >> 24) & 0xff
                    ^ (version_hash >> 16) & 0xff
                    ^ (version_hash >> 8) & 0xff
                    ^ version_hash & 0xff;

    if enc == encver {
        version_hash
    } else {
        0  // 無效
    }
}
```

**64 位元客戶端偵測:**
```rust
// src/file.rs:250-268
fn check_64bit_client(reader: &WzSliceReader) -> (bool, u16) {
    let encrypt_version = reader.read_u16_at(reader.header.fstart)?;

    if encrypt_version > 0xff {
        return (false, 0);  // 64 位元客戶端 (無加密版本)
    }

    if encrypt_version == 0x80 {
        // 額外驗證
        let prop_count = reader.read_i32_at(fstart + 2)?;
        if prop_count > 0 && (prop_count & 0xff) == 0 {
            return (false, 0);
        }
    }

    (true, encrypt_version)  // 有加密版本
}
```

**解析流程:**
```
from_file()
  ├─> 開啟檔案並進行記憶體映射
  ├─> 猜測或驗證 IV
  ├─> 使用 IV 建立 WzReader
  ├─> 計算偏移量 (fstart + 2)
  └─> 回傳未解析的 WzFile

parse()
  ├─> 檢查 64 位元 vs 32 位元客戶端
  ├─> 自動偵測補丁版本
  │   ├─> 對範圍內的每個版本
  │   │   ├─> 計算雜湊值
  │   │   ├─> 嘗試使用版本解碼
  │   │   └─> 驗證第一個映像標頭
  │   └─> 成功時回傳
  ├─> 建立 WzDirectory
  └─> 解析子節點
```

**錯誤處理:**
- `InvalidWzFile`: 無效的標頭或格式
- `ErrorGameVerHash`: 版本偵測失敗
- `UnknownImageHeader`: 不支援的映像格式
- `UnableToGuessVersion`: 無法偵測 IV

**依賴關係:**
- `WzReader`: 記憶體映射 I/O
- `WzDirectory`: 目錄解析
- `version` 模組: IV/版本工具

---

### MsFile 模組

**位置**: `src/ms/file.rs`

**用途**: 解析 MapleStoryM .ms 檔案,使用 Snow2/ChaCha20 加密。

**關鍵結構:**

```rust
pub struct MsFile {
    pub reader: Arc<WzReader>,
    pub offset: usize,
    pub block_size: usize,
    pub is_parsed: bool,
    pub encryption_type: EncryptionType,
}

pub enum EncryptionType {
    Snow2,
    ChaCha20,
}
```

**加密偵測:**
```rust
// 檢查前 4 個位元組
match &file_data[0..4] {
    [0x50, 0x4B, 0x47, 0x31] => EncryptionType::Snow2,    // "PKG1"
    [0x4D, 0x53, 0x46, 0x44] => EncryptionType::ChaCha20, // "MSFD"
    _ => return Err(Error::InvalidMsFile),
}
```

**職責:**
1. 偵測加密類型 (Snow2 或 ChaCha20)
2. 建立適當的解密讀取器
3. 解析 MS 目錄結構
4. 將 MS 映像轉換為 WZ 格式

**依賴關係:**
- `Snow2Reader`: Snow2 串流加密
- `ChaCha20Reader`: ChaCha20 加密
- `MsImage`: MS 映像格式

---

### WzImage 模組

**位置**: `src/wz_image.rs` (207 行)

**用途**: 解析包含遊戲資料結構的 .img 容器。

**關鍵結構:**

```rust
pub struct WzImage {
    pub reader: Arc<WzReader>,
    pub name: WzNodeName,
    pub offset: usize,
    pub block_size: usize,
    pub is_parsed: bool,
}
```

**標頭位元組:**
```rust
pub const WZ_IMAGE_HEADER_BYTE_WITHOUT_OFFSET: u8 = 0x73;
pub const WZ_IMAGE_HEADER_BYTE_WITH_OFFSET: u8 = 0x1B;
```

**職責:**
1. 解析映像標頭 (0x73, 0x1B, 0x01, 0x23)
2. 處理特殊格式:
   - `.lua` 檔案 → WzLua
   - `.txt` 檔案 → WzRawData
   - `#Property` 檔案 → 原始文字資料
3. 遞迴解析屬性清單
4. 支援直接路徑存取而不需完整解析

**標頭格式偵測:**
```rust
// src/wz_image.rs:148-194
match header_byte {
    0x73 => {  // 標準映像
        let name = reader.read_wz_string()?;  // "Property"
        let value = reader.read_u16()?;        // 0
        parse_property_list()
    }
    0x1B => {  // 帶偏移量表的映像
        // 與 0x73 類似
    }
    0x01 => {  // Lua 腳本
        if name.ends_with(".lua") {
            let len = reader.read_wz_int()?;
            WzLua::new(reader, offset, len)
        }
    }
    0x23 => {  // 原始屬性文字 (ASCII 35 = '#')
        // 檢查 "#Property" 魔術字
        WzRawData::new(reader, offset + 9, block_size - 9)
    }
    _ => Err(UnknownImageHeader)
}
```

**直接存取功能:**
```rust
// src/wz_image.rs:103-125
pub fn at_path(&self, path: &str) -> Result<WzNodeArc> {
    // 直接導航而不解析整個樹
    let reader = self.reader.create_slice_reader_without_hash();
    reader.seek(self.offset);

    // 驗證標頭
    let header_byte = reader.read_u8()?;
    assert_eq!(header_byte, 0x73);

    // 使用工具導航路徑
    util::get_node(path, &self.reader, &reader, self.offset)
}
```

**依賴關係:**
- `util::parse_property` - 屬性解析邏輯
- `WzLua`, `WzRawData` - 特殊內容類型

---

### Directory 模組

**位置**: `src/directory.rs` (211 行)

**用途**: 表示 .wz 檔案中的資料夾結構。

**關鍵結構:**

```rust
pub struct WzDirectory {
    pub reader: Arc<WzReader>,
    pub offset: usize,
    pub block_size: usize,
    pub hash: usize,           // 用於偏移量計算
    pub is_parsed: bool,
}

enum WzDirectoryType {
    UnknownType,                  // 類型 1: 跳過
    RetrieveStringFromOffset,     // 類型 2: 偏移量處的名稱
    WzDirectory,                  // 類型 3: 巢狀目錄
    WzImage,                      // 類型 4: 映像檔案
    NewUnknownType,               // 未知: 錯誤
}
```

**目錄項目格式:**
```
[1 位元組]  類型 (1, 2, 3, 4)
[可變]      名稱 (基於類型)
[4 位元組]  檔案大小 (壓縮的 WZ 整數)
[4 位元組]  校驗和 (壓縮的 WZ 整數)
[4 位元組]  偏移量 (WZ 偏移量計算)
```

**解析演算法:**
```rust
// src/directory.rs:122-209
pub fn resolve_children() -> Result<WzNodeArcVec> {
    let entry_count = reader.read_wz_int()?;

    for _ in 0..entry_count {
        let dir_byte = reader.read_u8()?;
        let dir_type = get_wz_directory_type_from_byte(dir_byte);

        // 取得項目名稱
        let fname = match dir_type {
            RetrieveStringFromOffset => {
                let str_offset = reader.read_i32()?;
                // 從字串表讀取名稱
                reader.read_wz_string_at_offset(fstart + str_offset)?
            }
            WzDirectory | WzImage => {
                reader.read_wz_string()?
            }
            _ => continue
        };

        // 讀取元資料
        let fsize = reader.read_wz_int()?;
        let checksum = reader.read_wz_int()?;
        let offset = reader.read_wz_offset(hash, None)?;

        // 建立子節點
        match dir_type {
            WzDirectory => {
                let child_dir = WzDirectory::new(offset, fsize, reader, false);
                nodes.push((fname, child_dir.into_lock()));
            }
            WzImage => {
                let image = WzImage::new(fname, offset, fsize, reader);
                nodes.push((fname, image.into_lock()));
            }
        }
    }

    // 遞迴解析子目錄
    for (_, node) in &nodes {
        if let WzObjectType::Directory(dir) = &node.object_type {
            let children = dir.resolve_children(node)?;
            // 新增到節點
        }
    }
}
```

**版本驗證:**
```rust
// src/directory.rs:75-120
pub fn verify_hash(&self) -> Result<()> {
    // 讀取所有項目以驗證偏移量計算
    for _ in 0..entry_count {
        // ... 讀取項目 ...

        let buf_start = offset;
        let buf_end = buf_start + fsize as usize;

        if !reader.is_valid_pos(buf_end) {
            return Err(Error::InvalidWzVersion);  // 錯誤的版本!
        }
    }
    Ok(())
}
```

**職責:**
1. 解析目錄結構
2. 讀取項目元資料 (名稱、大小、偏移量、校驗和)
3. 處理字串表查找 (類型 2)
4. 遞迴解析巢狀目錄
5. 驗證偏移量以進行版本驗證

---

## 核心節點系統

### Node 模組

**位置**: `src/node.rs` (814 行)

**用途**: 統一所有 WZ 資料類型的核心樹狀結構。

**架構:**

```rust
pub struct WzNode {
    pub name: WzNodeName,
    pub object_type: WzObjectType,
    pub parent: Weak<RwLock<WzNode>>,           // 防止循環
    pub children: HashMap<WzNodeName, WzNodeArc>,
}

pub type WzNodeArc = Arc<RwLock<WzNode>>;
```

**關鍵功能:**

1. **延遲解析** (src/node.rs:173-228):
```rust
pub fn parse(&mut self, parent: &WzNodeArc) -> Result<()> {
    match self.object_type {
        WzObjectType::Directory(ref mut dir) => {
            if dir.is_parsed {
                return Ok(());  // 已解析
            }
            let childs = dir.resolve_children(parent)?;
            dir.is_parsed = true;
            // 新增子節點
        }
        // File、Image、MsFile、MsImage 類似
    }
}
```

2. **路徑導航** (src/node.rs:423-446):
```rust
pub fn at_path_parsed(&self, path: &str) -> Result<WzNodeArc> {
    let mut pathes = path.split('/');
    let first = self.at(pathes.next().unwrap())?;

    pathes.try_fold(first, |node, name| {
        let mut write = node.write().unwrap();
        write.parse(&node)?;  // 存取時解析
        write.at(name).ok_or(Error::NodeNotFound)
    })
}
```

3. **父節點導航** (src/node.rs:491-520):
```rust
pub fn filter_parent<F>(&self, cb: F) -> Option<WzNodeArc>
where F: Fn(&WzNode) -> bool
{
    let mut parent = self.parent.upgrade();
    while let Some(parent_node) = parent {
        let read = parent_node.read().unwrap();
        if cb(&read) {
            return Some(Arc::clone(&parent_node));
        }
        parent = read.parent.upgrade();
    }
    None
}
```

4. **JSON 序列化** (src/node.rs:537-588):
```rust
#[cfg(feature = "json")]
pub fn to_simple_json(&self) -> Result<serde_json::Value> {
    if self.children.is_empty() {
        // 葉節點 - 回傳值
        match &self.object_type {
            WzObjectType::Value(v) => Ok(v.clone().into()),
            _ => Ok(Value::Null)
        }
    } else {
        // 容器 - 遞迴子節點
        let mut json = Map::new();
        for (name, child) in &self.children {
            json.insert(name.to_string(), child.to_simple_json()?);
        }
        Ok(Value::Object(json))
    }
}
```

**記憶體管理:**
- 弱父參考防止參考循環
- `unparse()` 清除子節點以釋放記憶體
- `transfer_childs()` 用於合併樹 (用於 Base.wz 解析)

---

### NodeCast 模組

**位置**: `src/node_cast.rs` (351 行)

**用途**: WzNode 物件的型別安全向下轉型。

**特徵定義:**

```rust
pub trait WzNodeCast {
    fn try_as_file(&self) -> Option<&WzFile>;
    fn try_as_image(&self) -> Option<&WzImage>;
    fn try_as_png(&self) -> Option<&WzPng>;
    fn try_as_int(&self) -> Option<&i32>;
    // ... 20+ 方法
}
```

**實作模式:**

```rust
// 檔案層級類型的巨集
macro_rules! try_as {
    ($func_name:ident, $variant:ident, $result:ty) => {
        fn $func_name(&self) -> Option<&$result> {
            match &self.object_type {
                WzObjectType::$variant(inner) => Some(inner),
                _ => None,
            }
        }
    };
}

// 值類型的巨集
macro_rules! try_as_wz_value {
    ($func_name:ident, $variant:ident, $result:ident) => {
        fn $func_name(&self) -> Option<&$result> {
            match &self.object_type {
                WzObjectType::Value(WzValue::$variant(inner)) => Some(inner),
                _ => None,
            }
        }
    };
}

// 使用方式
try_as!(try_as_file, File, WzFile);
try_as_wz_value!(try_as_int, Int, i32);
```

**特殊情況:**

```rust
// PNG 和 Sound 需要巢狀配對
fn try_as_png(&self) -> Option<&WzPng> {
    match &self.object_type {
        WzObjectType::Property(WzSubProperty::PNG(png)) => Some(png),
        _ => None,
    }
}

// String 處理 String 和 UOL 兩種類型
fn try_as_string(&self) -> Option<&WzString> {
    match &self.object_type {
        WzObjectType::Value(WzValue::String(s))
        | WzObjectType::Value(WzValue::UOL(s)) => Some(s),
        _ => None,
    }
}
```

---

## 讀取器元件

### Reader 模組

**位置**: `src/reader.rs` (1368 行)

**用途**: 具有 WZ 特定加密和資料類型的二進位 I/O。

**核心類型:**

```rust
pub struct WzBaseReader<T: AsRef<[u8]>> {
    pub map: T,                           // 記憶體映射資料
    pub wz_iv: [u8; 4],                  // 加密 IV
    pub keys: Arc<RwLock<WzMutableKey>>,  // 共享加密金鑰
}

pub type WzReader = WzBaseReader<Mmap>;

pub struct WzSliceReader<'a> {
    pub buf: &'a [u8],
    pub pos: Cell<usize>,                // 目前位置
    pub header: WzHeader<'a>,
    pub keys: Arc<RwLock<WzMutableKey>>,
}
```

**讀取器特徵:**

```rust
pub trait Reader {
    fn get_decrypt_slice(&self, range: Range<usize>) -> Result<Vec<u8>>;
    fn read_u8_at(&self, pos: usize) -> Result<u8>;
    fn read_i32_at(&self, pos: usize) -> Result<i32>;
    fn resolve_unicode_raw(&self, offset: usize, len: usize) -> Result<Vec<u16>>;
    fn resolve_ascii_raw(&self, offset: usize, len: usize) -> Result<Vec<u8>>;
    // ... 20+ 方法
}
```

**字串解密:**

```rust
// 字串類型偵測
pub enum WzStringType {
    Empty,         // 長度 0
    Unicode,       // 正長度
    Ascii,         // 負長度
}

// Unicode 解密
fn resolve_unicode_raw(&self, offset: usize, length: usize) -> Result<Vec<u16>> {
    let encrypted = self.get_decrypt_slice(offset..offset + length)?;
    let mut decoded = Vec::with_capacity(length / 2);

    for (i, chunk) in encrypted.chunks(2).enumerate() {
        let c = u16::from_le_bytes([chunk[0], chunk[1]]);
        decoded.push(resolve_unicode_char(c, i as i32));
    }

    Ok(decoded)
}

// 字元解密
fn resolve_unicode_char(c: u16, i: i32) -> u16 {
    let key = keys.read().unwrap();
    c ^ key.get_unicode_key(i)
}
```

**壓縮整數 (WZ Int):**

```rust
pub fn read_wz_int(&self) -> Result<i32> {
    let first_byte = self.read_i8()?;

    match first_byte {
        i8::MIN => self.read_i32(),  // 4 位元組整數
        _ => Ok(first_byte as i32),   // 1 位元組整數
    }
}
```

**偏移量計算:**

```rust
pub fn read_wz_offset(&self, hash: usize, offset: Option<usize>) -> Result<usize> {
    let offset = offset.unwrap_or_else(|| self.pos.get());
    let encrypted_offset = self.read_u32_at(offset)? as usize;

    // 使用版本雜湊值解密
    let start_pos = self.header.fstart;
    let decrypted = ((encrypted_offset - start_pos) ^ WZ_OFFSET as usize) + start_pos * 2;

    Ok(decrypted)
}
```

---

## 屬性元件

### PNG 屬性

**位置**: `src/property/png.rs`

**用途**: 具有延遲載入的壓縮圖像資料。

**結構:**

```rust
pub struct WzPng {
    pub width: i32,
    pub height: i32,
    // 私有欄位:
    reader: Arc<WzReader>,
    offset: (usize, usize),  // (data_offset, data_len)
    origin: (i32, i32),      // 畫布原點
    format: i32,              // 壓縮格式
}
```

**壓縮格式:**

```rust
const PNG_FORMAT_4444: i32 = 1;    // 16 位元 ARGB4444
const PNG_FORMAT_8888: i32 = 2;    // 32 位元 ARGB8888
const PNG_FORMAT_565: i32 = 513;   // 16 位元 RGB565
const PNG_FORMAT_DXT3: i32 = 517;  // DXT3 壓縮
const PNG_FORMAT_DXT5: i32 = 2050; // DXT5 壓縮
```

**解壓縮演算法:**

```rust
pub fn get_image(node: &WzNodeArc) -> Result<RgbaImage> {
    let png = node.read().unwrap().try_as_png()?;

    // 讀取壓縮資料
    let compressed = reader.get_slice(offset..offset + len);

    // 使用 zlib 解壓縮
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // 將格式轉換為 RGBA8888
    match png.format {
        PNG_FORMAT_4444 => decode_argb4444(&decompressed, width, height),
        PNG_FORMAT_8888 => decode_argb8888(&decompressed, width, height),
        PNG_FORMAT_565 => decode_rgb565(&decompressed, width, height),
        PNG_FORMAT_DXT3 => decode_dxt3(&decompressed, width, height),
        PNG_FORMAT_DXT5 => decode_dxt5(&decompressed, width, height),
        _ => Err(Error::UnsupportedFormat),
    }
}
```

---

### Sound 屬性

**位置**: `src/property/sound.rs`

**用途**: 音訊檔案儲存 (MP3 或二進位格式)。

**結構:**

```rust
pub struct WzSound {
    pub duration: i32,           // 毫秒
    pub sound_type: WzSoundType,
    // 私有:
    reader: Arc<WzReader>,
    offset: usize,
    data_len: usize,
    header_len: usize,
}

pub enum WzSoundType {
    Mp3,      // 標準 MP3
    Binary,   // 原始音訊資料
}
```

**擷取:**

```rust
pub fn get_bytes(&self) -> Result<Vec<u8>> {
    let data_start = self.offset + self.header_len;
    let data = reader.get_slice(data_start..data_start + self.data_len);

    Ok(data.to_vec())
}

pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let bytes = self.get_bytes()?;
    std::fs::write(path, bytes)?;
    Ok(())
}
```

---

## MS 格式元件

### Snow2 Reader

**位置**: `src/ms/snow2_reader.rs`

**用途**: MS 檔案的串流加密解密。

**實作:**

```rust
pub struct Snow2Reader<'a> {
    pub buf: &'a [u8],
    pub pos: Cell<usize>,
    pub header: MsHeader<'a>,
    decryptor: Snow2Decryptor,
}

impl Snow2Decryptor {
    pub fn decrypt(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            *byte ^= self.next_keystream_byte();
        }
    }

    fn next_keystream_byte(&mut self) -> u8 {
        // SNOW 2.0 串流加密演算法
        // ...
    }
}
```

---

## 工具元件

### Walk 模組

**位置**: `src/util/walk.rs`

**用途**: 樹遍歷工具。

**循序遍歷:**

```rust
pub fn walk_node<F>(node: &WzNodeArc, parse: bool, cb: &F)
where
    F: Fn(&WzNodeArc)
{
    cb(node);  // 訪問節點

    if parse {
        node.write().unwrap().parse(node).ok();
    }

    // 遞迴子節點
    for (_, child) in &node.read().unwrap().children {
        walk_node(child, parse, cb);
    }
}
```

**平行遍歷 (需要 `rayon` 功能):**

```rust
#[cfg(feature = "rayon")]
pub fn walk_node_parallel<F>(node: &WzNodeArc, cb: &F)
where
    F: Fn(&WzNodeArc) + Sync
{
    use rayon::prelude::*;

    cb(node);

    node.read().unwrap().children.par_iter()
        .for_each(|(_, child)| {
            walk_node_parallel(child, cb);
        });
}
```

---

### Resolver 模組

**位置**: `src/util/resolver.rs`

**用途**: 多檔案依賴解析。

**Base.wz + Data.wz 解析:**

```rust
pub fn resolve_base<P>(
    base_path: P,
    link_path: Option<P>,
) -> Result<WzNodeArc>
where
    P: AsRef<Path>
{
    let base_node = WzNode::from_wz_file(base_path, None)?.into_lock();
    base_node.write().unwrap().parse(&base_node)?;

    if let Some(data_path) = link_path {
        let data_node = WzNode::from_wz_file(
            data_path,
            Some(&base_node.read().unwrap().try_as_file()?.reader.keys)
        )?.into_lock();

        data_node.write().unwrap().parse(&data_node)?;

        // 合併樹
        data_node.write().unwrap().transfer_childs(&base_node);
    }

    Ok(base_node)
}
```

---

### Property 解析器

**位置**: `src/util/parse_property.rs`

**用途**: 遞迴屬性清單解析。

**演算法:**

```rust
pub fn parse_property_list(
    parent: Option<&WzNodeArc>,
    reader: &Arc<WzReader>,
    slice_reader: &WzSliceReader,
    base_offset: usize,
) -> Result<(WzNodeArcVec, Vec<WzNodeArc>)> {
    let mut nodes = Vec::new();
    let mut uol_nodes = Vec::new();

    loop {
        // 讀取屬性名稱
        let name = slice_reader.read_wz_string()?;
        if name == "EOP" { break; }  // 屬性結束

        // 讀取屬性類型
        let prop_type = slice_reader.read_u8()?;

        let node = match prop_type {
            0x00 => WzNode::new(&name, WzValue::Null, parent),
            0x0B => parse_numeric_property(slice_reader, &name, parent)?,
            0x04 => parse_float_property(slice_reader, &name, parent)?,
            0x08 => parse_string_property(slice_reader, &name, parent)?,
            0x09 => parse_extended_property(slice_reader, &name, parent)?,
            _ => return Err(Error::UnknownPropertyType(prop_type)),
        };

        nodes.push((name, node.into_lock()));
    }

    Ok((nodes, uol_nodes))
}
```

**擴充屬性類型:**

```rust
fn parse_extended_property(...) -> Result<WzNode> {
    let extended_type = slice_reader.read_wz_string()?;

    match extended_type.as_str() {
        "Property" => parse_sub_property(slice_reader)?,
        "Canvas" => parse_png_property(slice_reader)?,
        "Shape2D#Vector2D" => parse_vector2d(slice_reader)?,
        "Shape2D#Convex2D" => parse_convex(slice_reader)?,
        "Sound_DX8" => parse_sound(slice_reader)?,
        "UOL" => parse_uol(slice_reader)?,
        _ => return Err(Error::UnknownExtendedType(extended_type)),
    }
}
```

---

## 版本系統

**位置**: `src/version.rs` (139 行)

**用途**: 版本偵測和 IV 管理。

**MapleStory 版本:**

```rust
pub enum WzMapleVersion {
    GMS,  // Global MapleStory
    EMS,  // Europe MapleStory
    BMS,  // Brazil MapleStory
    MSEA, // Southeast Asia
    TMS,  // Taiwan MapleStory
    KMS,  // Korea MapleStory
    JMS,  // Japan MapleStory
    CMS,  // China MapleStory
}
```

**IV 表:**

```rust
pub fn get_iv_by_maple_version(version: WzMapleVersion) -> [u8; 4] {
    match version {
        WzMapleVersion::GMS => [0x12, 0x34, 0x56, 0x78],
        WzMapleVersion::EMS => [0x12, 0x34, 0x56, 0x78],
        WzMapleVersion::BMS => [0x12, 0x34, 0x56, 0x78],
        WzMapleVersion::MSEA => [0x12, 0x34, 0x56, 0x78],
        WzMapleVersion::TMS => [0x57, 0x30, 0x96, 0xF1],
        WzMapleVersion::KMS => [0xB9, 0x7D, 0x63, 0xE9],
        WzMapleVersion::JMS => [0xC8, 0x72, 0xCA, 0xE1],
        WzMapleVersion::CMS => [0x2D, 0xB8, 0xAE, 0x3D],
    }
}
```

**自動偵測:**

```rust
pub fn guess_iv_from_wz_file(data: &[u8]) -> Option<[u8; 4]> {
    const KNOWN_IVS: &[[u8; 4]] = &[
        [0x12, 0x34, 0x56, 0x78],  // GMS/EMS/BMS
        [0x57, 0x30, 0x96, 0xF1],  // TMS
        [0xB9, 0x7D, 0x63, 0xE9],  // KMS
        [0xC8, 0x72, 0xCA, 0xE1],  // JMS
        [0x2D, 0xB8, 0xAE, 0x3D],  // CMS
    ];

    for &iv in KNOWN_IVS {
        if verify_iv_from_wz_file(data, &iv) {
            return Some(iv);
        }
    }

    None
}
```

---

## 元件依賴關係

```
┌─────────────────────────────────────────────────────────┐
│                        WzNode                            │
│                    (核心結構)                             │
└───────────────────┬──────────────────────────────────────┘
                    │
        ┌───────────┼───────────────────────┐
        │           │                       │
        ▼           ▼                       ▼
    ┌───────┐  ┌──────────┐        ┌──────────────┐
    │WzFile │  │ WzImage  │        │ WzDirectory  │
    └───┬───┘  └────┬─────┘        └──────┬───────┘
        │           │                      │
        └───────────┼──────────────────────┘
                    │
                    ▼
            ┌──────────────┐
            │  WzReader    │
            │ (記憶體映射)  │
            └──────┬───────┘
                   │
        ┌──────────┼───────────┐
        │          │           │
        ▼          ▼           ▼
   ┌────────┐ ┌────────┐ ┌───────────┐
   │ memmap2│ │ WzKeys │ │WzSliceRdr │
   └────────┘ └────────┘ └───────────┘
```

---

*架構概述請參見 [ARCHITECTURE.md](./ARCHITECTURE.md)*
*API 參考請參見 [API.md](./API.md)*
*範例請參見 [EXAMPLES.md](./EXAMPLES.md)*
