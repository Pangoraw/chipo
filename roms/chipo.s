 ;   ___  _     _  _ __      
 ;  / __|| |_  (_)| '_ \ ___ 
 ; | (__ |   \ | || .__// _ \
 ;  \___||_||_||_||_|   \___/
 ; ===========================
 ; An assembly targeting Chip-8
 ; ===========================

.data ; defines sprites
stripe: 0xF000 0xF000 0xF000
h: 0x9090 0xF090 0x9000
i_letter: 0x7020 0x2020 0x7000
p: 0xE090 0xE080 0x8000

.code ; start code
	ld v0, 10
	ld v1, 12

	ld i, stripe
	call next

	ld v3, 0xC
	ld f, v3
	call next

	ld i, h
	call next

	ld i, i_letter
	call next

	ld i, p
	call next

	ld v3, 0
	ld f, v3
	call next

	ld i, stripe
	call next

	ld v2, k
	ret

next:
	add v0, 5
	drw v0, v1, 5
	ret
