// Input data
.data input-data 0x0..auto
    "test data"
    """multiline
    string goes
    here"""
    0xaf1b
    27
    0b10001

// Array storage, starts at next available word and extends for 32 words.
.data storage auto..32

// Entrypoint
.text main auto..auto
    add r0 imm 10
    dmp r0
    hlt
