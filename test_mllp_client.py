#!/usr/bin/env python3
import socket
import sys
import time
import subprocess
import threading
import signal
import os

# MLLP special characters
START_BLOCK = b'\x0B'  # VT - Vertical Tab (ASCII 11)
END_BLOCK = b'\x1C'    # FS - File Separator (ASCII 28)
CR = b'\x0D'           # CR - Carriage Return (ASCII 13)

# Sample HL7 message (ADT-A01)
HL7_MESSAGE = b"""MSH|^~\\&|SENDING_APP|SENDING_FACILITY|RECEIVING_APP|RECEIVING_FACILITY|20230401123000||ADT^A01|MSG00001|P|2.5
EVN|A01|20230401123000
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M||W|123 MAIN ST^^ANYTOWN^CA^12345||5551234|||||12345678
NK1|1|DOE^JANE^^^^|SPOUSE|555-5678
PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|"""

def start_server():
    """Start the MLLP server in a separate process"""
    print("Starting MLLP server...")
    server_process = subprocess.Popen(["cargo", "run", "--", "server"], 
                                     stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE)
    
    # Give the server time to start
    time.sleep(2)
    return server_process

def send_message(host="127.0.0.1", port=2575):
    """Send an HL7 message wrapped in MLLP frame to the server"""
    print(f"Connecting to MLLP server at {host}:{port}...")
    
    # Wrap message in MLLP frame
    mllp_frame = START_BLOCK + HL7_MESSAGE + END_BLOCK + CR
    
    try:
        # Create a socket connection to the server
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect((host, port))
        
        print("Sending HL7 message...")
        sock.sendall(mllp_frame)
        
        # Wait for and read the response
        data = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            data += chunk
            
            # If we have a complete message (has end block), break
            if END_BLOCK in data and CR in data:
                break
        
        # Process the response
        if data:
            print("Received response:")
            # Extract the HL7 content from the MLLP frame
            if data.startswith(START_BLOCK) and END_BLOCK in data and CR in data:
                end_pos = data.find(END_BLOCK)
                hl7_content = data[1:end_pos].decode('utf-8')
                print(hl7_content)
            else:
                print("Invalid MLLP frame in response")
                print(data)
        else:
            print("No response received")
        
    except ConnectionRefusedError:
        print("Connection refused. Is the server running?")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        sock.close()
    
    print("Test completed.")

def main():
    """Run the test: start server, send message, cleanup"""
    server_process = start_server()
    
    try:
        # Give the server time to start
        time.sleep(1)
        
        # Send a message
        # send_message(host="127.0.0.1", port=2575)
        send_message(host="18.208.115.20", port=2575) # send to remote EC2
        
    finally:
        # Cleanup: terminate the server process
        print("Stopping server...")
        server_process.terminate()
        try:
            server_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            server_process.kill()
        
        stdout, stderr = server_process.communicate()
        print("Server output:")
        print(stdout.decode('utf-8'))
        
        if stderr:
            print("Server errors:")
            print(stderr.decode('utf-8'))

if __name__ == "__main__":
    main()