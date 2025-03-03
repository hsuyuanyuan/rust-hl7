#[cfg(test)]
mod tests {
    use crate::{Message, adt::AdtMessage, oru::OruMessage};

    #[test]
    fn test_parse_adt_message() {
        let adt_message = r#"MSH|^~\&|SENDING_APP|SENDING_FACILITY|RECEIVING_APP|RECEIVING_FACILITY|20230401123000||ADT^A01|MSG00001|P|2.5
EVN|A01|20230401123000
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M||W|123 MAIN ST^^ANYTOWN^CA^12345||5551234|||||12345678
NK1|1|DOE^JANE^^^^|SPOUSE|555-5678
PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|"#;

        let message = Message::parse(adt_message).unwrap();
        assert_eq!(message.message_type, "ADT^A01");
        assert_eq!(message.version, "2.5");
        assert!(message.is_adt());
        assert!(!message.is_oru());

        let adt = AdtMessage::from_hl7(&message).unwrap();
        assert_eq!(adt.event_type, "A01");
        assert_eq!(adt.patient_id, "12345");
        assert_eq!(adt.patient_name, Some("DOE^JOHN^^^^".to_string()));
        assert_eq!(adt.date_of_birth, Some("19800101".to_string()));
        assert_eq!(adt.gender, Some("M".to_string()));
    }

    #[test]
    fn test_parse_oru_message() {
        let oru_message = r#"MSH|^~\&|LAB|FACILITY|EHR|FACILITY|20230401123000||ORU^R01|MSG00002|P|2.5
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M
OBR|1||LAB123456|CBC^COMPLETE BLOOD COUNT^L|||20230401120000
OBX|1|NM|WBC^LEUKOCYTES^L||10.5|10*3/uL|4.0-11.0|N|||F
OBX|2|NM|RBC^ERYTHROCYTES^L||4.5|10*6/uL|4.5-5.9|N|||F"#;

        let message = Message::parse(oru_message).unwrap();
        assert_eq!(message.message_type, "ORU^R01");
        assert_eq!(message.version, "2.5");
        assert!(!message.is_adt());
        assert!(message.is_oru());

        let oru = OruMessage::from_hl7(&message).unwrap();
        assert_eq!(oru.patient_id, "12345");
        assert_eq!(oru.observations.len(), 2);
        
        // Check first observation
        let obs1 = &oru.observations[0];
        assert_eq!(obs1.test_id, "WBC");
        assert_eq!(obs1.test_name, Some("LEUKOCYTES".to_string()));
        assert_eq!(obs1.value, Some("10.5".to_string()));
        assert_eq!(obs1.units, Some("10*3/uL".to_string()));
        assert_eq!(obs1.reference_range, Some("4.0-11.0".to_string()));
        
        // Check second observation
        let obs2 = &oru.observations[1];
        assert_eq!(obs2.test_id, "RBC");
        assert_eq!(obs2.test_name, Some("ERYTHROCYTES".to_string()));
        assert_eq!(obs2.value, Some("4.5".to_string()));
        assert_eq!(obs2.units, Some("10*6/uL".to_string()));
        assert_eq!(obs2.reference_range, Some("4.5-5.9".to_string()));
    }
}