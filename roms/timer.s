.code
	ld v9, 0
    ld v3, 0xD
    ld v2, 3
    ld v1, 0xD
    
loop2: 
    call draw
    call wait
   	add v9, 1
    
    se v9, 0xA
    jp loop2
    
    ret

draw:
	se v0, 0
	add v2, 6
    add v0, 1
    ld f, v0
    drw v2, v3, 5
    ret

wait:
	ld v1, 0xf
    ld dt, v1
loop:
	ld v1, dt
	se v1, 0
    jp loop
	ret    
    
