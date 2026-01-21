use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use wz_reader::property::{get_image, string};
use wz_reader::util::{resolve_base, walk_node};
use wz_reader::{WzNodeArc, WzNodeCast};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Equipment {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_path: Option<String>,
}

// usage:
//   cargo run --example extract_equipment --features "json,image/png" -- "path/to/Base.wz" "./output/equipment"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let base_path = args.get(1).expect("missing base path");
    let default_output = "./output/equipment".to_string();
    let output_dir = args.get(2).unwrap_or(&default_output);

    println!("載入 Base.wz...");
    let base_node = resolve_base(&base_path, None).unwrap();

    // 建立輸出目錄
    fs::create_dir_all(&output_dir).expect("failed to create output directory");

    println!("解析 String/Eqp.img...");

    // 步驟 1: 提取所有裝備 ID 和名稱
    let equipment = extract_equipment(&base_node);

    println!("找到 {} 個裝備", equipment.len());

    // 步驟 2: 尋找並導出圖片
    println!("搜尋對應的圖片...");
    let equipment = find_and_export_images(&base_node, equipment, &output_dir);

    // 步驟 3: 生成 JSON
    let items: Vec<Equipment> = equipment.into_values().collect();
    let found_images = items.iter().filter(|i| i.image_path.is_some()).count();

    println!("找到 {}/{} 個圖片", found_images, items.len());

    let json = serde_json::to_string_pretty(&items).unwrap();
    let json_path = format!("{}/equipment.json", output_dir);
    fs::write(&json_path, json).expect("failed to write json");

    println!("\n✓ 完成！");
    println!("  - JSON: {}", json_path);
    println!("  - 圖片: {}/", output_dir);
}

fn extract_equipment(base_node: &WzNodeArc) -> HashMap<String, Equipment> {
    let equipment = Mutex::new(HashMap::new());

    let eqp_string_node = base_node
        .read()
        .unwrap()
        .at_path("String/Eqp.img")
        .expect("String/Eqp.img not found");

    // 解析節點
    {
        let mut node_write = eqp_string_node.write().unwrap();
        node_write.parse(&eqp_string_node).ok();
    }

    // 提取所有裝備
    walk_node(&eqp_string_node, true, &|node| {
        let node_read = node.read().unwrap();

        // 裝備項目通常是 SubProperty
        if node_read.try_as_sub_property().is_some() {
            let id = node_read.name.clone();

            // 尋找 name 屬性
            if let Some(name_node) = node_read.at("name") {
                if let Ok(name) = string::resolve_string_from_node(&name_node) {
                    let id_string = id.to_string();
                    let mut items = equipment.lock().unwrap();
                    items.insert(
                        id_string.clone(),
                        Equipment {
                            id: id_string,
                            name,
                            image_path: None,
                        },
                    );
                }
            }
        }
    });

    equipment.into_inner().unwrap()
}

fn find_and_export_images(
    base_node: &WzNodeArc,
    equipment: HashMap<String, Equipment>,
    output_dir: &str,
) -> HashMap<String, Equipment> {
    let equipment_mutex = Mutex::new(equipment);

    // 搜尋 Character 目錄下所有子目錄
    if let Some(character_node) = base_node.read().unwrap().at_path("Character") {
        println!("  搜尋 Character/...");

        // 遍歷 Character 下的所有子目錄
        {
            let mut character_write = character_node.write().unwrap();
            character_write.parse(&character_node).ok();
        }

        walk_node(&character_node, true, &|node| {
            let node_read = node.read().unwrap();

            // 找到 PNG 圖片節點
            if node_read.try_as_png().is_some() {
                let full_path = node_read.get_full_path();

                let mut items = equipment_mutex.lock().unwrap();

                // 檢查是否匹配任何裝備 ID
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

    equipment_mutex.into_inner().unwrap()
}
