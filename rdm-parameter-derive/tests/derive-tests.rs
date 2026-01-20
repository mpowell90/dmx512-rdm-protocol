use rdm_parameter_derive::RdmGetRequestParameter;
use rdm_parameter_traits::RdmGetRequestParameterCodec;

mod tests {
    use super::*;

    #[test]
    fn basic_struct() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: u8,
            three: u8,
        }

        let my_struct = MyStruct {
            one: 1,
            two: 2,
            three: 3,
        };

        let mut buf = [0u8; 3];

        let res = my_struct.get_request_encode_data(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf, [1, 2, 3]);
    }

    #[test]
    fn struct_with_u16() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: u16,
            three: u8,
        }
        let my_struct = MyStruct {
            one: 1,
            two: 0x0203,
            three: 4,
        };
        let mut buf = [0u8; 4];
        let res = my_struct.get_request_encode_data(&mut buf);
        assert!(res.is_ok());
        assert_eq!(buf, [1, 0x02, 0x03, 4]);
    }

    #[test]
    fn struct_with_fixed_array() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: [u8; 3],
            three: u8,
        }
        let my_struct = MyStruct {
            one: 1,
            two: [2, 3, 4],
            three: 5,
        };
        let mut buf = [0u8; 5];
        let res = my_struct.get_request_encode_data(&mut buf);
        assert!(res.is_ok());
        assert_eq!(buf, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn struct_with_option_u16_some_encodes() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: Option<u16>,
            three: u8,
        }

        let my_struct = MyStruct {
            one: 1,
            two: Some(0x0203),
            three: 4,
        };

        let mut buf = [0u8; 4];
        let res = my_struct.get_request_encode_data(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf, [1, 0x02, 0x03, 4]);
    }

    #[test]
    fn struct_with_option_u16_none_skips() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: Option<u16>,
            three: u8,
        }

        let my_struct = MyStruct {
            one: 1,
            two: None,
            three: 4,
        };

        let mut buf = [0u8; 2];
        let res = my_struct.get_request_encode_data(&mut buf);

        assert!(res.is_ok());
        assert_eq!(buf, [1, 4]);
    }

    #[test]
    fn struct_with_option_u16_decode_none_when_truncated() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: Option<u16>,
            three: u8,
        }

        let bytes = [1u8, 4u8];
        let decoded = MyStruct::get_request_decode_data(&bytes).unwrap();
        assert_eq!(decoded.one, 1);
        assert_eq!(decoded.two, None);
        assert_eq!(decoded.three, 4);
    }

    #[test]
    fn struct_with_option_u16_decode_some_when_present() {
        #[derive(RdmGetRequestParameter)]
        struct MyStruct {
            one: u8,
            two: Option<u16>,
            three: u8,
        }

        let bytes = [1u8, 0x02u8, 0x03u8, 4u8];
        let decoded = MyStruct::get_request_decode_data(&bytes).unwrap();
        assert_eq!(decoded.one, 1);
        assert_eq!(decoded.two, Some(0x0203));
        assert_eq!(decoded.three, 4);
    }

    // #[test]
    // fn struct_with_fixed_array_of_structs() {
    //     struct Inner {
    //         a: u8,
    //         b: u8,
    //     }

    //     #[derive(RdmGetRequestParameter)]
    //     struct MyStruct {
    //         one: u8,
    //         two: [Inner; 3],
    //         three: u8,
    //     }
    //     let my_struct = MyStruct {
    //         one: 1,
    //         two: [
    //             Inner { a: 2, b: 3 },
    //             Inner { a: 4, b: 5 },
    //             Inner { a: 6, b: 7 },
    //         ],
    //         three: 5,
    //     };
    //     let mut buf = [0u8; 8];
    //     let res = my_struct.get_request_encode_data(&mut buf);
    //     assert!(res.is_ok());
    //     assert_eq!(buf, [1, 2, 3, 4, 5, 6, 7, 5]);
    // }
}
