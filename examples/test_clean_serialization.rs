//! 测试干净的 JSON/XML 序列化

use musicflow_server::models::response::{SubsonicResponse, ToXml};

#[derive(Debug, Clone, serde::Serialize)]
struct License {
    pub valid: bool,
    pub email: String,
    pub key: String,
}

impl ToXml for License {
    fn to_xml_element(&self) -> String {
        format!(
            "<license valid=\"{}\" email=\"{}\" key=\"{}\"/>",
            self.valid, self.email, self.key
        )
    }
}

fn main() {
    println!("\n========================================");
    println!("   JSON/XML 干净输出验证");
    println!("========================================\n");

    // 测试 1: 空响应 JSON
    println!("【测试 1】Ping - JSON");
    let ping_json: SubsonicResponse<()> = SubsonicResponse::ok(None);
    let json = serde_json::to_string_pretty(&ping_json).unwrap();
    println!("{}", json);
    assert!(!json.contains("\"@"), "JSON 不应包含 @ 前缀");
    println!("✅ JSON 无 @ 污染\n");

    // 测试 2: 空响应 XML
    println!("【测试 2】Ping - XML");
    let ping_xml: SubsonicResponse<()> = SubsonicResponse::ok_xml(None);
    let xml = ping_xml.to_xml();
    println!("{}", xml);
    assert!(xml.contains("status=\"ok\""), "XML 应包含 status 属性");
    println!("✅ XML 格式正确\n");

    // 测试 3: License JSON
    println!("【测试 3】License - JSON");
    let license = License {
        valid: true,
        email: "admin@example.com".to_string(),
        key: "ABC123".to_string(),
    };
    let license_json = SubsonicResponse::ok(Some(license.clone()));
    let json = serde_json::to_string_pretty(&license_json).unwrap();
    println!("{}", json);
    assert!(json.contains("\"valid\""), "应包含 valid 字段");
    assert!(!json.contains("\"@"), "不应包含 @ 前缀");
    println!("✅ JSON 字段名干净\n");

    // 测试 4: License XML
    println!("【测试 4】License - XML");
    let license_xml = SubsonicResponse::ok_xml(Some(license));
    let xml = license_xml.to_xml();
    println!("{}", xml);
    assert!(xml.contains("valid=\"true\""), "应包含 valid 属性");
    assert!(xml.contains("xmlns="), "应包含命名空间");
    println!("✅ XML 格式正确\n");

    println!("========================================");
    println!("   ✅ 所有测试通过!");
    println!("========================================\n");
}
