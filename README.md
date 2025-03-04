# rust-hl7

A Rust library for parsing and processing HL7 (Health Level 7) messages, including ADT (Admission, Discharge, Transfer), ORU (Observation Result), and RDE (Pharmacy/Treatment Encoded Order) messages.

## Features

- Parse HL7 messages into a structured format
- Support for ADT (Admission, Discharge, Transfer) messages
- Support for ORU (Observation Result) messages
- Support for RDE (Pharmacy/Treatment Encoded Order) messages
- Extract patient information, observations, medication orders, and other important data

## Usage

```rust
use rust_hl7::{Message, adt::AdtMessage, oru::OruMessage, rde::RdeMessage};

// Parse an HL7 message
let message_str = "MSH|^~\\&|SENDING_APP|...";
let message = Message::parse(message_str).expect("Failed to parse message");

// Check message type
if message.is_adt() {
    // Process as ADT message
    let adt = AdtMessage::from_hl7(&message).expect("Failed to process ADT");
    println!("Patient ID: {}", adt.patient_id);
} else if message.is_oru() {
    // Process as ORU message
    let oru = OruMessage::from_hl7(&message).expect("Failed to process ORU");
    println!("Patient ID: {}", oru.patient_id);
    
    // Access observations
    for obs in &oru.observations {
        println!("Test: {}, Value: {:?}", obs.test_id, obs.value);
    }
} else if message.is_rde() {
    // Process as RDE message (pharmacy order)
    let rde = RdeMessage::from_hl7(&message).expect("Failed to process RDE");
    println!("Patient ID: {}", rde.patient_id);
    
    // Access medication orders
    for med in &rde.medication_orders {
        println!("Medication: {}, Dose: {:?}, Route: {:?}", 
            med.medication_name.as_deref().unwrap_or("Unknown"), 
            med.dosage, 
            med.route);
    }
}
```

## Supported Message Types

### ADT (Admission, Discharge, Transfer)

ADT messages handle patient administrative data, including:

- A01: Patient admission
- A02: Patient transfer
- A03: Patient discharge
- A04: Patient registration
- A08: Patient information update

### ORU (Observation Result)

ORU messages contain clinical observations and lab results, including:

- R01: Unsolicited observation message

### RDE (Pharmacy/Treatment Encoded Order)

RDE messages contain pharmacy/medication orders, including:

- O11: Pharmacy/treatment encoded order message

## Build and Run

```bash
cargo build
cargo run
```

## License

MIT

