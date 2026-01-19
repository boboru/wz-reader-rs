# 使用範例

> **最後更新**: 2026 年 1 月 18 日
> **版本**: 0.0.16

## 目錄

- [快速開始](#快速開始)
- [載入檔案](#載入檔案)
- [節點導航](#節點導航)
- [提取內容](#提取內容)
- [處理屬性](#處理屬性)
- [進階用法](#進階用法)
- [效能最佳化](#效能最佳化)
- [常見模式](#常見模式)
- [實際應用範例](#實際應用範例)

---

## 快速開始

### 最小範例

```rust
use wz_reader::{WzNode, NodeCast};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 載入 .wz 檔案
    let node = WzNode::from_wz_file("Base.wz", None)?.into_lock();

    // 解析檔案
    node.write().unwrap().parse(&node)?;

    // 印出所有頂層項目
    for (name, _child) in &node.read().unwrap().children {
        println!("{}", name);
    }

    Ok(())
}
```

### 基礎樹狀結構遍歷

```rust
use wz_reader::util::walk_node;

walk_node(&node, true, &|node| {
    let read = node.read().unwrap();
    println!("{}", read.get_full_path());
});
```

---

## 載入檔案

### 1. 單一 .wz 檔案（自動偵測）

**使用情境**: 載入任何 .wz 檔案而無需知道版本

```rust
use wz_reader::{WzNode, WzNodeArc};

let node: WzNodeArc = WzNode::from_wz_file("UI.wz", None)?.into_lock();
```

**執行過程:**
- 開啟並記憶體映射檔案
- 自動偵測加密 IV
- 自動偵測補丁版本（嘗試 1-2000）
- 回傳未解析節點（延遲載入）

### 2. 使用已知版本

**使用情境**: 當您知道版本時可更快載入

```rust
use wz_reader::version::WzMapleVersion;

let node = WzNode::from_wz_file_full(
    "Base.wz",
    Some(WzMapleVersion::GMS),
    Some(83),  // 補丁版本
    None,
    None
)?;
```

**優點:**
- 跳過版本偵測（更快）
- 保證版本正確
- 無需試錯

### 3. 使用共享金鑰載入多個檔案

**使用情境**: 從相同版本載入多個檔案

```rust
// 載入第一個檔案
let base = WzNode::from_wz_file("Base.wz", None)?;
let base_arc = base.into_lock();

// 共享加密金鑰以加快載入速度
let base_keys = &base_arc.read().unwrap()
    .try_as_file().unwrap()
    .reader.keys;

let ui = WzNode::from_wz_file_full(
    "UI.wz",
    None,
    None,
    None,
    Some(base_keys)  // 重用金鑰
)?;
```

**優點:**
- 節省記憶體（共享金鑰資料）
- 更快的初始化
- 一致的加密

### 4. Base.wz 與相依性

**使用情境**: 解析 Base.wz 與 Data.wz 連結

```rust
use wz_reader::util::resolve_base;

let base = resolve_base(
    "Data/Base.wz",
    Some("Data/Data.wz")
)?;
```

**功能:**
- 載入 Base.wz
- 載入 Data.wz（如果提供）
- 自動合併樹狀結構
- 解析交叉引用

### 5. 整個資料夾

**使用情境**: 載入目錄中所有 .wz 檔案

```rust
use wz_reader::util::resolve_root_wz_file_dir;

let root = resolve_root_wz_file_dir("C:/MapleStory/Data", None)?;
```

### 6. MapleStoryM .ms 檔案

```rust
let ms_node = WzNode::from_ms_file("Data.ms", None)?;
```

**自動偵測:**
- Snow2 加密
- ChaCha20 加密

### 7. 獨立 .img 檔案

```rust
let img = WzNode::from_img_file("00002000.img", None, None)?;
```

---

## 節點導航

### 1. 直接存取子節點

```rust
let read = node.read().unwrap();

if let Some(character) = read.at("Character") {
    println!("找到 Character 目錄");
}
```

### 2. 路徑導航（未解析）

```rust
// 僅在節點已解析時有效
let target = node.read().unwrap()
    .at_path("Character/00002000.img");
```

### 3. 路徑導航（自動解析）

```rust
// 解析路徑上的所有節點
let target = node.read().unwrap()
    .at_path_parsed("Character/00002000.img/stand/0")?;
```

**最適合**: 無需解析整個樹狀結構即可存取深層節點

### 4. 相對導航

```rust
let current = /* 某個節點 */;
let read = current.read().unwrap();

// 前往父節點
let parent = read.at_relative("..")?;

// 前往兄弟節點
let sibling = read.at_path_relative("../other_item")?;
```

### 5. 父節點搜尋

```rust
// 尋找最近的 WzImage 父節點
let image = node.read().unwrap().get_parent_wz_image()?;

// 尋找特定父節點
let base = node.read().unwrap()
    .filter_parent(|n| n.name.as_str() == "Base")?;
```

### 6. 取得路徑

```rust
let read = node.read().unwrap();

// 從根節點的完整路徑
println!("{}", read.get_full_path());
// 輸出: "Base/Character/00002000.img/stand/0"

// 從根節點的路徑（排除根節點名稱）
println!("{}", read.get_path_from_root());
// 輸出: "Character/00002000.img/stand/0"

// 從最近的 image 的路徑
println!("{}", read.get_path_from_image());
// 輸出: "stand/0"
```

---

## 提取內容

### 1. 提取 PNG 圖片

```rust
use wz_reader::property::get_image;
use wz_reader::NodeCast;

fn extract_image(node: &WzNodeArc) -> Result<(), Box<dyn std::error::Error>> {
    let read = node.read().unwrap();

    if read.try_as_png().is_some() {
        let image = get_image(node)?;
        let path = format!("output/{}.png", read.name);
        image.save(path)?;
        println!("已儲存: {}", read.name);
    }

    Ok(())
}
```

**批次提取:**

```rust
use wz_reader::util::walk_node;

walk_node(&root, true, &|node| {
    if let Err(e) = extract_image(node) {
        eprintln!("失敗: {}", e);
    }
});
```

### 2. 提取音效檔案

```rust
fn extract_sound(node: &WzNodeArc) -> Result<(), Box<dyn std::error::Error>> {
    let read = node.read().unwrap();

    if let Some(sound) = read.try_as_sound() {
        let path = format!("output/{}.mp3", read.name);
        sound.save(path)?;
        println!("已儲存: {}", read.name);
    }

    Ok(())
}
```

**或取得位元組:**

```rust
if let Some(sound) = read.try_as_sound() {
    let audio_data = sound.get_bytes()?;
    // 處理 audio_data
}
```

### 3. 提取 Lua 腳本

```rust
fn extract_lua(node: &WzNodeArc) -> Result<(), Box<dyn std::error::Error>> {
    let read = node.read().unwrap();

    if let Some(lua) = read.try_as_lua() {
        let script = lua.get_string()?;
        let path = format!("output/{}.lua", read.name);
        std::fs::write(path, script)?;
    }

    Ok(())
}
```

### 4. 提取原始資料

```rust
if let Some(raw) = read.try_as_raw_data() {
    let data = raw.get_bytes()?;
    std::fs::write("output.dat", data)?;
}
```

---

## 處理屬性

### 1. 型別檢查與轉換

```rust
use wz_reader::NodeCast;

let read = node.read().unwrap();

// 檢查型別
if read.is_null() {
    println!("空節點");
}

// 安全轉換
if let Some(&value) = read.try_as_int() {
    println!("整數: {}", value);
} else if let Some(&x) = read.try_as_float() {
    println!("浮點數: {}", x);
} else if let Some(vec) = read.try_as_vector2d() {
    println!("向量: ({}, {})", vec.x(), vec.y());
}
```

### 2. 字串屬性

```rust
// 加密字串
if let Some(wz_str) = read.try_as_string() {
    let text = wz_str.get_string()?;
    println!("字串: {}", text);
}

// 已解析字串
match &read.object_type {
    WzObjectType::Value(WzValue::ParsedString(s)) => {
        println!("字串: {}", s);
    }
    _ => {}
}
```

### 3. Vector2D（座標）

```rust
if let Some(vec) = read.try_as_vector2d() {
    println!("位置: ({}, {})", vec.x(), vec.y());

    // 建立新向量
    let origin = Vector2D::new(0, 0);
}
```

### 4. UOL（路徑引用）

```rust
if let Some(uol) = read.try_as_uol() {
    let ref_path = uol.get_string()?;
    println!("引用: {}", ref_path);

    // 解析引用
    let target = read.at_path_relative(&ref_path)?;
}
```

### 5. PNG 屬性

```rust
if let Some(png) = read.try_as_png() {
    println!("尺寸: {}x{}", png.width, png.height);

    // 提取圖片
    let image = get_image(node)?;

    // 檢查是否有子節點（origin、z 等）
    for (name, child) in &read.children {
        println!("  {}: {:?}", name, child.read().unwrap().object_type);
    }
}
```

### 6. 音效屬性

```rust
if let Some(sound) = read.try_as_sound() {
    println!("時長: {}ms", sound.duration);
    println!("類型: {:?}", sound.sound_type);

    match sound.sound_type {
        WzSoundType::Mp3 => println!("MP3 格式"),
        WzSoundType::Binary => println!("二進位格式"),
    }
}
```

---

## 進階用法

### 1. 平行解析

```rust
use std::thread;

let node = WzNode::from_wz_file("Base.wz", None)?.into_lock();
node.write().unwrap().parse(&node)?;

// 平行解析子節點
let handles: Vec<_> = node.read().unwrap()
    .children
    .values()
    .map(|child| {
        let child = Arc::clone(child);
        thread::spawn(move || {
            child.write().unwrap().parse(&child).unwrap();
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

**使用 Rayon:**

```rust
#[cfg(feature = "rayon")]
use rayon::prelude::*;

node.read().unwrap()
    .children
    .par_iter()
    .for_each(|(_, child)| {
        child.write().unwrap().parse(child).ok();
    });
```

### 2. 記憶體管理（取消解析）

```rust
// 解析並處理
{
    let mut write = node.write().unwrap();
    write.parse(&node)?;

    // 處理子節點...

    // 釋放記憶體
    write.unparse();
}
```

**適用於:**
- 分塊處理大型檔案
- 保持低記憶體使用
- 批次處理

### 3. 直接存取 Image（無需完整解析）

```rust
use wz_reader::WzImage;

// 從節點取得 WzImage
let read = node.read().unwrap();
if let Some(image) = read.try_as_image() {
    // 存取特定路徑而無需解析整個 image
    let sprite = image.at_path("stand/0")?;

    // 比解析整個 image 快得多
}
```

### 4. JSON 匯出

```rust
#[cfg(feature = "json")]
{
    // 簡單 JSON（僅值）
    let json = node.read().unwrap().to_simple_json()?;
    println!("{}", serde_json::to_string_pretty(&json)?);

    // 完整 JSON（包含後設資料）
    let full_json = node.read().unwrap().to_json()?;
    std::fs::write("output.json", serde_json::to_string_pretty(&full_json)?)?;
}
```

### 5. 自訂樹狀結構遍歷

```rust
fn walk_with_depth<F>(node: &WzNodeArc, depth: usize, cb: &F)
where
    F: Fn(&WzNodeArc, usize)
{
    cb(node, depth);

    for (_, child) in &node.read().unwrap().children {
        walk_with_depth(child, depth + 1, cb);
    }
}

// 使用方式
walk_with_depth(&root, 0, &|node, depth| {
    let indent = "  ".repeat(depth);
    let read = node.read().unwrap();
    println!("{}{}", indent, read.name);
});
```

### 6. 篩選節點

```rust
fn find_all_pngs(node: &WzNodeArc) -> Vec<WzNodeArc> {
    let mut pngs = Vec::new();

    walk_node(node, true, &|n| {
        if n.read().unwrap().try_as_png().is_some() {
            pngs.push(Arc::clone(n));
        }
    });

    pngs
}
```

### 7. 手動建構節點樹

```rust
use wz_reader::{WzNode, property::Vector2D};

// 建立根節點
let root = WzNode::from_str("root", 0, None).into_lock();

// 建立子節點
let child1 = WzNode::from_str("position", Vector2D::new(100, 200), Some(&root))
    .into_lock();
let child2 = WzNode::from_str("name", "Warrior".to_string(), Some(&root))
    .into_lock();

// 加入到父節點
root.write().unwrap().add(&child1);
root.write().unwrap().add(&child2);

// 匯出為 JSON
#[cfg(feature = "json")]
{
    let json = root.read().unwrap().to_simple_json()?;
    // {"position": {"x": 100, "y": 200}, "name": "Warrior"}
}
```

---

## 效能最佳化

### 1. 延遲解析策略

```rust
// 不要預先解析所有內容
// ❌ 不好:
node.write().unwrap().parse(&node)?;
walk_node(&node, false, &|n| { /* ... */ });

// ✅ 好: 按需解析
let target = node.read().unwrap()
    .at_path_parsed("specific/path/only")?;
```

### 2. 重用加密金鑰

```rust
// 載入多個檔案時
let keys = &first_file.reader.keys;

for path in file_paths {
    let file = WzNode::from_wz_file_full(
        path, None, None, None,
        Some(keys)  // 重用
    )?;
}
```

### 3. 使用後取消解析

```rust
for entry in entries {
    let mut write = entry.write().unwrap();
    write.parse(&entry)?;

    // 處理...

    write.unparse();  // 釋放記憶體
}
```

### 4. 平行處理

```rust
#[cfg(feature = "rayon")]
{
    use rayon::prelude::*;

    images.par_iter().for_each(|img| {
        extract_image(img).ok();
    });
}
```

### 5. 選擇性解析

```rust
// 僅解析所需內容
let read = node.read().unwrap();

for (name, child) in &read.children {
    if name.contains("Character") {
        child.write().unwrap().parse(child)?;
    }
}
```

---

## 常見模式

### 模式 1: 尋找並提取

```rust
fn find_and_extract_item(root: &WzNodeArc, item_id: i32) -> Result<()> {
    let path = format!("Character/Equip/{:08}.img", item_id);
    let item = root.read().unwrap().at_path_parsed(&path)?;

    // 提取圖示
    if let Some(icon_node) = item.read().unwrap().at("info/icon") {
        if icon_node.read().unwrap().try_as_png().is_some() {
            let image = get_image(&icon_node)?;
            image.save(format!("item_{}.png", item_id))?;
        }
    }

    Ok(())
}
```

### 模式 2: 建立索引

```rust
use std::collections::HashMap;

fn build_item_index(root: &WzNodeArc) -> HashMap<i32, String> {
    let mut index = HashMap::new();

    walk_node(root, true, &|node| {
        let read = node.read().unwrap();

        // 尋找物品 ID
        if let Some(&id) = read.try_as_int() {
            if read.name.as_str() == "id" {
                if let Some(name_node) = read.at("../name") {
                    if let Some(name_str) = name_node.read().unwrap().try_as_string() {
                        if let Ok(name) = name_str.get_string() {
                            index.insert(id, name);
                        }
                    }
                }
            }
        }
    });

    index
}
```

### 模式 3: 比較版本

```rust
fn compare_files(old_path: &str, new_path: &str) -> Result<()> {
    let old = WzNode::from_wz_file(old_path, None)?.into_lock();
    let new = WzNode::from_wz_file(new_path, None)?.into_lock();

    // 解析兩者
    old.write().unwrap().parse(&old)?;
    new.write().unwrap().parse(&new)?;

    // 比較
    for (name, old_child) in &old.read().unwrap().children {
        if let Some(new_child) = new.read().unwrap().at(name) {
            // 比較節點...
        } else {
            println!("已移除: {}", name);
        }
    }

    Ok(())
}
```

---

## 實際應用範例

### 範例 1: 提取所有角色精靈圖

參見 `examples/extracting_pngs.rs`:

```rust
use wz_reader::property::get_image;
use wz_reader::util::walk_node;

let base = resolve_base("Base.wz", None)?;

walk_node(&base, true, &|node| {
    let read = node.read().unwrap();

    if read.try_as_png().is_some() {
        let path = read.get_full_path().replace("/", "-");
        if let Ok(image) = get_image(node) {
            image.save(format!("output/{}.png", path)).ok();
        }
    }
});
```

### 範例 2: Web 伺服器（Axum）

參見 `examples/with_axum.rs`:

```rust
use axum::{Router, extract::Path, Json};

#[tokio::main]
async fn main() {
    let base = resolve_base("Base.wz", None).unwrap();

    let app = Router::new()
        .route("/item/:id", get(get_item))
        .with_state(base);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_item(
    State(base): State<WzNodeArc>,
    Path(id): Path<i32>,
) -> Json<Value> {
    let path = format!("Character/Equip/{:08}.img", id);
    let node = base.read().unwrap().at_path_parsed(&path).unwrap();
    let json = node.read().unwrap().to_simple_json().unwrap();
    Json(json)
}
```

### 範例 3: 平行圖片處理

參見 `examples/parallel_parse_wz_image.rs`:

```rust
use std::thread;

let node = WzNode::from_wz_file("Base.wz", None)?.into_lock();
node.write().unwrap().parse(&node)?;

// 平行解析所有 image
let handles: Vec<_> = node.read().unwrap()
    .children
    .values()
    .map(|child| {
        let child = Arc::clone(child);
        thread::spawn(move || {
            child.write().unwrap().parse(&child).unwrap();
            println!("已解析: {}", child.read().unwrap().name);
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

---

## 錯誤處理

### 基礎錯誤處理

```rust
match WzNode::from_wz_file("Base.wz", None) {
    Ok(node) => {
        // 處理節點
    }
    Err(e) => {
        eprintln!("載入失敗: {}", e);
    }
}
```

### 優雅降級

```rust
walk_node(&root, true, &|node| {
    let read = node.read().unwrap();

    if let Some(png) = read.try_as_png() {
        if let Ok(image) = get_image(node) {
            image.save(format!("{}.png", read.name)).ok();
        } else {
            eprintln!("提取失敗: {}", read.get_full_path());
        }
    }
});
```

---

## 提示與最佳實踐

1. **始終使用延遲解析** - 除非必要，否則不要解析整個檔案
2. **處理後取消解析** - 為大型操作釋放記憶體
3. **重用加密金鑰** - 載入多個檔案時
4. **使用平行處理** - 用於批次操作
5. **優雅處理錯誤** - 檔案可能損壞或不完整
6. **快取常用節點** - 儲存 Arc 克隆
7. **使用 `at_path_parsed()` 深度存取** - 比手動導航更有效率
8. **最佳化前先分析** - 使用 `cargo bench` 識別瓶頸

---

*API 參考請見 [API.md](./API.md)*
*設定說明請見 [SETUP.md](./SETUP.md)*
*元件細節請見 [COMPONENTS.md](./COMPONENTS.md)*
