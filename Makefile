PREFIX:=renderer/shader

SVERT:=$(PREFIX)/sprite.vert
SFRAG:=$(PREFIX)/sprite.frag
TVERT:=$(PREFIX)/text.vert
TFRAG:=$(PREFIX)/text.frag

spv:
	glslangValidator -V $(SVERT) -o $(SVERT).spv
	glslangValidator -V $(SFRAG) -o $(SFRAG).spv
	glslangValidator -V $(TVERT) -o $(TVERT).spv
	glslangValidator -V $(TFRAG) -o $(TFRAG).spv

clean:
	rm -f $(SVERT).spv $(SFRAG).spv
	rm -f $(TVERT).spv $(TFRAG).spv

vg:
	# i use kcachegrind
	valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes ./target/release/ex
