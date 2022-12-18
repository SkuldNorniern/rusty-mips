use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDetail {
    pub debug_ins_if: Option<String>,
    pub debug_ins_id: Option<String>,
    pub debug_ins_ex: Option<String>,
    pub debug_ins_mem: Option<String>,
    pub debug_ins_wb: Option<String>,
    pub nodes: Vec<PipelineNodeInfo>
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum PipelineNodeInfo {
    Hex32{ name: String, value: u32},
    Dec{name: String, value: i64},
    Bool{name: String, value: bool},
}

#[cfg(test)]
mod test {
    use serde_json::json;
    use super::*;

    #[test]
    fn empty() {
        let pipeline = PipelineDetail {
            debug_ins_if: Some("a".into()),
            debug_ins_id: Some("b".into()),
            debug_ins_ex: None,
            debug_ins_mem: None,
            debug_ins_wb: Some("e".into()),
            nodes: vec![],
        };

        let serialized = serde_json::to_value(pipeline).unwrap();

        assert_eq!(serialized, json!({
            "debugInsIf": "a",
            "debugInsId": "b",
            "debugInsEx": null,
            "debugInsMem": null,
            "debugInsWb": "e",
            "nodes": []
        }));
    }

    #[test]
    fn basic() {
        use PipelineNodeInfo::*;

        let pipeline = PipelineDetail {
            debug_ins_if: Some("a".into()),
            debug_ins_id: Some("b".into()),
            debug_ins_ex: Some("c".into()),
            debug_ins_mem: Some("d".into()),
            debug_ins_wb: Some("e".into()),
            nodes: vec![
                Hex32{name: "x".into(), value:0x123456},
                Dec{name: "y".into(), value:1234},
                Dec{name: "z".into(), value: -12},
                Bool{name: "w".into(), value: false},
            ],
        };

        let serialized = serde_json::to_value(&pipeline).unwrap();

        assert_eq!(serialized, json!({
            "debugInsIf": "a",
            "debugInsId": "b",
            "debugInsEx": "c",
            "debugInsMem": "d",
            "debugInsWb": "e",
            "nodes": [
                { "type": "hex32", "name": "x", "value": 1193046 },
                { "type": "dec", "name": "y", "value": 1234 },
                { "type": "dec", "name": "z", "value": -12 },
                { "type": "bool", "name": "w", "value": false },
            ]
        }), "{}", serde_json::to_string_pretty(&pipeline).unwrap());
    }
}
