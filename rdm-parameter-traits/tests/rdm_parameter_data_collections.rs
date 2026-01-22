#![cfg(feature = "std")]

use rdm_parameter_traits::RdmParameterData;

#[test]
fn vec_u16_roundtrip() {
    let v: Vec<u16> = vec![0x0102, 0x0304];
    let mut buf = [0u8; 4];

    let written = v.encode_rdm_parameter_data(&mut buf).unwrap();
    assert_eq!(written, buf.len());

    let decoded = Vec::<u16>::decode_rdm_parameter_data(&buf).unwrap();
    assert_eq!(decoded, v);
}

#[test]
fn btreemap_roundtrip() {
    use std::collections::BTreeMap;

    let mut m = BTreeMap::<u8, u16>::new();
    m.insert(1, 0x0203);
    m.insert(2, 0x0405);

    let mut buf = [0u8; (1 + 2) * 2];
    // size is dynamic based on element sizes; compute and allocate
    let written = m.encode_rdm_parameter_data(&mut buf).unwrap();
    assert_eq!(written, buf.len());

    let decoded = BTreeMap::<u8, u16>::decode_rdm_parameter_data(&buf).unwrap();
    assert_eq!(decoded, m);
}

#[test]
fn hashmap_stable_roundtrip() {
    use std::collections::HashMap;

    let mut m: HashMap<u8, u16> = HashMap::new();
    m.insert(2, 0x0405);
    m.insert(1, 0x0203);

    let mut buf = [0u8; (1 + 2) * 2];
    let written = m.encode_rdm_parameter_data(&mut buf).unwrap();
    assert_eq!(written, buf.len());

    let decoded = HashMap::<u8, u16>::decode_rdm_parameter_data(&buf).unwrap();
    assert_eq!(decoded, m);
}
