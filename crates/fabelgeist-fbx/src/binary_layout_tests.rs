use super::parse;

// These helpers encode serialized fixture fields, before reader admission.
#[derive(Clone, Copy)]
enum WireHeader {
    Narrow,
    Wide,
}

impl WireHeader {
    fn record_bytes(self) -> usize {
        match self {
            Self::Narrow => 13,
            Self::Wide => 25,
        }
    }

    fn word(self, bytes: &mut Vec<u8>, value: u64) {
        match self {
            Self::Narrow => bytes.extend((value as u32).to_le_bytes()),
            Self::Wide => bytes.extend(value.to_le_bytes()),
        }
    }

    fn header(self, end: u64, count: u64, properties: u64, name: u8) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.word(&mut bytes, end);
        self.word(&mut bytes, count);
        self.word(&mut bytes, properties);
        bytes.push(name);
        bytes
    }

    fn node(
        self,
        start: usize,
        name: &[u8],
        properties: &[u8],
        count: u64,
        children: &[u8],
    ) -> Vec<u8> {
        let end = start + self.record_bytes() * 2 + name.len() + properties.len() + children.len();
        let mut bytes = self.header(end as u64, count, properties.len() as u64, name.len() as u8);
        bytes.extend(name);
        bytes.extend(properties);
        bytes.extend(children);
        bytes.resize(bytes.len() + self.record_bytes(), 0);
        bytes
    }
}

fn file(version: u32, records: &[u8]) -> Vec<u8> {
    let mut bytes = b"Kaydara FBX Binary  \0\x1a\0".to_vec();
    assert_eq!(bytes.len(), 23);
    bytes.extend(version.to_le_bytes());
    bytes.extend(records);
    bytes
}

fn snapshot(bytes: &[u8]) -> String {
    match parse(bytes) {
        Ok(nodes) => format!("nodes:{nodes:?}"),
        Err(error) => format!("error:{error:#}"),
    }
}

fn observe() -> String {
    // Version and record encoding are supplied independently as wire fixtures.
    let cases = [
        (0, WireHeader::Narrow),
        (7100, WireHeader::Narrow),
        (7400, WireHeader::Narrow),
        (7499, WireHeader::Narrow),
        (7500, WireHeader::Wide),
        (7501, WireHeader::Wide),
        (7700, WireHeader::Wide),
        (u32::MAX, WireHeader::Wide),
    ];
    let mut out = String::new();
    let mut integer = vec![b'I'];
    integer.extend((-17i32).to_le_bytes());
    let mut long = vec![b'L'];
    long.extend(i64::MIN.to_le_bytes());
    for (version, format) in cases {
        let leaf = format.node(27, b"Root", &integer, 1, &[]);
        let child = format.node(
            27 + format.record_bytes() + 4 + long.len(),
            b"Child",
            &integer,
            1,
            &[],
        );
        let nested = format.node(27, b"Root", &long, 1, &child);
        let mut siblings = leaf.clone();
        siblings.extend(format.node(27 + leaf.len(), b"Other", &long, 1, &[]));
        let truncated_name = format.header((27 + format.record_bytes() + 9) as u64, 0, 0, 9);
        let mut truncated_property =
            format.header((27 + format.record_bytes() + 4 + 5) as u64, 1, 5, 4);
        truncated_property.extend(b"RootI");
        let records = [
            ("empty", Vec::new()),
            ("terminated", vec![0; format.record_bytes()]),
            ("leaf", leaf),
            ("nested", nested),
            ("siblings", siblings),
            ("truncated-name", truncated_name),
            ("truncated-property", truncated_property),
        ];
        for (label, records) in records {
            out.push_str(&format!(
                "{version}-{label}:{}\n",
                snapshot(&file(version, &records))
            ));
        }
    }
    for (label, bytes) in [
        ("ascii", b"; FBX".as_slice()),
        ("empty-input", b"".as_slice()),
        ("bad-magic", b"an unrelated binary container".as_slice()),
    ] {
        out.push_str(&format!("{label}:{}\n", snapshot(bytes)));
    }
    out
}

#[test]
fn binary_headers_preserve_actual_main_version_and_layout_behavior() {
    assert_eq!(observe(), include_str!("fixtures/binary_layout.txt"));
}
