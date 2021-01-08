<div align="center">
	<img src="https://user-images.githubusercontent.com/9824244/104019357-b73e9c00-51bb-11eb-9b46-b49b924d6d8e.png" />
</div>

---

> Chip-8 is a simple, interpreted, programming language which was first used on some 	do-it-yourself computer systems in the late 1970s and early 1980s.

Chipo is a Chip-8 emulator written in Rust. It was built using [this documentation](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) on the Chip-8 specification. It also comes with an assembler tool chain allowing the compilation of assembly like files to Chip-8 bytecode.

## Running programs

Chipo can run both `.c8` files and assembly files written with a syntax inspired by [this specification](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM) ending with `.s` extension (see section [Creating programs](#creating-programs) for more details). To run a pre-compiled file, run:

```bash
./chipo -f space-invader.c8
```

To run an assembly file, run:

```bash
./chipo -f roms/test.s 
```

## Creating programs

Chipo implements an assembler to bytecode translation.

```assembly
.data ; The data section preloads the memory with data
g: 0x1200 0x1200 ; Only unsigned 16 hexadecimal numbers are supported for now

.code
start:
	ld v1 10 ; Load 100 in register v1
	call func ; Call function at address @func
	ld v0 k ; Wait for keyboard event
	ret

func:
	ld v2 0x0F
	ld v3 1
	ld f v3
	drw v2 v1 5
	ret
```

This program can then be compiled to Chip-8 bytecode by running:

```bash
./chipo -f main.s -o main.c8
```

This is useful when working on another Chip-8 emulator and testing specific op codes without having to write binary files by hand. It can also be run directly specifiying no output file, Chipo will recognize the `.s` extension to run the assembly file directly.

