use crate::Message;
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Decoder, Encoder};
use tracing::{error, info, warn};

// MLLP specific constants
const MLLP_START_BLOCK: u8 = 0x0B; // Vertical Tab
const MLLP_END_BLOCK: u8 = 0x1C;   // File Separator
const MLLP_CARRIAGE_RETURN: u8 = 0x0D; // Carriage Return

/// Errors that can occur in MLLP operations
#[derive(Debug, Error)]
pub enum MllpError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid MLLP frame: {0}")]
    InvalidFrame(String),
    
    #[error("HL7 error: {0}")]
    Hl7Error(#[from] crate::HL7Error),
}

/// Codec for encoding/decoding MLLP frames
pub struct MllpCodec;

impl Decoder for MllpCodec {
    type Item = Bytes;
    type Error = MllpError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Look for start block
        if let Some(start_pos) = src.iter().position(|&b| b == MLLP_START_BLOCK) {
            // Remove anything before the start block
            if start_pos > 0 {
                let _ = src.split_to(start_pos);
            }
            
            // Now look for end sequence (FS + CR)
            if let Some(end_pos) = src.windows(2).position(|w| w[0] == MLLP_END_BLOCK && w[1] == MLLP_CARRIAGE_RETURN) {
                // We have a complete message
                // Extract the entire framed message including start and end markers
                let mut framed_message = src.split_to(end_pos + 2);
                
                // Skip the start block
                let _ = framed_message.split_to(1);
                
                // Create a new BytesMut with just the message content (without end sequence)
                let content_len = framed_message.len() - 2; // Subtract the end sequence
                let content = framed_message.split_to(content_len);
                
                return Ok(Some(content.freeze()));
            }
        }
        
        // No complete message yet
        if src.len() > 100_000 {
            // If buffer gets too large without finding a valid frame, something is wrong
            return Err(MllpError::InvalidFrame("Buffer exceeds maximum size without valid frame".to_string()));
        }
        
        Ok(None)
    }
}

impl Encoder<Bytes> for MllpCodec {
    type Error = MllpError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Add start block
        dst.extend_from_slice(&[MLLP_START_BLOCK]);
        
        // Add message content
        dst.extend_from_slice(&item);
        
        // Add end sequence
        dst.extend_from_slice(&[MLLP_END_BLOCK, MLLP_CARRIAGE_RETURN]);
        
        Ok(())
    }
}

/// Handler function for processing received HL7 messages
pub type MessageHandler = Arc<dyn Fn(Message) -> Result<Message, crate::HL7Error> + Send + Sync>;

/// MLLP Server that listens for connections and handles HL7 messages
pub struct MllpServer {
    address: String,
    handler: MessageHandler,
}

impl MllpServer {
    /// Create a new MLLP server with specified address and message handler
    pub fn new<A: ToString>(address: A, handler: MessageHandler) -> Self {
        Self {
            address: address.to_string(),
            handler,
        }
    }

    /// Start the MLLP server
    pub async fn run(&self) -> Result<(), MllpError> {
        let listener = TcpListener::bind(&self.address).await?;
        info!("MLLP server listening on {}", self.address);

        loop {
            let (socket, addr) = match listener.accept().await {
                Ok(accepted) => accepted,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            info!("New connection from {}", addr);
            
            // Clone the handler for the new connection
            let handler = self.handler.clone();
            
            // Spawn a new task to handle this connection
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, addr, handler).await {
                    error!("Error handling connection from {}: {}", addr, e);
                }
            });
        }
    }
}

