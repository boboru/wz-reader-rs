# API 參考文件

> **最後更新**: 2026 年 1 月 18 日
> **版本**: 0.0.16

## 目錄

- [核心型別](#核心型別)
  - [WzNode](#wznode)
  - [WzNodeArc](#wznodearc)
  - [WzFile](#wzfile)
  - [WzImage](#wzimage)
  - [MsFile](#msfile)
- [物件型別系統](#物件型別系統)
  - [WzObjectType](#wzobjecttype)
  - [WzValue](#wzvalue)
  - [WzSubProperty](#wzsubproperty)
- [屬性型別](#屬性型別)
  - [WzPng](#wzpng)
  - [WzSound](#wzsound)
  - [WzString](#wzstring)
  - [Vector2D](#vector2d)
- [讀取器系統](#讀取器系統)
  - [WzReader](#wzreader)
  - [WzSliceReader](#wzslicereader)
  - [Reader Trait](#reader-trait)
- [Traits](#traits)
  - [NodeCast](#nodecast)
- [工具函式](#工具函式)
  - [樹狀結構遍歷](#樹狀結構遍歷)
  - [路徑解析](#路徑解析)
  - [節點工具](#節點工具)
- [錯誤型別](#錯誤型別)

---

## 核心型別

### WzNode

代表 WZ 檔案樹狀結構中單一節點的基本建構單元。

**定義** (src/node.rs:36):
```rust
pub struct WzNode {
    pub name: WzNodeName,
    pub object_type: WzObjectType,
    pub parent: Weak<RwLock<WzNode>>,
    pub children: HashMap<WzNodeName, Arc<RwLock<WzNode>>>,
}
```

#### 建構函式

##### `new()`
```rust
pub fn new(
    name: &WzNodeName,
    object_type: impl Into<WzObjectType>,
    parent: Option<&WzNodeArc>,
) -> Self
```

建立一個具有指定名稱和型別的新 WzNode。

**範例:**
```rust
use wz_reader::{WzNode, WzNodeName};

let root = WzNode::new(&"root".into(), 42, None).into_lock();
let child = WzNode::new(&"child".into(), 100, Some(&root)).into_lock();
```

##### `from_str()`
```rust
pub fn from_str(
    name: &str,
    object_type: impl Into<WzObjectType>,
    parent: Option<&WzNodeArc>,
) -> Self
```

使用 `&str` 作為名稱的便利建構函式。

**範例:**
```rust
let node = WzNode::from_str("test", 1, None);
```

##### `from_wz_file()`
```rust
pub fn from_wz_file<P>(
    path: P,
    parent: Option<&WzNodeArc>,
) -> Result<Self, Error>
where
    P: AsRef<Path>
```

從 .wz 檔案建立 WzNode，並自動偵測版本。

**範例:**
```rust
let node = WzNode::from_wz_file("Base.wz", None)?;
let node_arc = node.into_lock();
```

##### `from_wz_file_full()`
```rust
pub fn from_wz_file_full<P>(
    path: P,
    version: Option<version::WzMapleVersion>,
    patch_version: Option<i32>,
    parent: Option<&WzNodeArc>,
    existing_key: Option<&SharedWzMutableKey>,
) -> Result<Self, Error>
```

從 .wz 檔案建立 WzNode，並提供明確的版本控制和金鑰共用功能。

**參數:**
- `path`: .wz 檔案的路徑
- `version`: 可選的 MapleStory 版本 (GMS, EMS, BMS 等)
- `patch_version`: 可選的補丁版本號
- `parent`: 可選的父節點
- `existing_key`: 可選的共用加密金鑰，用於多檔案場景

**範例:**
```rust
use wz_reader::version::WzMapleVersion;

let base = WzNode::from_wz_file_full(
    "Base.wz",
    Some(WzMapleVersion::GMS),
    Some(83),
    None,
    None
)?;
```

##### `from_ms_file()`
```rust
pub fn from_ms_file<P>(
    path: P,
    parent: Option<&WzNodeArc>,
) -> Result<Self, Error>
where
    P: AsRef<Path>
```

從 MapleStoryM .ms 檔案建立 WzNode。

**範例:**
```rust
let node = WzNode::from_ms_file("Data.ms", None)?;
```

##### `from_img_file()`
```rust
pub fn from_img_file<P>(
    path: P,
    version: Option<version::WzMapleVersion>,
    parent: Option<&WzNodeArc>,
) -> Result<Self, Error>
```

從獨立的 .img 檔案建立 WzNode。

**範例:**
```rust
let img_node = WzNode::from_img_file("00002000.img", None, None)?;
```

#### 節點操作

##### `parse()`
```rust
pub fn parse(&mut self, parent: &WzNodeArc) -> Result<(), Error>
```

根據節點的物件型別解析其子節點。這是一個延遲操作 - 節點只在明確請求時才會被解析。

**範例:**
```rust
let node = WzNode::from_wz_file("Base.wz", None)?.into_lock();
node.write().unwrap().parse(&node)?;
```

##### `unparse()`
```rust
pub fn unparse(&mut self)
```

清除節點的子節點並標記為未解析狀態以釋放記憶體。

**範例:**
```rust
node.write().unwrap().unparse();
```

##### `add()`
```rust
pub fn add(&mut self, node: &WzNodeArc)
```

新增一個子節點。

**範例:**
```rust
let parent = WzNode::from_str("parent", 1, None).into_lock();
let child = WzNode::from_str("child", 2, Some(&parent)).into_lock();

parent.write().unwrap().add(&child);
```

#### 路徑操作

##### `get_full_path()`
```rust
pub fn get_full_path(&self) -> String
```

回傳從根節點到此節點的完整路徑。

**範例:**
```rust
let path = node.read().unwrap().get_full_path();
// 回傳: "Base/Character/00002000.img/stand/0"
```

##### `get_path_from_root()`
```rust
pub fn get_path_from_root(&self) -> String
```

回傳排除根節點名稱的路徑。

**範例:**
```rust
let path = node.read().unwrap().get_path_from_root();
// 回傳: "Character/00002000.img/stand/0"
```

##### `get_path_from_image()`
```rust
pub fn get_path_from_image(&self) -> String
```

回傳從最近的 WzImage 祖先節點開始的路徑。

**範例:**
```rust
let path = node.read().unwrap().get_path_from_image();
// 回傳: "stand/0"
```

#### 導航方法

##### `at()`
```rust
pub fn at(&self, name: &str) -> Option<WzNodeArc>
```

根據名稱取得直接子節點。

**範例:**
```rust
let child = node.read().unwrap().at("Character");
```

##### `at_path()`
```rust
pub fn at_path(&self, path: &str) -> Option<WzNodeArc>
```

根據路徑取得後代節點（不解析中間節點）。

**範例:**
```rust
let node = root.read().unwrap()
    .at_path("Character/00002000.img")?;
```

##### `at_path_parsed()`
```rust
pub fn at_path_parsed(&self, path: &str) -> Result<WzNodeArc, Error>
```

根據路徑取得後代節點，並解析所有中間節點。

**範例:**
```rust
let node = root.read().unwrap()
    .at_path_parsed("Character/00002000.img/stand/0")?;
```

##### `at_relative()`
```rust
pub fn at_relative(&self, path: &str) -> Option<WzNodeArc>
```

使用相對路徑表示法取得節點（支援 ".."）。

**範例:**
```rust
let parent = node.read().unwrap().at_relative("..")?;
```

##### `at_path_relative()`
```rust
pub fn at_path_relative(&self, path: &str) -> Option<WzNodeArc>
```

使用相對路徑取得節點。

**範例:**
```rust
let sibling = node.read().unwrap()
    .at_path_relative("../other_node")?;
```

#### 父節點導航

##### `filter_parent()`
```rust
pub fn filter_parent<F>(&self, cb: F) -> Option<WzNodeArc>
where
    F: Fn(&WzNode) -> bool
```

尋找第一個符合條件的父節點。

**範例:**
```rust
let base_file = node.read().unwrap()
    .filter_parent(|n| n.name.as_str() == "Base");
```

##### `get_parent_wz_image()`
```rust
pub fn get_parent_wz_image(&self) -> Option<WzNodeArc>
```

取得最近的 WzImage 祖先節點。

**範例:**
```rust
let image = node.read().unwrap().get_parent_wz_image()?;
```

##### `get_base_wz_file()`
```rust
pub fn get_base_wz_file(&self) -> Option<WzNodeArc>
```

如果父節點鏈中存在 Base.wz 檔案節點，則取得該節點。

#### 序列化（需要 `json` 功能）

##### `to_json()`
```rust
#[cfg(feature = "json")]
pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>
```

產生包含所有中繼資料的完整 JSON 表示。

**範例:**
```rust
let json = node.read().unwrap().to_json()?;
```

##### `to_simple_json()`
```rust
#[cfg(feature = "json")]
pub fn to_simple_json(&self) -> Result<serde_json::Value, serde_json::Error>
```

產生僅包含名稱和值的簡化 JSON。

**範例:**
```rust
let json = node.read().unwrap().to_simple_json()?;
// 回傳: {"stand": {"0": 42, "1": 43}}
```

---

### WzNodeArc

執行緒安全的 WzNode 參考計數指標。

**定義** (src/node.rs:46):
```rust
pub type WzNodeArc = Arc<RwLock<WzNode>>;
pub type WzNodeArcVec = Vec<(WzNodeName, WzNodeArc)>;
```

**使用方式:**
```rust
// 將 WzNode 轉換為 WzNodeArc
let node_arc: WzNodeArc = node.into_lock();

// 讀取存取
{
    let read_guard = node_arc.read().unwrap();
    println!("{}", read_guard.name);
}

// 寫入存取
{
    let mut write_guard = node_arc.write().unwrap();
    write_guard.parse(&node_arc)?;
}
```

---

### WzFile

代表一個 .wz 封存檔案。

**定義** (src/file.rs:51):
```rust
pub struct WzFile {
    pub reader: Arc<WzReader>,
    pub offset: usize,
    pub block_size: usize,
    pub is_parsed: bool,
    pub wz_file_meta: WzFileMeta,
}

pub struct WzFileMeta {
    pub path: String,
    pub patch_version: i32,
    pub wz_version_header: i32,
    pub wz_with_encrypt_version_header: bool,
    pub hash: usize,
}
```

#### 方法

##### `from_file()`
```rust
pub fn from_file<P>(
    path: P,
    wz_iv: Option<[u8; 4]>,
    patch_version: Option<i32>,
    existing_key: Option<&SharedWzMutableKey>,
) -> Result<WzFile, Error>
```

開啟一個 .wz 檔案，並可選擇性地指定版本參數。

**範例:**
```rust
// 自動偵測版本
let wz_file = WzFile::from_file("Base.wz", None, None, None)?;

// 明確指定 IV
let wz_file = WzFile::from_file(
    "Base.wz",
    Some([0x12, 0x34, 0x56, 0x78]),
    Some(83),
    None
)?;
```

##### `parse()`
```rust
pub fn parse(
    &mut self,
    parent: &WzNodeArc,
    patch_version: Option<i32>,
) -> Result<WzNodeArcVec, Error>
```

解析檔案結構並回傳子節點。

---

### WzImage

代表 .wz 檔案中的 .img 容器。

**定義** (src/wz_image.rs:37):
```rust
pub struct WzImage {
    pub reader: Arc<WzReader>,
    pub name: WzNodeName,
    pub offset: usize,
    pub block_size: usize,
    pub is_parsed: bool,
}
```

#### 方法

##### `from_file()`
```rust
pub fn from_file<P>(
    path: P,
    wz_iv: Option<[u8; 4]>
) -> Result<Self, Error>
```

開啟一個獨立的 .img 檔案。

**範例:**
```rust
let img = WzImage::from_file("00002000.img", None)?;
```

##### `at_path()`
```rust
pub fn at_path(&self, path: &str) -> Result<WzNodeArc, Error>
```

直接存取節點而不解析整個映像檔。

**範例:**
```rust
let node = image.at_path("stand/0")?;
```

##### `resolve_children()`
```rust
pub fn resolve_children(
    &self,
    parent: Option<&WzNodeArc>,
) -> Result<(WzNodeArcVec, Vec<WzNodeArc>), Error>
```

解析映像檔中的所有子節點。

---

### MsFile

代表一個 MapleStoryM .ms 檔案。

**定義** (src/ms/file.rs):
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

#### 方法

##### `from_file()`
```rust
pub fn from_file<P>(path: P) -> Result<Self, Error>
```

開啟一個 .ms 檔案並自動偵測加密方式。

**範例:**
```rust
let ms_file = MsFile::from_file("Data.ms")?;
```

---

## 物件型別系統

### WzObjectType

代表所有可能的 WZ 物件型別的型別安全列舉。

**定義** (src/object.rs:25):
```rust
pub enum WzObjectType {
    File(Box<WzFile>),
    MsFile(Box<MsFile>),
    Image(Box<WzImage>),
    MsImage(Box<MsImage>),
    Directory(Box<WzDirectory>),
    Property(WzSubProperty),
    Value(WzValue),
}
```

**自動轉換:**
```rust
// 基本型別
let obj: WzObjectType = 42.into();                    // Int
let obj: WzObjectType = 3.14_f32.into();             // Float
let obj: WzObjectType = "text".to_string().into();   // String

// 檔案
let obj: WzObjectType = wz_file.into();              // File
let obj: WzObjectType = wz_image.into();             // Image

// 屬性
let obj: WzObjectType = wz_png.into();               // Property(PNG)
```

---

### WzValue

WZ 檔案中的基本值型別。

**定義** (src/property/mod.rs:60):
```rust
pub enum WzValue {
    Null,
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(WzString),
    Vector(Vector2D),
    UOL(WzString),              // 路徑參考
    Lua(WzLua),
    Video(WzVideo),
    RawData(WzRawData),
    ParsedString(String),
}
```

**使用方式:**
```rust
use wz_reader::WzNodeCast;

let node = /* ... */;
let read = node.read().unwrap();

if let Some(&value) = read.try_as_int() {
    println!("整數值: {}", value);
}
```

---

### WzSubProperty

可以包含子節點的複雜屬性型別。

**定義** (src/property/mod.rs:48):
```rust
pub enum WzSubProperty {
    PNG(Box<WzPng>),
    Sound(Box<WzSound>),
    Convex,
    Property,
}
```

---

## 屬性型別

### WzPng

代表一個壓縮的圖片。

**定義** (src/property/png.rs):
```rust
pub struct WzPng {
    pub width: i32,
    pub height: i32,
    // 內部欄位...
}
```

#### 方法

##### `get_image()`
```rust
pub fn get_image(node: &WzNodeArc) -> Result<RgbaImage, Error>
```

提取並解壓縮圖片資料。

**範例:**
```rust
use wz_reader::property::get_image;

if let Some(_) = node.read().unwrap().try_as_png() {
    let image = get_image(&node)?;
    image.save("output.png")?;
}
```

---

### WzSound

代表音訊資料。

**定義** (src/property/sound.rs):
```rust
pub struct WzSound {
    pub duration: i32,
    pub sound_type: WzSoundType,
    // 內部欄位...
}

pub enum WzSoundType {
    Mp3,
    Binary,
}
```

#### 方法

##### `get_bytes()`
```rust
pub fn get_bytes(&self) -> Result<Vec<u8>, Error>
```

提取原始音訊資料。

**範例:**
```rust
if let Some(sound) = node.read().unwrap().try_as_sound() {
    let audio_data = sound.get_bytes()?;
    std::fs::write("sound.mp3", audio_data)?;
}
```

##### `save()`
```rust
pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Error>
```

將音效儲存至檔案。

**範例:**
```rust
sound.save("output.mp3")?;
```

---

### WzString

加密的字串值。

**定義** (src/property/string.rs):
```rust
pub struct WzString {
    // 用於延遲解密的內部欄位
}
```

#### 方法

##### `get_string()`
```rust
pub fn get_string(&self) -> Result<String, Error>
```

解密並回傳字串值。

**範例:**
```rust
if let Some(wz_str) = node.read().unwrap().try_as_string() {
    let text = wz_str.get_string()?;
    println!("{}", text);
}
```

---

### Vector2D

二維座標型別。

**定義** (src/property/vector.rs):
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D(pub i32, pub i32);
```

#### 方法

```rust
impl Vector2D {
    pub fn new(x: i32, y: i32) -> Self;
    pub fn x(&self) -> i32;
    pub fn y(&self) -> i32;
}
```

**範例:**
```rust
if let Some(vec) = node.read().unwrap().try_as_vector2d() {
    println!("位置: ({}, {})", vec.x(), vec.y());
}
```

---

## 讀取器系統

### WzReader

記憶體映射檔案讀取器。

**定義** (src/reader.rs:36):
```rust
pub type WzReader = WzBaseReader<Mmap>;

pub struct WzBaseReader<T: AsRef<[u8]>> {
    pub map: T,
    pub wz_iv: [u8; 4],
    pub keys: Arc<RwLock<WzMutableKey>>,
}
```

#### 方法

##### `new()`
```rust
pub fn new(map: Mmap) -> Self
```

##### `with_iv()`
```rust
pub fn with_iv(self, iv: [u8; 4]) -> Self
```

建構器方法，用於設定加密 IV。

##### `with_existing_keys()`
```rust
pub fn with_existing_keys(self, keys: SharedWzMutableKey) -> Self
```

在多個讀取器之間共用加密金鑰。

**範例:**
```rust
let base = WzFile::from_file("Base.wz", None, None, None)?;
let ui = WzFile::from_file(
    "UI.wz",
    None,
    None,
    Some(&base.reader.keys)
)?;
```

##### `create_slice_reader()`
```rust
pub fn create_slice_reader(&self) -> WzSliceReader
```

建立一個用於循序讀取的定位讀取器。

---

### WzSliceReader

用於循序二進位操作的定位讀取器。

**定義** (src/reader.rs:54):
```rust
pub struct WzSliceReader<'a> {
    pub buf: &'a [u8],
    pub pos: Cell<usize>,
    pub header: WzHeader<'a>,
    pub keys: Arc<RwLock<WzMutableKey>>,
}
```

#### 方法

##### `seek()`
```rust
pub fn seek(&self, pos: usize)
```

設定讀取位置。

##### `skip()`
```rust
pub fn skip(&self, count: usize)
```

向前跳過指定的位元組數。

##### `read_u8()`, `read_i32()` 等
```rust
pub fn read_u8(&self) -> Result<u8>;
pub fn read_i16(&self) -> Result<i16>;
pub fn read_i32(&self) -> Result<i32>;
// ... 更多方法
```

##### `read_wz_string()`
```rust
pub fn read_wz_string(&self) -> Result<String>
```

讀取並解密 WZ 字串。

##### `read_wz_int()`
```rust
pub fn read_wz_int(&self) -> Result<i32>
```

讀取 WZ 壓縮整數。

---

### Reader Trait

所有讀取器型別的通用介面。

**定義** (src/reader.rs:65):
```rust
pub trait Reader {
    fn get_size(&self) -> usize;
    fn get_decrypt_slice(&self, range: Range<usize>) -> Result<Vec<u8>>;
    fn read_u8_at(&self, pos: usize) -> Result<u8>;
    fn read_i32_at(&self, pos: usize) -> Result<i32>;
    // ... 20+ 個方法
}
```

---

## Traits

### NodeCast

WzNode 物件的安全型別轉換。

**定義** (src/node_cast.rs:17):
```rust
pub trait WzNodeCast {
    fn try_as_file(&self) -> Option<&WzFile>;
    fn try_as_directory(&self) -> Option<&WzDirectory>;
    fn try_as_image(&self) -> Option<&WzImage>;
    fn try_as_png(&self) -> Option<&WzPng>;
    fn try_as_sound(&self) -> Option<&WzSound>;
    fn try_as_string(&self) -> Option<&WzString>;
    fn try_as_int(&self) -> Option<&i32>;
    fn try_as_float(&self) -> Option<&f32>;
    fn try_as_vector2d(&self) -> Option<&Vector2D>;
    fn try_as_uol(&self) -> Option<&WzString>;
    fn try_as_lua(&self) -> Option<&WzLua>;
    fn try_as_raw_data(&self) -> Option<&WzRawData>;
    fn try_as_video(&self) -> Option<&WzVideo>;

    fn is_null(&self) -> bool;
    fn is_sub_property(&self) -> bool;
    fn is_convex(&self) -> bool;
}
```

**範例:**
```rust
use wz_reader::NodeCast;

let node = /* ... */;
let read = node.read().unwrap();

if let Some(png) = read.try_as_png() {
    println!("圖片: {}x{}", png.width, png.height);
} else if let Some(&value) = read.try_as_int() {
    println!("整數: {}", value);
}
```

---

## 工具函式

### 樹狀結構遍歷

#### `walk_node()`
```rust
pub fn walk_node<F>(node: &WzNodeArc, parse: bool, cb: &F)
where
    F: Fn(&WzNodeArc)
```

遞迴遍歷節點樹。

**參數:**
- `node`: 起始的根節點
- `parse`: 遍歷期間是否解析節點
- `cb`: 對每個節點呼叫的回呼函式

**範例:**
```rust
use wz_reader::util::walk_node;

walk_node(&root, true, &|node| {
    let read = node.read().unwrap();
    println!("{}", read.get_full_path());
});
```

#### `walk_node_parallel()`（需要 `rayon` 功能）
```rust
#[cfg(feature = "rayon")]
pub fn walk_node_parallel<F>(node: &WzNodeArc, cb: &F)
where
    F: Fn(&WzNodeArc) + Sync
```

平行樹狀結構遍歷以提升效能。

---

### 路徑解析

#### `resolve_base()`
```rust
pub fn resolve_base<P>(
    base_path: P,
    link_path: Option<P>,
) -> Result<WzNodeArc, Error>
where
    P: AsRef<Path>
```

解析 Base.wz，並可選擇性地連結 Data.wz。

**範例:**
```rust
use wz_reader::util::resolve_base;

let base = resolve_base("Data/Base.wz", Some("Data/Data.wz"))?;
```

---

### 節點工具

#### `resolve_childs_parent()`
```rust
pub fn resolve_childs_parent(node: &WzNodeArc)
```

在反序列化後解析父節點參考。

#### `resolve_uol()`
```rust
pub fn resolve_uol(node: &WzNodeArc, parent: Option<&WzNode>)
```

解析 UOL（路徑參考）節點。

---

## 錯誤型別

### Node::Error

```rust
pub enum Error {
    NodeHasBeenUsing,
    WzDirectoryParseError(directory::Error),
    WzFileParseError(file::Error),
    WzImageParseError(wz_image::Error),
    MsFileParseError(ms::file::Error),
    NodeNotFound,
}
```

### file::Error

```rust
pub enum Error {
    FileError(std::io::Error),
    InvalidWzFile,
    ErrorGameVerHash,
    UnknownImageHeader(u8, String),
    UnableToGuessVersion,
    ReaderError(reader::Error),
    DirectoryError(directory::Error),
}
```

### reader::Error

```rust
pub enum Error {
    DecryptError(usize),
    ReadError(scroll::Error),
    ReadUtf8Error(std::string::FromUtf8Error),
    ReadUtf16Error(std::string::FromUtf16Error),
}
```

---

## 完整使用範例

```rust
use wz_reader::{WzNode, WzNodeArc, NodeCast};
use wz_reader::util::{resolve_base, walk_node};
use wz_reader::property::get_image;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 載入 Base.wz 並自動偵測版本
    let base: WzNodeArc = WzNode::from_wz_file("Base.wz", None)?
        .into_lock();

    // 導航到特定節點
    let character = base.read().unwrap()
        .at_path_parsed("Character/00002000.img/stand/0")?;

    // 型別安全存取
    let read = character.read().unwrap();

    if let Some(png) = read.try_as_png() {
        println!("圖片尺寸: {}x{}", png.width, png.height);

        // 提取圖片
        let image = get_image(&character)?;
        image.save("character.png")?;
    } else if let Some(&value) = read.try_as_int() {
        println!("值: {}", value);
    }

    // 遍歷整個樹狀結構
    walk_node(&base, true, &|node| {
        let n = node.read().unwrap();
        if n.try_as_sound().is_some() {
            println!("找到音效: {}", n.get_full_path());
        }
    });

    // JSON 匯出
    #[cfg(feature = "json")]
    {
        let json = read.to_simple_json()?;
        println!("{}", serde_json::to_string_pretty(&json)?);
    }

    Ok(())
}
```

---

## 版本偵測 API

```rust
use wz_reader::version::{
    WzMapleVersion,
    get_iv_by_maple_version,
    guess_iv_from_wz_file,
};

// 取得已知版本的 IV
let iv = get_iv_by_maple_version(WzMapleVersion::GMS);

// 從檔案自動偵測
let file = std::fs::File::open("Base.wz")?;
let map = unsafe { memmap2::Mmap::map(&file)? };
let detected_iv = guess_iv_from_wz_file(&map);
```

---

*若需了解架構細節，請參閱 [ARCHITECTURE.md](./ARCHITECTURE.md)*
*若需了解元件細節，請參閱 [COMPONENTS.md](./COMPONENTS.md)*
*若需了解使用範例，請參閱 [EXAMPLES.md](./EXAMPLES.md)*
