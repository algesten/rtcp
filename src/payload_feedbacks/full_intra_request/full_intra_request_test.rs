use super::*;

#[test]
fn test_full_intra_request_unmarshal() {
    let tests = vec![
        (
            "valid",
            Bytes::from_static(&[
                0x84, 0xce, 0x00, 0x03, // v=2, p=0, FMT=4, PSFB, len=3
                0x00, 0x00, 0x00, 0x00, // ssrc=0x0
                0x4b, 0xc4, 0xfc, 0xb4, // ssrc=0x4bc4fcb4
                0x12, 0x34, 0x56, 0x78, // ssrc=0x12345678
                0x42, 0x00, 0x00, 0x00, // Seqno=0x42
            ]),
            FullIntraRequest {
                sender_ssrc: 0x0,
                media_ssrc: 0x4bc4fcb4,
                fir: vec![FirEntry {
                    ssrc: 0x12345678,
                    sequence_number: 0x42,
                }],
            },
            None,
        ),
        (
            "also valid",
            Bytes::from_static(&[
                0x84, 0xce, 0x00, 0x05, // v=2, p=0, FMT=4, PSFB, len=3
                0x00, 0x00, 0x00, 0x00, // ssrc=0x0
                0x4b, 0xc4, 0xfc, 0xb4, // ssrc=0x4bc4fcb4
                0x12, 0x34, 0x56, 0x78, // ssrc=0x12345678
                0x42, 0x00, 0x00, 0x00, // Seqno=0x42
                0x98, 0x76, 0x54, 0x32, // ssrc=0x98765432
                0x57, 0x00, 0x00, 0x00, // Seqno=0x57
            ]),
            FullIntraRequest {
                sender_ssrc: 0x0,
                media_ssrc: 0x4bc4fcb4,
                fir: vec![
                    FirEntry {
                        ssrc: 0x12345678,
                        sequence_number: 0x42,
                    },
                    FirEntry {
                        ssrc: 0x98765432,
                        sequence_number: 0x57,
                    },
                ],
            },
            None,
        ),
        (
            "packet too short",
            Bytes::from_static(&[0x00, 0x00, 0x00, 0x00]),
            FullIntraRequest::default(),
            Some(Error::PacketTooShort),
        ),
        (
            "invalid header",
            Bytes::from_static(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
            FullIntraRequest::default(),
            Some(Error::BadVersion),
        ),
        (
            "wrong type",
            Bytes::from_static(&[
                0x84, 0xc9, 0x00, 0x03, // v=2, p=0, FMT=4, RR, len=3
                0x00, 0x00, 0x00, 0x00, // ssrc=0x0
                0x4b, 0xc4, 0xfc, 0xb4, // ssrc=0x4bc4fcb4
                0x12, 0x34, 0x56, 0x78, // ssrc=0x12345678
                0x42, 0x00, 0x00, 0x00, // Seqno=0x42
            ]),
            FullIntraRequest::default(),
            Some(Error::WrongType),
        ),
        (
            "wrong fmt",
            Bytes::from_static(&[
                0x82, 0xce, 0x00, 0x03, // v=2, p=0, FMT=2, PSFB, len=3
                0x00, 0x00, 0x00, 0x00, // ssrc=0x0
                0x4b, 0xc4, 0xfc, 0xb4, // ssrc=0x4bc4fcb4
                0x12, 0x34, 0x56, 0x78, // ssrc=0x12345678
                0x42, 0x00, 0x00, 0x00, // Seqno=0x42
            ]),
            FullIntraRequest::default(),
            Some(Error::WrongType),
        ),
    ];

    for (name, data, want, want_error) in tests {
        let got = FullIntraRequest::unmarshal(&data);

        assert_eq!(
            got.is_err(),
            want_error.is_some(),
            "Unmarshal {} rr: err = {:?}, want {:?}",
            name,
            got,
            want_error
        );

        if let Some(err) = want_error {
            let got_err = got.err().unwrap();
            assert_eq!(
                got_err, err,
                "Unmarshal {} rr: err = {:?}, want {:?}",
                name, got_err, err,
            );
        } else {
            let actual = got.unwrap();
            assert_eq!(
                actual, want,
                "Unmarshal {} rr: got {:?}, want {:?}",
                name, actual, want
            );
        }
    }
}

#[test]
fn test_full_intra_request_round_trip() {
    let tests: Vec<(&str, FullIntraRequest, Result<(), Error>)> = vec![
        (
            "valid",
            FullIntraRequest {
                sender_ssrc: 1,
                media_ssrc: 2,
                fir: vec![FirEntry {
                    ssrc: 3,
                    sequence_number: 42,
                }],
            },
            Ok(()),
        ),
        (
            "also valid",
            FullIntraRequest {
                sender_ssrc: 5000,
                media_ssrc: 6000,
                fir: vec![FirEntry {
                    ssrc: 3,
                    sequence_number: 57,
                }],
            },
            Ok(()),
        ),
    ];

    for (name, fir, marshal_error) in tests {
        let data = fir.marshal();

        assert_eq!(
            data.is_ok(),
            marshal_error.is_ok(),
            "Marshal {}: err = {:?}, want {:?}",
            name,
            data,
            marshal_error
        );

        match data {
            Ok(e) => {
                let decoded =
                    FullIntraRequest::unmarshal(&e).expect(format!("Unmarshal {}", name).as_str());

                assert_eq!(
                    decoded, fir,
                    "{} rr round trip: got {:?}, want {:?}",
                    name, decoded, fir
                );
            }

            Err(_) => continue,
        }
    }
}

#[test]
fn test_full_intra_request_unmarshal_header() {
    let tests = vec![(
        "valid header",
        Bytes::from_static(&[
            0x84, 0xce, 0x00, 0x02, // v=2, p=0, FMT=1, PSFB, len=1
            0x00, 0x00, 0x00, 0x00, // ssrc=0x0
            0x4b, 0xc4, 0xfc, 0xb4, 0x00, 0x00, 0x00, 0x00, // ssrc=0x4bc4fcb4
        ]),
        Header {
            count: FORMAT_FIR,
            packet_type: PacketType::PayloadSpecificFeedback,
            length: 2,
            ..Default::default()
        },
    )];

    for (name, data, want) in tests {
        let result = FullIntraRequest::unmarshal(&data);

        assert!(
            result.is_ok(),
            "Unmarshal header {} rr: want {:?}",
            name,
            result,
        );

        match result {
            Err(_) => continue,

            Ok(fir) => {
                let h = fir.header();

                assert_eq!(
                    h, want,
                    "Unmarshal header {} rr: got {:?}, want {:?}",
                    name, h, want
                )
            }
        }
    }
}
