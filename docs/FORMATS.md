# 檔案格式文件

> **最後更新**: 2026 年 1 月 18 日
> **版本**: 0.0.16

## 目錄

- [概述](#概述)
- [WZ 檔案格式](#wz-檔案格式)
- [IMG 檔案格式](#img-檔案格式)
- [MS 檔案格式](#ms-檔案格式)
- [加密系統](#加密系統)
- [屬性編碼](#屬性編碼)
- [圖像格式](#圖像格式)
- [音訊格式](#音訊格式)
- [資料型別](#資料型別)

---

## 概述

MapleStory 使用專有檔案格式來儲存遊戲資源：

- **.wz** - 包含目錄和圖像的封存檔案
- **.img** - 圖像容器檔案（可獨立存在或位於 .wz 內）
- **.ms** - MapleStoryM 格式，使用不同的加密方式

所有格式都使用：
- **小端序（Little-endian）** 位元組順序
- **自訂加密**（.wz 使用 AES 加密，.ms 使用串流加密）
- **Zlib 壓縮** 用於圖像資料
- **變長整數** 用於緊湊儲存

---

## WZ 檔案格式

### 檔案結構

```
[WZ 標頭]
[目錄項目]
  ├─ [子目錄或圖像]*
  └─ [屬性資料]
```

### WZ 標頭

**偏移量 0x00**（固定大小：變動）

```c
struct WzHeader {
    char signature[4];     // "PKG1"
    uint64_t file_size;    // 檔案總大小
    uint32_t fstart;       // 資料起始偏移量
    char copyright[61];    // 版權字串（選用）
};
```

**簽章**：永遠為 `50 4B 47 31`（"PKG1"）

**檔案大小**：8 位元組，小端序

**FStart**：目錄結構開始的偏移量

### 版本標頭

**偏移量：fstart**

```c
// 32 位元用戶端（最常見）
struct VersionHeader {
    uint16_t encrypt_version;  // 加密版本
    // ... 目錄項目接續
};

// 64 位元用戶端（現代版本）
// 沒有 encrypt_version，直接從項目開始
```

**版本偵測：**
- 若 `encrypt_version > 0xff` → 64 位元用戶端
- 若 `encrypt_version == 0x80` → 檢查額外欄位
- 否則 → 32 位元用戶端帶加密版本

### 目錄結構

**格式：**

```c
struct DirectoryEntry {
    uint8_t type;          // 1, 2, 3, 或 4
    // 名稱（依類型而異）
    int32_t file_size;     // 壓縮的 WZ 整數
    int32_t checksum;      // 壓縮的 WZ 整數
    uint32_t offset;       // WZ 偏移量計算
};
```

**項目類型：**

| 類型 | 名稱 | 描述 |
|------|------|-------------|
| 1 | UnknownType | 跳過項目（未使用） |
| 2 | RetrieveStringFromOffset | 名稱位於字串表偏移量 |
| 3 | WzDirectory | 子目錄 |
| 4 | WzImage | 圖像容器（.img） |

**類型 2（字串表）：**
```c
int32_t str_offset;        // 字串表中的偏移量
// 從此處讀取名稱：fstart + str_offset
```

**類型 3 & 4（直接名稱）：**
```c
WzString name;             // 加密字串
```

### 偏移量計算

**WZ 偏移量解密：**
```rust
fn decrypt_offset(encrypted: u32, hash: u32, fstart: u32) -> u32 {
    const WZ_OFFSET: u32 = 0x581C3F6D;
    ((encrypted - fstart) ^ WZ_OFFSET) + (fstart * 2)
}
```

**雜湊計算：**
```rust
fn calculate_hash(patch_version: i32) -> i32 {
    let mut hash: i32 = 0;
    for ch in patch_version.to_string().chars() {
        let code = ch.to_ascii_lowercase() as i32;
        hash = hash * 32 + code + 1;
    }
    hash
}
```

### 版本雜湊驗證

```rust
fn verify_version(encver: i32, patch_version: i32) -> bool {
    let hash = calculate_hash(patch_version);

    // 直接匹配
    if encver == patch_version {
        return true;
    }

    // XOR 驗證
    let enc = 0xff
        ^ (hash >> 24) & 0xff
        ^ (hash >> 16) & 0xff
        ^ (hash >> 8) & 0xff
        ^ hash & 0xff;

    enc == encver
}
```

---

## IMG 檔案格式

### IMG 標頭

**偏移量 0x00**

```c
struct ImgHeader {
    uint8_t header_byte;   // 0x73, 0x1B, 0x01, 或 0x23
    // 內容依標頭類型而異
};
```

**標頭類型：**

| 位元組 | 類型 | 描述 |
|------|------|-------------|
| 0x73 | Standard | 無偏移量表的屬性列表 |
| 0x1B | WithOffset | 含偏移量表的屬性列表 |
| 0x01 | Lua | Lua 腳本檔案 |
| 0x23 | RawText | 以 "#Property" 開始的原始文字 |

### 標準圖像（0x73）

```c
struct StandardImage {
    uint8_t header;        // 0x73
    WzString magic;        // "Property"
    uint16_t zero;         // 0x0000
    // 屬性列表接續
};
```

### Lua 圖像（0x01）

```c
struct LuaImage {
    uint8_t header;        // 0x01
    int32_t script_len;    // WZ 壓縮整數
    uint8_t script_data[script_len];  // Lua 腳本
};
```

### 原始文字（0x23 '#'）

```
#Property
{ key = value, key = value, ... }
```

格式類似 JSON 但使用自訂語法。

---

## MS 檔案格式

MapleStoryM 使用不同格式，採用更強的加密。

### MS 標頭

```c
struct MsHeader {
    char signature[4];     // "PKG1"（Snow2）或 "MSFD"（ChaCha20）
    uint32_t version;      // 格式版本
    uint64_t file_size;
    uint32_t data_offset;
    // 加密特定欄位
};
```

**加密偵測：**
- `50 4B 47 31`（"PKG1"）→ Snow2 加密
- `4D 53 46 44`（"MSFD"）→ ChaCha20 加密

### Snow2 加密

基於 SNOW 2.0 演算法的串流加密。

**金鑰排程：**
```rust
struct Snow2State {
    lfsr: [u32; 16],       // 線性回饋移位暫存器
    fsr: [u32; 2],         // 有限狀態暫存器
}
```

**解密：**
```rust
fn decrypt_snow2(data: &mut [u8], key: &[u8]) {
    let mut state = Snow2State::init(key);
    for byte in data.iter_mut() {
        *byte ^= state.next_keystream_byte();
    }
}
```

### ChaCha20 加密

使用 256 位元金鑰的 ChaCha20 串流加密。

**初始化：**
```rust
use c2_chacha::ChaCha20;

let cipher = ChaCha20::new(&key, &nonce);
cipher.apply_keystream(data);
```

---

## 加密系統

### AES 加密（WZ 格式）

**金鑰生成：**

```rust
const AES_KEY: [u8; 32] = [
    0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
    0x06, 0x00, 0x00, 0x00, 0xB4, 0x00, 0x00, 0x00,
    0x1B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00,
    0x33, 0x00, 0x00, 0x00, 0x52, 0x00, 0x00, 0x00,
];

fn generate_wz_key(iv: [u8; 4]) -> [u8; 32] {
    // 使用 AES_KEY 擴展 IV 為 32 位元組金鑰
    let mut key = [0u8; 32];
    for i in 0..key.len() {
        key[i] = AES_KEY[i] ^ iv[i % 4];
    }
    key
}
```

**各 MapleStory 版本的 IV：**

| 版本 | IV（十六進位） | IV（位元組） |
|---------|----------|------------|
| GMS/EMS/BMS | `12 34 56 78` | [0x12, 0x34, 0x56, 0x78] |
| TMS | `57 30 96 F1` | [0x57, 0x30, 0x96, 0xF1] |
| KMS | `B9 7D 63 E9` | [0xB9, 0x7D, 0x63, 0xE9] |
| JMS | `C8 72 CA E1` | [0xC8, 0x72, 0xCA, 0xE1] |
| CMS | `2D B8 AE 3D` | [0x2D, 0xB8, 0xAE, 0x3D] |

### 字串加密

**Unicode 字串：**

```rust
fn decrypt_unicode(data: &[u8], key: &WzMutableKey, pos: usize) -> Vec<u16> {
    let mut result = Vec::new();
    for (i, chunk) in data.chunks(2).enumerate() {
        let c = u16::from_le_bytes([chunk[0], chunk[1]]);
        let key_byte = key.get_unicode_key(pos + i);
        result.push(c ^ key_byte);
    }
    result
}
```

**ASCII 字串：**

```rust
fn decrypt_ascii(data: &[u8], key: &WzMutableKey, pos: usize) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| {
            let key_byte = key.get_ascii_key(pos + i);
            byte ^ key_byte
        })
        .collect()
}
```

**金鑰生成（滾動 XOR）：**

```rust
impl WzMutableKey {
    fn get_unicode_key(&mut self, index: usize) -> u16 {
        let key = self.unicode_key[index % self.unicode_key.len()];
        self.shuffle();  // 更新金鑰狀態
        key
    }

    fn shuffle(&mut self) {
        // 旋轉並混合金鑰位元組
        // 實作細節依版本而異
    }
}
```

---

## 屬性編碼

### 屬性列表結構

```
[屬性數量：WZ 整數]
[屬性項目]*
"EOP"  // 屬性結束
```

### 屬性項目格式

```c
struct PropertyEntry {
    WzString name;         // 屬性名稱
    uint8_t type;          // 屬性類型位元組
    // 值（依類型而異）
};
```

### 屬性類型

| 類型 | 名稱 | 描述 |
|------|------|-------------|
| 0x00 | Null | 無值 |
| 0x0B | Number | Short、Int 或 Long |
| 0x04 | Float | Float 或 Double |
| 0x08 | String | 加密字串 |
| 0x09 | Extended | 擴展屬性（見下文） |

### 擴展屬性類型

在類型 0x09 之後讀取：

```c
WzString extended_type;    // 擴展類型名稱

// 依據 extended_type：
"Property"           → 子屬性容器
"Canvas"             → PNG 圖像
"Shape2D#Vector2D"   → 2D 向量（x, y）
"Shape2D#Convex2D"   → 多邊形資料
"Sound_DX8"          → 音訊檔案
"UOL"                → 路徑參照（字串）
```

### 數字編碼（0x0B）

```c
int8_t format = read_i8();

if (format == 0) {
    value = 0;
} else if (format == -128) {
    value = read_i64();  // Long
} else if (format < 0) {
    value = read_i32();  // Int
} else {
    value = format;      // Short（儲存於格式位元組）
}
```

### 浮點數編碼（0x04）

```c
uint8_t format = read_u8();

if (format == 0x80) {
    value = read_f32();    // Float
} else if (format == 0) {
    value = 0.0f;
} else {
    value = format;        // 小浮點數
}
```

### Canvas（PNG）編碼

```c
struct Canvas {
    uint8_t unknown;       // 永遠為 0x00
    uint8_t has_property;  // 若有子屬性則為 0x01

    if (has_property == 0x01) {
        // 讀取子屬性（寬度、高度、原點等）
        read_property_list();
        WzString magic = read_wz_string();  // "PNG"
    }

    int32_t width = read_wz_int();
    int32_t height = read_wz_int();
    int32_t format = read_wz_int();
    uint8_t unknown2 = read_u8();
    int32_t data_len = read_i32() - 1;

    uint8_t header = read_u8();  // Zlib 標頭
    uint8_t compressed_data[data_len];
};
```

### 音訊編碼

```c
struct Sound {
    uint8_t unknown;       // 永遠為 0x00
    int32_t data_len = read_wz_int();
    int32_t duration = read_wz_int();

    // 標頭依格式而異
    uint8_t header[82];    // MP3 或二進位標頭
    uint8_t audio_data[data_len];
};
```

---

## 圖像格式

### PNG 壓縮格式

| 格式 ID | 名稱 | 位元/像素 | 描述 |
|-----------|------|------------|-------------|
| 1 | ARGB4444 | 16 | 4 位元 alpha、RGB |
| 2 | ARGB8888 | 32 | 8 位元 alpha、RGB |
| 513 | RGB565 | 16 | 無 alpha，5-6-5 RGB |
| 517 | DXT3 | 4 | 帶 alpha 的區塊壓縮 |
| 2050 | DXT5 | 4 | 帶內插 alpha 的區塊壓縮 |

### 解壓縮流程

```
1. 讀取壓縮資料
2. 使用 zlib 解壓縮
3. 依據格式解碼：
   - ARGB4444：將 4 位元擴展為 8 位元
   - ARGB8888：直接複製（BGRA → RGBA）
   - RGB565：將 5/6 位元擴展為 8 位元
   - DXT3/DXT5：區塊解壓縮
4. 輸出 RGBA8888
```

### ARGB4444 解碼

```rust
fn decode_argb4444(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for i in 0..(width * height) as usize {
        let pixel = u16::from_le_bytes([data[i*2], data[i*2+1]]);

        let a = ((pixel >> 12) & 0xF) as u8;
        let r = ((pixel >> 8) & 0xF) as u8;
        let g = ((pixel >> 4) & 0xF) as u8;
        let b = (pixel & 0xF) as u8;

        // 將 4 位元擴展為 8 位元：值 * 17
        rgba[i*4 + 0] = r * 17;
        rgba[i*4 + 1] = g * 17;
        rgba[i*4 + 2] = b * 17;
        rgba[i*4 + 3] = a * 17;
    }

    rgba
}
```

### ARGB8888 解碼

```rust
fn decode_argb8888(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for i in 0..(width * height) as usize {
        // BGRA → RGBA
        rgba[i*4 + 0] = data[i*4 + 2];  // R ← B
        rgba[i*4 + 1] = data[i*4 + 1];  // G
        rgba[i*4 + 2] = data[i*4 + 0];  // B ← R
        rgba[i*4 + 3] = data[i*4 + 3];  // A
    }

    rgba
}
```

### DXT5 解壓縮

DXT5 使用 4x4 像素區塊並帶內插 alpha：

```rust
fn decode_dxt5_block(block: &[u8; 16]) -> [u8; 64] {
    let alpha0 = block[0];
    let alpha1 = block[1];

    // 內插 6 個額外的 alpha 值
    let alpha_table = interpolate_alpha(alpha0, alpha1);

    // 解碼 3 位元 alpha 索引
    let alpha_indices = decode_alpha_indices(&block[2..8]);

    // 解碼 RGB565 顏色
    let color0 = u16::from_le_bytes([block[8], block[9]]);
    let color1 = u16::from_le_bytes([block[10], block[11]]);

    // 內插顏色並合併
    decode_block_colors(color0, color1, &block[12..16], alpha_indices, alpha_table)
}
```

---

## 音訊格式

### MP3 格式

```c
struct Mp3Sound {
    uint8_t header[82];    // MP3 標頭
    // 標準 MP3 資料接續
};
```

### 二進位格式

```c
struct BinarySound {
    uint8_t header[82];    // 自訂標頭
    uint8_t pcm_data[];    // 原始 PCM 或壓縮音訊
};
```

---

## 資料型別

### WZ 壓縮整數

變長整數編碼：

```c
int8_t first = read_i8();

if (first == -128) {
    // 4 位元組整數
    return read_i32();
} else {
    // 1 位元組整數
    return first;
}
```

**範圍：**
- `-127` 至 `127` → 1 位元組
- 其他值 → 5 位元組（標記 + int32）

### WZ 字串

帶長度前綴的加密字串：

```c
int8_t len_marker = read_i8();

if (len_marker == 0) {
    return "";  // 空字串
} else if (len_marker > 0) {
    // Unicode 字串
    int length = len_marker * 2;
    uint8_t encrypted[length];
    return decrypt_unicode(encrypted);
} else if (len_marker == -128) {
    // 長 Unicode 字串
    int length = read_i32() * 2;
    uint8_t encrypted[length];
    return decrypt_unicode(encrypted);
} else {
    // ASCII 字串
    int length = -len_marker;
    if (length == 127) {
        length = read_i32();
    }
    uint8_t encrypted[length];
    return decrypt_ascii(encrypted);
}
```

**編碼：**
- `0`：空字串
- `1` 至 `127`：Unicode，N 個字元（N*2 位元組）
- `-128`：長 Unicode（後接 4 位元組長度）
- `-1` 至 `-127`：ASCII，N 個字元
- `-128`（在第一個 -127 之後）：長 ASCII

### Vector2D

```c
struct Vector2D {
    int32_t x;  // WZ 壓縮整數
    int32_t y;  // WZ 壓縮整數
};
```

---

## 檔案格式範例

### 最小 WZ 檔案

```
50 4B 47 31              // "PKG1" 簽章
00 00 00 00 00 00 00 00  // 檔案大小佔位符
3C 00 00 00              // FStart（60）
...copyright...
83 00                    // 加密版本（131 = v83）
01 00 00 00              // 1 個項目
04                       // 類型 4（圖像）
04 54 65 73 74           // 字串："Test"
01                       // 大小：1
00                       // 校驗和：0
3C 00 00 00              // 偏移量
```

### 最小屬性列表

```
02 00 00 00              // 2 個屬性
03 69 64                 // 名稱："id"
0B                       // 類型：Number
2A                       // 值：42

04 6E 61 6D 65           // 名稱："name"
08                       // 類型：String
05 48 65 6C 6C 6F        // 值："Hello"

45 4F 50                 // "EOP"
```

---

## 參考資料

- **WzComparerR2**: https://github.com/Kagamia/WzComparerR2
- **MapleLib**: https://github.com/lastbattle/MapleLib
- **HaRepacker**: https://github.com/lastbattle/Harepacker-resurrected

---

*實作細節請參閱 [COMPONENTS.md](./COMPONENTS.md)*
*API 使用方式請參閱 [API.md](./API.md)*
*架構資訊請參閱 [ARCHITECTURE.md](./ARCHITECTURE.md)*
