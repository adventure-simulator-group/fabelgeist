use derive_more::Display;

use super::binary_layout::{BINARY_SIGNATURE, FILE_HEADER_BYTES, FbxVersion};
use super::parse;

const ROOT_NAME: &[u8] = b"Root";
const CHILD_NAME: &[u8] = b"Child";
const SIBLING_NAME: &[u8] = b"Other";
const TRUNCATED_NAME: &[u8] = b"Truncated";
// Independent wire expectations: three 32/64-bit words and one name-length
// byte.
// Blender's <IIIB / <QQQB layouts are cited in binary_layout.rs.
const NARROW_WIRE_HEADER_BYTES: usize = 13;
const WIDE_WIRE_HEADER_BYTES: usize = 25;

struct VersionFixture {
    version: FbxVersion,
    header: WireHeader,
}

// Native serialized fields intentionally precede the public parser's admission.
struct WireNodeHeader {
    end_offset: u64,
    property_count: u64,
    property_bytes: u64,
    name_bytes: u8,
}

struct WireNode<'a> {
    start_offset: usize,
    name: &'a [u8],
    properties: &'a [u8],
    property_count: u64,
    children: &'a [u8],
}

struct RecordFixture {
    scenario: RecordScenario,
    bytes: Vec<u8>,
}

#[derive(Clone, Copy)]
enum WireHeader {
    Narrow,
    Wide,
}

#[derive(Clone, Copy)]
enum VersionScenario {
    OpenZero,
    EarliestDocumented,
    TypicalNarrow,
    BeforeWideCutoff,
    AtWideCutoff,
    AfterWideCutoff,
    LatestDocumented,
    OpenMaximum,
}

#[derive(Display)]
enum RecordScenario {
    #[display("empty")]
    Empty,
    #[display("terminated")]
    Terminated,
    #[display("leaf")]
    Leaf,
    #[display("nested")]
    Nested,
    #[display("siblings")]
    Siblings,
    #[display("truncated-name")]
    TruncatedName,
    #[display("truncated-property")]
    TruncatedProperty,
}

#[derive(Display)]
enum RejectedInput {
    #[display("ascii")]
    Ascii,
    #[display("empty-input")]
    Empty,
    #[display("bad-magic")]
    WrongSignature,
}

impl VersionFixture {
    fn from_scenario(scenario: VersionScenario) -> Self {
        // Encoding is independent of the production version/layout
        // classification.
        match scenario {
            VersionScenario::OpenZero => Self {
                version: 0.into(),
                header: WireHeader::Narrow,
            },
            VersionScenario::EarliestDocumented => Self {
                version: 7100.into(),
                header: WireHeader::Narrow,
            },
            VersionScenario::TypicalNarrow => Self {
                version: 7400.into(),
                header: WireHeader::Narrow,
            },
            VersionScenario::BeforeWideCutoff => Self {
                version: 7499.into(),
                header: WireHeader::Narrow,
            },
            VersionScenario::AtWideCutoff => Self {
                version: 7500.into(),
                header: WireHeader::Wide,
            },
            VersionScenario::AfterWideCutoff => Self {
                version: 7501.into(),
                header: WireHeader::Wide,
            },
            VersionScenario::LatestDocumented => Self {
                version: 7700.into(),
                header: WireHeader::Wide,
            },
            VersionScenario::OpenMaximum => Self {
                version: u32::MAX.into(),
                header: WireHeader::Wide,
            },
        }
    }