/// Handle a single MLLP connection
async fn handle_connection(
    mut socket: TcpStream,
    addr: std::net::SocketAddr,
    handler: MessageHandler,
) -> Result<(), MllpError> {
    let (read_half, mut write_half) = socket.split();
    
    let mut read_buffer = BytesMut::with_capacity(4096);
    let mut read_half = tokio::io::BufReader::new(read_half);
    
    loop {
        // Read data into the buffer
        let bytes_read = read_half.read_buf(&mut read_buffer).await?;
        if bytes_read == 0 {
            // Connection closed
            info!("Connection closed by {}", addr);
            break;
        }
        
        // Check for a complete MLLP frame
        if let Some(message_bytes) = extract_mllp_message(&mut read_buffer)? {
            info!("Received message ({} bytes)", message_bytes.len());
            
            // Convert to string
            let message_str = match std::str::from_utf8(&message_bytes) {
                Ok(s) => s.to_string(),
                Err(e) => {
                    warn!("Received non-UTF8 message: {}", e);
                    // Skip this message
                    continue;
                }
            };
            
            // Parse HL7 message
            match Message::parse(&message_str) {
                Ok(hl7_message) => {
                    // Process the message with the handler
                    match handler(hl7_message) {
                        Ok(response) => {
                            // Generate acknowledgment
                            let ack = generate_response(&response)?;
                            
                            // Wrap in MLLP frame
                            let mllp_response = wrap_in_mllp(&ack);
                            
                            // Send the response
                            write_half.write_all(&mllp_response).await?;
                            info!("Sent response ({} bytes)", mllp_response.len());
                        }
                        Err(e) => {
                            error!("Error processing message: {}", e);
                            // Send a negative acknowledgment
                            let nack = generate_nack(&message_str, &e.to_string())?;
                            let mllp_nack = wrap_in_mllp(&nack);
                            write_half.write_all(&mllp_nack).await?;
                        }
                    }
                }
                Err(e) => {
                    error!("Error parsing HL7 message: {}", e);
                    // Send a negative acknowledgment
                    let nack = generate_nack(&message_str, &e.to_string())?;
                    let mllp_nack = wrap_in_mllp(&nack);
                    write_half.write_all(&mllp_nack).await?;
                }
            }
        }
    }
    
    Ok(())
}

/// Extract a complete MLLP message from the buffer
fn extract_mllp_message(buffer: &mut BytesMut) -> Result<Option<Bytes>, MllpError> {
    // Look for start block
    if let Some(start_pos) = buffer.iter().position(|&b| b == MLLP_START_BLOCK) {
        // Remove anything before the start block
        if start_pos > 0 {
            let _ = buffer.split_to(start_pos);
        }
        
        // Now look for end sequence (FS + CR)
        if let Some(end_pos) = buffer.windows(2).position(|w| w[0] == MLLP_END_BLOCK && w[1] == MLLP_CARRIAGE_RETURN) {
            // We have a complete message
            // Extract the entire framed message including start and end markers
            let mut framed_message = buffer.split_to(end_pos + 2);
            
            // Skip the start block
            let _ = framed_message.split_to(1);
            
            // Create a new BytesMut with just the message content (without end sequence)
            let content_len = framed_message.len() - 2; // Subtract the end sequence
            let content = framed_message.split_to(content_len);
            
            return Ok(Some(content.freeze()));
        }
    }
    
    // No complete message yet
    if buffer.len() > 100_000 {
        // If buffer gets too large without finding a valid frame, something is wrong
        return Err(MllpError::InvalidFrame("Buffer exceeds maximum size without valid frame".to_string()));
    }
    
    Ok(None)
}

/// Wrap an HL7 message in MLLP frame
fn wrap_in_mllp(message: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(message.len() + 3);
    result.push(MLLP_START_BLOCK);
    result.extend_from_slice(message.as_bytes());
    result.push(MLLP_END_BLOCK);
    result.push(MLLP_CARRIAGE_RETURN);
    result
}

/// Generate an HL7 ACK (acknowledgment) message for the given message
fn generate_response(_message: &Message) -> Result<String, MllpError> {
    // In a real implementation, you would build a proper ACK message based on the input
    // For this example, we'll create a simple ACK
    
    // Get current time in HL7 format
    let now = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    
    // Build ACK message
    // In a real implementation, we would extract the message control ID from the original message
    // and other fields to create a proper ACK
    let ack = format!(
        "MSH|^~\\&|RECEIVING_APP|RECEIVING_FACILITY|SENDING_APP|SENDING_FACILITY|{}||ACK|MSG00001|P|2.5\r\n\
         MSA|AA|MSG00001|Message processed successfully",
        now
    );
    
    Ok(ack)
}

/// Generate a negative acknowledgment (NACK) message for a failed HL7 message
fn generate_nack(original_message: &str, error_msg: &str) -> Result<String, MllpError> {
    // Get current time in HL7 format
    let now = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    
    // Find message control ID from original message, defaulting to "UNKNOWN" if not found
    let control_id = if let Some(msh_line) = original_message.lines().next() {
        let fields: Vec<&str> = msh_line.split('|').collect();
        fields.get(9).unwrap_or(&"UNKNOWN").to_string()
    } else {
        "UNKNOWN".to_string()
    };
    
    // Build NACK message
    let nack = format!(
        "MSH|^~\\&|RECEIVING_APP|RECEIVING_FACILITY|SENDING_APP|SENDING_FACILITY|{}||ACK|{}|P|2.5\r\n\
         MSA|AE|{}|Error processing message: {}",
        now, control_id, control_id, error_msg
    );
    
    Ok(nack)
}