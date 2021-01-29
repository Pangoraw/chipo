.data

do: 0xE090 0x9090 0xE060 0x9090 0x9060
ne: 0x90D0 0xB090 0x90F0 0x80F0 0x80F0
ex:  0x4040 0x4000 0x4000

.code
	ld v9, 0
    ld v3, 0xA
    ld v2, 3
    
loop2: 
    call draw
    call wait
   	add v9, 1
    
    se v9, 0xA
    jp loop2
    
    ld v0, 0x16
    ld v1, 0x13
    call draw_done
    
    ret

draw_done:
	ld v5, 0
    ld v3, 5
    ld i, do

loop_done:
    drw v0, v1, 5
    add v5, 1
    add v0, 5
    add i, v3

    se v5, 5
    jp loop_done
    ret

draw:
	se v0, 0
	add v2, 6
    add v0, 1
    ld f, v0
    drw v2, v3, 5
    ret

wait:
	ld v1, 0x1F
    ld dt, v1
loop:
	ld v1, dt
	se v1, 0
    jp loop
	ret    
    