    fn records(&self) -> [RecordFixture; 7] {
        let mut integer = vec![b'I'];
        integer.extend((-17i32).to_le_bytes());
        let mut long = vec![b'L'];
        long.extend(i64::MIN.to_le_bytes());
        let format = self.header;
        let leaf = WireNode {
            start_offset: FILE_HEADER_BYTES,
            name: ROOT_NAME,
            properties: &integer,
            property_count: 1,
            children: &[],
        }
        .encode(format);
        let child = WireNode {
            start_offset: FILE_HEADER_BYTES + format.record_bytes() + ROOT_NAME.len() + long.len(),
            name: CHILD_NAME,
            properties: &integer,
            property_count: 1,
            children: &[],
        }
        .encode(format);
        let nested = WireNode {
            start_offset: FILE_HEADER_BYTES,
            name: ROOT_NAME,
            properties: &long,
            property_count: 1,
            children: &child,
        }
        .encode(format);
        let mut siblings = leaf.clone();
        siblings.extend(
            WireNode {
                start_offset: FILE_HEADER_BYTES + leaf.len(),
                name: SIBLING_NAME,
                properties: &long,
                property_count: 1,
                children: &[],
            }
            .encode(format),
        );
        let truncated_name = WireNodeHeader {
            end_offset: (FILE_HEADER_BYTES + format.record_bytes() + TRUNCATED_NAME.len()) as u64,
            property_count: 0,
            property_bytes: 0,
            name_bytes: TRUNCATED_NAME.len() as u8,
        }
        .encode(format);
        let mut truncated_property = WireNodeHeader {
            end_offset: (FILE_HEADER_BYTES
                + format.record_bytes()
                + ROOT_NAME.len()
                + integer.len()) as u64,
            property_count: 1,
            property_bytes: integer.len() as u64,
            name_bytes: ROOT_NAME.len() as u8,
        }
        .encode(format);
        truncated_property.extend(ROOT_NAME);
        truncated_property.push(b'I');
        [
            RecordFixture {
                scenario: RecordScenario::Empty,
                bytes: Vec::new(),
            },
            RecordFixture {
                scenario: RecordScenario::Terminated,
                bytes: vec![0; format.record_bytes()],
            },
            RecordFixture {
                scenario: RecordScenario::Leaf,
                bytes: leaf,
            },
            RecordFixture {
                scenario: RecordScenario::Nested,
                bytes: nested,
            },
            RecordFixture {
                scenario: RecordScenario::Siblings,
                bytes: siblings,
            },
            RecordFixture {
                scenario: RecordScenario::TruncatedName,
                bytes: truncated_name,
            },
            RecordFixture {
                scenario: RecordScenario::TruncatedProperty,
                bytes: truncated_property,
            },
        ]
    }

    fn file(&self, records: &[u8]) -> Vec<u8> {
        let mut bytes = BINARY_SIGNATURE.to_vec();
        let wire_version: u32 = self.version.into();
        bytes.extend(wire_version.to_le_bytes());
        bytes.extend(records);
        bytes
    }
}

impl WireHeader {
    fn record_bytes(self) -> usize {
        match self {
            Self::Narrow => NARROW_WIRE_HEADER_BYTES,
            Self::Wide => WIDE_WIRE_HEADER_BYTES,
        }
    }

    // Native integer-to-wire projection, selected by the independent fixture
    // format.
    fn word(self, bytes: &mut Vec<u8>, value: u64) {
        match self {
            Self::Narrow => bytes.extend((value as u32).to_le_bytes()),
            Self::Wide => bytes.extend(value.to_le_bytes()),
        }
    }
}

impl WireNodeHeader {
    fn encode(self, format: WireHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        format.word(&mut bytes, self.end_offset);
        format.word(&mut bytes, self.property_count);
        format.word(&mut bytes, self.property_bytes);
        bytes.push(self.name_bytes);
        bytes
    }
}

impl WireNode<'_> {
    fn encode(self, format: WireHeader) -> Vec<u8> {
        let end = self.start_offset
            + format.record_bytes() * 2
            + self.name.len()
            + self.properties.len()
            + self.children.len();
        let mut bytes = WireNodeHeader {
            end_offset: end as u64,
            property_count: self.property_count,
            property_bytes: self.properties.len() as u64,
            name_bytes: self.name.len() as u8,
        }
        .encode(format);
        bytes.extend(self.name);
        bytes.extend(self.properties);
        bytes.extend(self.children);
        bytes.resize(bytes.len() + format.record_bytes(), 0);
        bytes
    }
}

impl RejectedInput {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Ascii => b"; FBX",
            Self::Empty => b"",
            Self::WrongSignature => b"an unrelated binary container",
        }
    }
}

fn snapshot(bytes: &[u8]) -> String {
    match parse(bytes) {
        Ok(nodes) => format!("nodes:{nodes:?}"),
        Err(error) => format!("error:{error:#}"),
    }
}

fn observe() -> String {
    let scenarios = [
        VersionScenario::OpenZero,
        VersionScenario::EarliestDocumented,
        VersionScenario::TypicalNarrow,
        VersionScenario::BeforeWideCutoff,
        VersionScenario::AtWideCutoff,
        VersionScenario::AfterWideCutoff,
        VersionScenario::LatestDocumented,
        VersionScenario::OpenMaximum,
    ];
    let mut out = String::new();
    for scenario in scenarios {
        let fixture = VersionFixture::from_scenario(scenario);
        let wire_version: u32 = fixture.version.into();
        for RecordFixture { scenario, bytes } in fixture.records() {
            out.push_str(&format!(
                "{wire_version}-{scenario}:{}\n",
                snapshot(&fixture.file(&bytes))
            ));
        }
    }
    for scenario in [
        RejectedInput::Ascii,
        RejectedInput::Empty,
        RejectedInput::WrongSignature,
    ] {
        out.push_str(&format!("{scenario}:{}\n", snapshot(scenario.bytes())));
    }
    out
}

#[test]
fn binary_headers_preserve_actual_main_version_and_layout_behavior() {
    assert_eq!(observe(), include_str!("fixtures/binary_layout.txt"));
}
