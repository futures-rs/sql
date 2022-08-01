use std::collections::BTreeMap;

use rdbc_serde::OrmSerializer;
use serde::{Deserialize, Serialize};

#[test]
fn test_serializer() {
    _ = pretty_env_logger::try_init();

    #[allow(dead_code)]
    #[derive(Serialize, Deserialize, Default)]
    struct TestTable<'a, Id>
    where
        Id: Serialize,
    {
        id: Id,
        data: Data,
        status: bool,
        msg: &'a str,
        count: u64,
        // rename field
        #[serde(rename = "rdata", with = "serde_bytes")]
        raw_data: Vec<u8>,
    }

    #[allow(dead_code)]
    #[derive(Serialize, Deserialize, Default)]
    struct Data {
        id: i32,
    }

    let msg = "hello world";

    let table = TestTable {
        id: 1_i32,
        data: Data { id: 2 },
        status: true,
        msg,
        count: std::u64::MAX,
        raw_data: msg.as_bytes().to_owned(),
    };

    let mut serializer = OrmSerializer::default();

    table.serialize(&mut serializer).unwrap();

    let args = vec![
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("id".to_owned()),
            value: rdbc::Value::I64(1),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("data".to_owned()),
            value: rdbc::Value::String("{id:2}".to_owned()),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("status".to_owned()),
            value: rdbc::Value::I64(1),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("msg".to_owned()),
            value: rdbc::Value::String(msg.to_owned()),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("count".to_owned()),
            value: rdbc::Value::I64(std::u64::MAX as i64),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("rdata".to_owned()),
            value: rdbc::Value::Bytes(msg.as_bytes().to_owned()),
        },
    ];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_bytes() {
    _ = pretty_env_logger::try_init();

    #[derive(Default, Serialize)]
    struct Test {
        data: Vec<u8>, // serialize as json array [xxx, ...]
    }

    let mut serializer = OrmSerializer::default();

    Test {
        data: "Hello world".as_bytes().to_owned(),
    }
    .serialize(&mut serializer)
    .unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Name("data".to_owned()),
        value: rdbc::Value::String(format!(
            "[{}]",
            "Hello world"
                .as_bytes()
                .iter()
                .map(|c| format!("{}", c))
                .collect::<Vec<_>>()
                .join(","),
        )),
    }];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_tuple_struct() {
    _ = pretty_env_logger::try_init();

    #[derive(Default, Serialize)]
    struct RGB(u8, u8, u8);

    let mut serializer = OrmSerializer::default();

    RGB(1, 2, 4).serialize(&mut serializer).unwrap();

    let args = vec![
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(1),
            value: rdbc::Value::I64(1),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(2),
            value: rdbc::Value::I64(2),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(3),
            value: rdbc::Value::I64(4),
        },
    ];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_newtype_struct() {
    _ = pretty_env_logger::try_init();

    #[derive(Default, Serialize)]
    struct Level(u8);

    let mut serializer = OrmSerializer::default();

    Level(10).serialize(&mut serializer).unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Index(1),
        value: rdbc::Value::I64(10),
    }];

    assert_eq!(args, serializer.args);

    #[derive(Default, Serialize)]
    struct Data {
        level: Level,
    }

    let mut serializer = OrmSerializer::default();

    Data::default().serialize(&mut serializer).unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Name("level".to_owned()),
        value: rdbc::Value::I64(0),
    }];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_tuple() {
    _ = pretty_env_logger::try_init();

    let data = ("hello", 1, vec!["data", "data_2"]);

    let mut serializer = OrmSerializer::default();

    data.serialize(&mut serializer).unwrap();

    let args = vec![
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(1),
            value: rdbc::Value::String("hello".to_owned()),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(2),
            value: rdbc::Value::I64(1),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Index(3),
            value: rdbc::Value::String(r#"["data","data_2"]"#.to_owned()),
        },
    ];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_tuple_variant() {
    #[derive(Serialize)]
    enum Color {
        RGB(u8, u8, u8),
        RGBA(u8, u8, u8, u8),
    }

    let mut serializer = OrmSerializer::default();

    Color::RGB(1, 1, 1).serialize(&mut serializer).unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Index(1),
        value: rdbc::Value::String("RGB[1,1,1]".to_owned()),
    }];

    assert_eq!(args, serializer.args);

    #[derive(Serialize)]
    struct Background {
        color: Color,
    }

    let mut serializer = OrmSerializer::default();

    Background {
        color: Color::RGBA(1, 1, 1, 1),
    }
    .serialize(&mut serializer)
    .unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Name("color".to_owned()),
        value: rdbc::Value::String("RGBA[1,1,1,1]".to_owned()),
    }];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_newtype_variant() {
    #[derive(Serialize)]
    enum Color {
        Hex(u32),
    }

    let mut serializer = OrmSerializer::default();

    Color::Hex(0xff00ff).serialize(&mut serializer).unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Index(1),
        value: rdbc::Value::String("Hex[16711935]".to_owned()),
    }];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_unit_variant() {
    #[allow(dead_code)]
    #[derive(Serialize)]
    enum Orient {
        Left,
        Right,
        Top,
        Bottom,
        Center,
    }

    let mut serializer = OrmSerializer::default();

    Orient::Left.serialize(&mut serializer).unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Index(1),
        value: rdbc::Value::String("Left".to_owned()),
    }];

    assert_eq!(args, serializer.args);

    #[derive(Serialize)]
    struct Align {
        hoz: Orient,
        vect: Orient,
    }

    let mut serializer = OrmSerializer::default();

    Align {
        hoz: Orient::Bottom,
        vect: Orient::Center,
    }
    .serialize(&mut serializer)
    .unwrap();

    let args = vec![
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("hoz".to_owned()),
            value: rdbc::Value::String("Bottom".to_owned()),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("vect".to_owned()),
            value: rdbc::Value::String("Center".to_owned()),
        },
    ];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_option() {
    #[derive(Serialize, Default)]
    struct Table {
        id: Option<u32>,
        color: Option<String>,
    }

    let mut serializer = OrmSerializer::default();

    Table {
        color: Some("red".to_owned()),
        ..Default::default()
    }
    .serialize(&mut serializer)
    .unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Name("color".to_owned()),
        value: rdbc::Value::String("red".to_owned()),
    }];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_map() {
    let data = BTreeMap::from([(1, "hello"), (2, "world")]);

    let mut serializer = OrmSerializer::default();

    data.serialize(&mut serializer).unwrap();

    let args = vec![
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("1".to_owned()),
            value: rdbc::Value::String("hello".to_owned()),
        },
        rdbc::Arg {
            pos: rdbc::Placeholder::Name("2".to_owned()),
            value: rdbc::Value::String("world".to_owned()),
        },
    ];

    assert_eq!(args, serializer.args);
}

#[test]
fn test_struct_variant() {
    #[derive(Serialize)]
    enum Color {
        RGB { r: u8, g: u8, b: u8 },
        RGBA { r: u8, g: u8, b: u8, a: u8 },
    }

    let mut serializer = OrmSerializer::default();

    Color::RGB { r: 1, g: 1, b: 1 }
        .serialize(&mut serializer)
        .unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Index(1),
        value: rdbc::Value::String("RGB{r:1,g:1,b:1}".to_owned()),
    }];

    assert_eq!(args, serializer.args);

    #[derive(Serialize)]
    struct Background {
        color: Color,
    }

    let mut serializer = OrmSerializer::default();

    Background {
        color: Color::RGBA {
            r: 1,
            g: 1,
            b: 1,
            a: 1,
        },
    }
    .serialize(&mut serializer)
    .unwrap();

    let args = vec![rdbc::Arg {
        pos: rdbc::Placeholder::Name("color".to_owned()),
        value: rdbc::Value::String("RGBA{r:1,g:1,b:1,a:1}".to_owned()),
    }];

    assert_eq!(args, serializer.args);
}
