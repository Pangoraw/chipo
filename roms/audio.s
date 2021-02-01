.code
start:
	ld v5, 0x1F 
	; update v5 
	; to change the frequency
	ld v0, v5
	ld st, v0
	drw v3, v3, 5
	call wait

	ld v0, v5
	drw v3, v3, 5
	call wait

	jp start

wait:
	ld dt, v0
loop:
	ld v0, dt
	se v0, 0
	jp loop
	ret    

			
		 
