.text main 0x0..auto
    clr r0
    nop2
    clr r1
    ldr r1 abs 0xa0
    inc r1
    ldr r0 abs 0x90
    add r0 r1
    str r0 d1 0x84
    blt r1 pcr 0x05

.data inp1 0x0f..auto
    0x0f

.text blt 0x12..auto
    nop2 r1
    jmp r0 abs @jump

.data inp2 0x15..auto
    0x0f

.text jump 0x3e..auto
    bez r0 pcr 0x01
    dmp r0
    cmp r0
    cmp r0 d0
    and r1
    clr r1
    inc r1
    clr r0
    sll r1 d0 0xa1
    sra r1 imm 0x02
    nop1 r0 abs
    hlt

.data inputs1 0x80..auto
    0xbd

.data inputs2 0x90..auto
    0x10

.data inputs3 0xa0..auto
    0xfb
    0x03

