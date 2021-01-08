.data
	; preload memory here
	; loaded at runtime
g: 0x6080 0xA090 0x6000 
h: 0x9090 0xF090 0x9000
i_letter: 0x7020 0x2020 0x7000

.code
start:
	cls
	ld v0 5
	ld v1 3
	ld v4 0
	ld v5 0

	; for v5 < 10; ++v5
for:
	sne v5 10
	jp end_for ; inline comment

	call draw
	call next
	add v5 1

	jp for
end_for:

	add v0 6
	ld v1 3
	ld v5 0

for2:
	sne v5 6
	jp end_for2

	call draw
	call next
	add v5 1

	jp for2
end_for2:

	ld i g
	call draw
	call next
	ld i h
	call draw
	call next
	ld i i_letter
	call draw

	call wait
	ret

draw:
	drw v1 v0 5
	ret

next:
	add v4 1
	ld f v4
	add v1 6
	ret

wait:
	ld v3 k
	ret
