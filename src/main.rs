use clap::{Parser, Subcommand};
use rust_hl7::{
    mllp::{MllpError, MllpServer},
    Message, HL7Error, adt::AdtMessage, oru::OruMessage, rde::RdeMessage,
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "rust-hl7")]
#[command(about = "A Rust HL7 Parser and MLLP Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and display HL7 messages (demo)
    Parse,
    
    /// Start the MLLP server
    Server {
        /// Address to bind the server to
        #[arg(short, long, default_value = "127.0.0.1:2575")]
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default subscriber");

    let cli = Cli::parse();

    match cli.command {
        Commands::Parse => {
            run_parse_demo();
        }
        Commands::Server { address } => {
            run_mllp_server(&address).await?;
        }
    }

    Ok(())
}

/// Runs the demo parsing sample HL7 messages
fn run_parse_demo() {
    // Example ADT message (patient admission)
    let adt_message = r#"MSH|^~\&|SENDING_APP|SENDING_FACILITY|RECEIVING_APP|RECEIVING_FACILITY|20230401123000||ADT^A01|MSG00001|P|2.5
EVN|A01|20230401123000
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M||W|123 MAIN ST^^ANYTOWN^CA^12345||5551234|||||12345678
NK1|1|DOE^JANE^^^^|SPOUSE|555-5678
PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|"#;
    
    // Example ORU message (lab results)
    let oru_message = r#"MSH|^~\&|LAB|FACILITY|EHR|FACILITY|20230401123000||ORU^R01|MSG00002|P|2.5
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M
OBR|1||LAB123456|CBC^COMPLETE BLOOD COUNT^L|||20230401120000
OBX|1|NM|WBC^LEUKOCYTES^L||10.5|10*3/uL|4.0-11.0|N|||F
OBX|2|NM|RBC^ERYTHROCYTES^L||4.5|10*6/uL|4.5-5.9|N|||F
OBX|3|NM|HGB^HEMOGLOBIN^L||14.2|g/dL|13.5-17.5|N|||F
OBX|4|NM|HCT^HEMATOCRIT^L||42.1|%|41.0-53.0|N|||F
OBX|5|NM|PLT^PLATELETS^L||250|10*3/uL|150-450|N|||F"#;

    // Example RDE message (pharmacy order)
    let rde_message = r#"MSH|^~\&|PHARMACY|FACILITY|EHR|FACILITY|20230401123000||RDE^O11|MSG00003|P|2.5
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M
ORC|NW|ORD123456|||||^^^20230401^^R|
RXE|AMOX500^AMOXICILLIN 500MG||500|MG|TAB|BID||||||30||SWALLOW||20230401|20230415
RXR|||SWALLOW"#;
    
    // Process ADT message
    match Message::parse(adt_message) {
        Ok(message) => {
            println!("Successfully parsed ADT message");
            println!("Message type: {}", message.message_type);
            println!("Version: {}", message.version);
            
            // Process as ADT
            match AdtMessage::from_hl7(&message) {
                Ok(adt) => {
                    println!("\nADT Message Details:");
                    println!("Event type: {}", adt.event_type);
                    println!("Patient ID: {}", adt.patient_id);
                    if let Some(name) = adt.patient_name {
                        println!("Patient name: {}", name);
                    }
                    if let Some(dob) = adt.date_of_birth {
                        println!("Date of birth: {}", dob);
                    }
                    if let Some(gender) = adt.gender {
                        println!("Gender: {}", gender);
                    }
                }
                Err(e) => println!("Error processing ADT message: {}", e),
            }
        }
        Err(e) => println!("Error parsing ADT message: {}", e),
    }
    
    // Process ORU message
    match Message::parse(oru_message) {
        Ok(message) => {
            println!("\nSuccessfully parsed ORU message");
            println!("Message type: {}", message.message_type);
            println!("Version: {}", message.version);
            
            // Process as ORU
            match OruMessage::from_hl7(&message) {
                Ok(oru) => {
                    println!("\nORU Message Details:");
                    println!("Patient ID: {}", oru.patient_id);
                    println!("Observations:");
                    
                    for (i, obs) in oru.observations.iter().enumerate() {
                        println!("  Observation #{}:", i + 1);
                        println!("    Test ID: {}", obs.test_id);
                        
                        if let Some(name) = &obs.test_name {
                            println!("    Test name: {}", name);
                        }
                        
                        if let Some(value) = &obs.value {
                            println!("    Value: {}", value);
                        }
                        
                        if let Some(units) = &obs.units {
                            println!("    Units: {}", units);
                        }
                        
                        if let Some(range) = &obs.reference_range {
                            println!("    Reference range: {}", range);
                        }
                        
                        if let Some(flags) = &obs.abnormal_flags {
                            println!("    Abnormal flags: {}", flags);
                        }
                    }
                }
                Err(e) => println!("Error processing ORU message: {}", e),
            }
        }
        Err(e) => println!("Error parsing ORU message: {}", e),
    }
    
    // Process RDE message
    match Message::parse(rde_message) {
        Ok(message) => {
            println!("\nSuccessfully parsed RDE message");
            println!("Message type: {}", message.message_type);
            println!("Version: {}", message.version);
            
            // Process as RDE
            match RdeMessage::from_hl7(&message) {
                Ok(rde) => {
                    println!("\nRDE Message Details:");
                    println!("Patient ID: {}", rde.patient_id);
                    if let Some(order_num) = &rde.order_number {
                        println!("Order number: {}", order_num);
                    }
                    println!("Medication Orders:");
                    
                    for (i, med) in rde.medication_orders.iter().enumerate() {
                        println!("  Medication #{}:", i + 1);
                        println!("    ID: {}", med.medication_id);
                        
                        if let Some(name) = &med.medication_name {
                            println!("    Name: {}", name);
                        }
                        
                        if let Some(strength) = &med.strength {
                            println!("    Strength: {}", strength);
                        }
                        
                        if let Some(form) = &med.form {
                            println!("    Form: {}", form);
                        }
                        
                        if let Some(frequency) = &med.frequency {
                            println!("    Frequency: {}", frequency);
                        }
                        
                        if let Some(route) = &med.route {
                            println!("    Route: {}", route);
                        }
                    }
                }
                Err(e) => println!("Error processing RDE message: {}", e),
            }
        }
        Err(e) => println!("Error parsing RDE message: {}", e),
    }
}

/// Runs an MLLP server on the specified address
async fn run_mllp_server(address: &str) -> Result<(), MllpError> {
    info!("Starting MLLP server on {}", address);
    
    // Create a message handler function
    let message_handler = Arc::new(|message: Message| -> Result<Message, HL7Error> {
        // Log the received message type
        info!("Received message of type: {}", message.message_type);
        
        // In a real application, you would process the message here
        // For this example, we'll just echo it back
        Ok(message)
    });
    
    // Create and run the server
    let server = MllpServer::new(address, message_handler);
    server.run().await
}