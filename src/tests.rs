#[cfg(test)]
mod tests {
    use crate::{Message, adt::AdtMessage, oru::OruMessage, rde::RdeMessage};

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
        assert!(!message.is_rde());

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
        assert!(!message.is_rde());

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
    
    #[test]
    fn test_parse_rde_message() {
        let rde_message = r#"MSH|^~\&|PHARMACY|FACILITY|EHR|FACILITY|20230401123000||RDE^O11|MSG00003|P|2.5
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M
ORC|NW|ORD12345|||||||20230401123000|||
RXE|509^MEDROL|2|4MG||TAB|BID||509^MEDROL|10|||||||||||20230401|20230407
RXR|PO|ORAL|SWALLOW
RXE|123^AMOXICILLIN|3|500MG||CAP|TID||123^AMOXICILLIN|21|||||||||||20230401|20230408
RXR|PO|ORAL|SWALLOW"#;

        let message = Message::parse(rde_message).unwrap();
        assert_eq!(message.message_type, "RDE^O11");
        assert_eq!(message.version, "2.5");
        assert!(!message.is_adt());
        assert!(!message.is_oru());
        assert!(message.is_rde());

        let rde = RdeMessage::from_hl7(&message).unwrap();
        assert_eq!(rde.patient_id, "12345");
        assert_eq!(rde.order_control, Some("NW".to_string()));
        assert_eq!(rde.order_number, Some("ORD12345".to_string()));
        assert_eq!(rde.medication_orders.len(), 2);
        
        // Check first medication order
        let med1 = &rde.medication_orders[0];
        assert_eq!(med1.rx_id, "RX1");
        assert_eq!(med1.medication_id, "509");
        assert_eq!(med1.medication_name, Some("MEDROL".to_string()));
        assert_eq!(med1.strength, Some("4MG".to_string()));
        assert_eq!(med1.form, Some("TAB".to_string()));
        assert_eq!(med1.frequency, Some("BID".to_string()));
        assert_eq!(med1.route, Some("SWALLOW".to_string()));
        assert_eq!(med1.start_date, Some("20230401".to_string()));
        assert_eq!(med1.stop_date, Some("20230407".to_string()));
        
        // Check second medication order
        let med2 = &rde.medication_orders[1];
        assert_eq!(med2.rx_id, "RX2");
        assert_eq!(med2.medication_id, "123");
        assert_eq!(med2.medication_name, Some("AMOXICILLIN".to_string()));
        assert_eq!(med2.strength, Some("500MG".to_string()));
        assert_eq!(med2.form, Some("CAP".to_string()));
        assert_eq!(med2.frequency, Some("TID".to_string()));
        assert_eq!(med2.route, Some("SWALLOW".to_string()));
        assert_eq!(med2.start_date, Some("20230401".to_string()));
        assert_eq!(med2.stop_date, Some("20230408".to_string()));
    }
}