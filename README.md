# rust-hl7

A Rust library for parsing and processing HL7 (Health Level 7) messages, including ADT (Admission, Discharge, Transfer), ORU (Observation Result), and RDE (Pharmacy/Treatment Encoded Order) messages. Also includes an MLLP (Minimal Lower Layer Protocol) server for receiving HL7 messages over a network connection.

## Features

- Parse HL7 messages into a structured format
- Support for ADT (Admission, Discharge, Transfer) messages
- Support for ORU (Observation Result) messages
- Support for RDE (Pharmacy/Treatment Encoded Order) messages
- Extract patient information, observations, medication orders, and other important data
- MLLP server for receiving HL7 messages over TCP/IP
- Automatic message acknowledgment (ACK/NACK) generation

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
# Build the project
cargo build

# Run the message parser demo
cargo run -- parse

# Start the MLLP server (defaults to 127.0.0.1:2575)
cargo run -- server

# Start the MLLP server on a custom address
cargo run -- server --address 0.0.0.0:8080
```

## Using the MLLP Server

The MLLP server listens for HL7 messages over TCP/IP using the Minimal Lower Layer Protocol (MLLP). It automatically generates acknowledgment messages (ACK) for successful processing or negative acknowledgments (NACK) for errors.

### MLLP Message Format

MLLP messages are wrapped with:
- Start block (VT, ASCII 0x0B)
- HL7 message content
- End block (FS, ASCII 0x1C) followed by Carriage Return (CR, ASCII 0x0D)

### Custom Message Processing

You can customize how the server processes messages by modifying the message handler function in `main.rs`:

```rust
let message_handler = Arc::new(|message: Message| -> Result<Message, HL7Error> {
    // Your custom processing logic here
    // For example:
    if message.is_adt() {
        // Process ADT messages
        let adt = AdtMessage::from_hl7(&message)?;
        println!("Received ADT for patient: {}", adt.patient_id);
    }
    
    // Return the message or a response message
    Ok(message)
});
```

## License

Apache
