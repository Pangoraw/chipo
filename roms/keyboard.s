.code
start:    
	ld v0, k
	ld f, v0
    
	drw v2, v1, 5
	add v2, 6

 	call wait
	jp start

wait:
	ld v3, 0x2F
	ld dt, v3

loop:
	ld v3, dt
	se v3, 0
	jp loop
	ret    
    
