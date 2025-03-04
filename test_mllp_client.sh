#!/bin/bash

# MLLP special characters (ASCII values)
START_BLOCK=$(printf "\x0B")  # VT - Vertical Tab (ASCII 11)
END_BLOCK=$(printf "\x1C")    # FS - File Separator (ASCII 28)
CR=$(printf "\x0D")           # CR - Carriage Return (ASCII 13)

# Sample HL7 message (ADT-A01)
HL7_MESSAGE="MSH|^~\\&|SENDING_APP|SENDING_FACILITY|RECEIVING_APP|RECEIVING_FACILITY|20230401123000||ADT^A01|MSG00001|P|2.5
EVN|A01|20230401123000
PID|1||12345^^^MRN||DOE^JOHN^^^^||19800101|M||W|123 MAIN ST^^ANYTOWN^CA^12345||5551234|||||12345678
NK1|1|DOE^JANE^^^^|SPOUSE|555-5678
PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|"

# Wrap message in MLLP frame
MLLP_FRAME="${START_BLOCK}${HL7_MESSAGE}${END_BLOCK}${CR}"

# Host and port for MLLP server
HOST="127.0.0.1"
PORT="2575"

echo "Connecting to MLLP server at $HOST:$PORT..."
echo "Sending HL7 message wrapped in MLLP frame..."

# Use netcat to send the message and receive the response
# Using echo -e to interpret escape sequences or printf to output the binary data
printf "%s" "$MLLP_FRAME" | nc $HOST $PORT -q 1

echo "Message sent!"