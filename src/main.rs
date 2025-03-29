use clap::{Parser, Subcommand};
use rust_hl7::{
    mllp::{MllpError, MllpServer},
    Message, HL7Error, adt::AdtMessage, oru::OruMessage, rde::RdeMessage,
};
use std::sync::Arc;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use tracing_appender::{rolling, non_blocking};
use tracing_appender::rolling::Rotation;

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
        #[arg(short, long, default_value = "0.0.0.0:2575")] // Note: original = 127.0.0.1, only accept conn from localhost
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up sentry integration
    let _guard = sentry::init(("https://605b19c1cb65a806857c8691e1bd5d53@o4508883257196544.ingest.us.sentry.io/4508933895028736", sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    // Define log directory
    let log_dir = "logs";
    
    // Clean up old log files (older than 7 days)
    if let Err(e) = cleanup_old_logs(log_dir, 7) {
        eprintln!("Warning: Failed to clean up old log files: {}", e);
    }

    // Set up logging with file output
    // Create a rotating logger that rotates daily
    let file_appender = rolling::RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "rust-hl7.log",
    );
    
    // Set up non-blocking writer
    let (non_blocking_writer, _logging_guard) = non_blocking(file_appender);
    
    // Configure subscriber with non-blocking writer
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(non_blocking_writer)
        .with_ansi(false)  // This disables color codes
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

    [adt_message, oru_message, rde_message].iter().for_each(|message| {
        match parse_message(message) {
            Ok(result) => println!("{}", result),
            Err(e) => println!("Error parsing message: {}", e),
        }
    })
}

/// Parse an HL7 message string and return parsed message details
fn parse_message(msg_str: &str) -> Result<String, HL7Error> {
    let message = Message::parse(msg_str)?;
    output_message_details(message)
}
fn output_message_details(message: Message) -> Result<String, HL7Error> {
    let mut output = String::new();
    
    match message.message_type.as_str() {
        "ADT^A01" => {
            output.push_str("Successfully parsed ADT message: ");
            output.push_str(&format!("Message type={} ", message.message_type));
            output.push_str(&format!("Version={} ", message.version));

            // Process as ADT
            let adt = AdtMessage::from_hl7(&message)?;
            
            output.push_str("ADT Message Details: ");
            output.push_str(&format!("Event type={} ", adt.event_type));
            output.push_str(&format!("Patient ID={} ", adt.patient_id));
            
            if let Some(name) = adt.patient_name {
                output.push_str(&format!("Patient name={} ", name));
            }
            
            if let Some(dob) = adt.date_of_birth {
                output.push_str(&format!("Date of birth={} ", dob));
            }
            
            if let Some(gender) = adt.gender {
                output.push_str(&format!("Gender={} ", gender));
            }
        }
        "ORU^R01" => {
            output.push_str("Successfully parsed ORU message: ");
            output.push_str(&format!("Message type={} ", message.message_type));
            output.push_str(&format!("Version={} ", message.version));

            // Process as ORU
            let oru = OruMessage::from_hl7(&message)?;
            
            output.push_str("ORU Message Details: ");
            output.push_str(&format!("Patient ID={} ", oru.patient_id));
            output.push_str("Observations: ");

            for (i, obs) in oru.observations.iter().enumerate() {
                output.push_str(&format!("  Observation#{}", i + 1));
                output.push_str(&format!("    Test ID={}", obs.test_id));

                if let Some(name) = &obs.test_name {
                    output.push_str(&format!("    Test name={}\n", name));
                }

                if let Some(value) = &obs.value {
                    output.push_str(&format!("    Value: {}\n", value));
                }

                if let Some(units) = &obs.units {
                    output.push_str(&format!("    Units: {}\n", units));
                }

                if let Some(range) = &obs.reference_range {
                    output.push_str(&format!("    Reference range: {}\n", range));
                }

                if let Some(flags) = &obs.abnormal_flags {
                    output.push_str(&format!("    Abnormal flags: {}\n", flags));
                }
            }
        }
        "RDE^O11" => {
            output.push_str("\nSuccessfully parsed RDE message\n");
            output.push_str(&format!("Message type: {}\n", message.message_type));
            output.push_str(&format!("Version: {}\n", message.version));

            // Process as RDE
            let rde = RdeMessage::from_hl7(&message)?;
            
            output.push_str("\nRDE Message Details:\n");
            output.push_str(&format!("Patient ID: {}\n", rde.patient_id));
            
            if let Some(order_num) = &rde.order_number {
                output.push_str(&format!("Order number: {}\n", order_num));
            }
            
            output.push_str("Medication Orders:\n");

            for (i, med) in rde.medication_orders.iter().enumerate() {
                output.push_str(&format!("  Medication #{}:\n", i + 1));
                output.push_str(&format!("    ID: {}\n", med.medication_id));

                if let Some(name) = &med.medication_name {
                    output.push_str(&format!("    Name: {}\n", name));
                }

                if let Some(strength) = &med.strength {
                    output.push_str(&format!("    Strength: {}\n", strength));
                }

                if let Some(form) = &med.form {
                    output.push_str(&format!("    Form: {}\n", form));
                }

                if let Some(frequency) = &med.frequency {
                    output.push_str(&format!("    Frequency: {}\n", frequency));
                }

                if let Some(route) = &med.route {
                    output.push_str(&format!("    Route: {}\n", route));
                }
            }
        }
        _ => {
            output.push_str(&format!("Unknown message type: {}\n", message.message_type));
        }
    }
    
    Ok(output)
}

/// Cleans up log files older than the specified number of days
fn cleanup_old_logs(log_dir: &str, days: u64) -> std::io::Result<()> {
    let log_path = Path::new(log_dir);
    if !log_path.exists() {
        return Ok(());
    }
    
    let max_age = Duration::from_secs(days * 24 * 60 * 60);
    let now = SystemTime::now();
    
    for entry in fs::read_dir(log_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Ok(metadata) = fs::metadata(&path) {
            if !metadata.is_file() {
                continue;
            }
            
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = now.duration_since(modified) {
                    if duration > max_age {
                        info!("Removing old log file: {}", path.display());
                        fs::remove_file(path)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Runs an MLLP server on the specified address
async fn run_mllp_server(address: &str) -> Result<(), MllpError> {
    info!("Starting MLLP server on {}", address);
    
    // Create a message handler function
    let message_handler = Arc::new(|message: Message| -> Result<Message, HL7Error> {
        // Log the received message type
        info!("Received message of type: {}", message.message_type);

        info!("Message details: {}", output_message_details(message.to_owned())?);
        
        // In a real application, you would process the message here
        // For this example, we'll just echo it back
        Ok(message)
    });
    
    // Create and run the server
    let server = MllpServer::new(address, message_handler);
    server.run().await
}