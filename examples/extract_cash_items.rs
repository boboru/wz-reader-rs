use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use wz_reader::property::{get_image, string};
use wz_reader::util::{resolve_base, walk_node};
use wz_reader::{WzNodeArc, WzNodeCast};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CashItem {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_path: Option<String>,
}

// usage:
//   cargo run --example extract_cash_items --features "json,image/png" -- "path/to/Base.wz" "./output/cash_items"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_path = args.get(1).expect("missing base path");
    let output_dir = args.get(2).unwrap_or(&"./output/cash_items".to_string());

    println!("載入 Base.wz...");
    let base_node = resolve_base(&base_path, None).unwrap();

    // 建立輸出目錄
    fs::create_dir_all(&output_dir).expect("failed to create output directory");

    println!("解析 String/Cash.img...");

    // 步驟 1: 提取現金道具 ID 和名稱
    let cash_items = extract_cash_items(&base_node);

    println!("找到 {} 個現金道具", cash_items.len());

    // 步驟 2: 尋找並導出圖片
    println!("搜尋對應的圖片...");
    let cash_items = find_and_export_images(&base_node, cash_items, &output_dir);

    // 步驟 3: 生成 JSON
    let items: Vec<CashItem> = cash_items.into_values().collect();
    let found_images = items.iter().filter(|i| i.image_path.is_some()).count();

    println!("找到 {}/{} 個圖片", found_images, items.len());

    let json = serde_json::to_string_pretty(&items).unwrap();
    let json_path = format!("{}/cash_items.json", output_dir);
    fs::write(&json_path, json).expect("failed to write json");

    println!("\n✓ 完成！");
    println!("  - JSON: {}", json_path);
    println!("  - 圖片: {}/", output_dir);
}

fn extract_cash_items(base_node: &WzNodeArc) -> HashMap<String, CashItem> {
    let cash_items = Mutex::new(HashMap::new());

    let cash_string_node = base_node
        .read()
        .unwrap()
        .at_path("String/Cash.img")
        .expect("String/Cash.img not found");

    // 解析節點
    {
        let mut node_write = cash_string_node.write().unwrap();
        node_write.parse(&cash_string_node).ok();
    }

    // 提取所有道具
    walk_node(&cash_string_node, true, &|node| {
        let node_read = node.read().unwrap();

        if node_read.try_as_sub_property().is_some() {
            let id = node_read.name.clone();

            if let Some(name_node) = node_read.at("name") {
                if let Ok(name) = string::resolve_string_from_node(&name_node) {
                    let mut items = cash_items.lock().unwrap();
                    items.insert(
                        id.clone(),
                        CashItem {
                            id,
                            name,
                            image_path: None,
                        },
                    );
                }
            }
        }
    });

    cash_items.into_inner().unwrap()
}

fn find_and_export_images(
    base_node: &WzNodeArc,
    mut cash_items: HashMap<String, CashItem>,
    output_dir: &str,
) -> HashMap<String, CashItem> {
    let cash_items_mutex = Mutex::new(cash_items);

    // 可能的圖片位置
    let search_paths = [
        "Character/Cash",
        "Item/Cash",
        "Character",
        "Item/Etc",
    ];

    for path in &search_paths {
        if let Some(dir_node) = base_node.read().unwrap().at_path(path) {
            println!("  搜尋 {}...", path);

            walk_node(&dir_node, true, &|node| {
                let node_read = node.read().unwrap();

                if node_read.try_as_png().is_some() {
                    let full_path = node_read.get_full_path();

                    let mut items = cash_items_mutex.lock().unwrap();

                    // 檢查是否匹配任何現金道具 ID
                    for (id, item) in items.iter_mut() {
                        if item.image_path.is_none() && full_path.contains(id) {
                            if let Ok(image) = get_image(&node) {
                                let filename = format!("{}.png", id);
                                let save_path = format!("{}/{}", output_dir, filename);

                                if image.save(&save_path).is_ok() {
                                    item.image_path = Some(filename);
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    cash_items_mutex.into_inner().unwrap()
}
