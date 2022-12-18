use serde::Serialize;

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDetail {
    pub debug_ins_if: Option<String>,
    pub debug_ins_id: Option<String>,
    pub debug_ins_ex: Option<String>,
    pub debug_ins_mem: Option<String>,
    pub debug_ins_wb: Option<String>,
    pub nodes: Vec<PipelineNodeInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum PipelineNodeInfo {
    Hex32 {
        id: &'static str,
        name: &'static str,
        value: u32,
    },
    Dec {
        id: &'static str,
        name: &'static str,
        value: i64,
    },
    Bool {
        id: &'static str,
        name: &'static str,
        value: bool,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

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

        assert_eq!(
            serialized,
            json!({
                "debugInsIf": "a",
                "debugInsId": "b",
                "debugInsEx": null,
                "debugInsMem": null,
                "debugInsWb": "e",
                "nodes": []
            })
        );
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
                Hex32 {
                    id: "id_x",
                    name: "x".into(),
                    value: 0x123456,
                },
                Dec {
                    id: "id_y",
                    name: "y".into(),
                    value: 1234,
                },
                Dec {
                    id: "id_z",
                    name: "z".into(),
                    value: -12,
                },
                Bool {
                    id: "id_w",
                    name: "w".into(),
                    value: false,
                },
            ],
        };

        let serialized = serde_json::to_value(&pipeline).unwrap();

        assert_eq!(
            serialized,
            json!({
                "debugInsIf": "a",
                "debugInsId": "b",
                "debugInsEx": "c",
                "debugInsMem": "d",
                "debugInsWb": "e",
                "nodes": [
                    { "type": "hex32", "id": "id_x", "name": "x", "value": 1193046 },
                    { "type": "dec", "id": "id_y", "name": "y", "value": 1234 },
                    { "type": "dec", "id": "id_z", "name": "z", "value": -12 },
                    { "type": "bool", "id": "id_w", "name": "w", "value": false },
                ]
            }),
            "{}",
            serde_json::to_string_pretty(&pipeline).unwrap()
        );
    }
}
