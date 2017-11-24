PREFIX:=renderer/src

SVERT:=$(PREFIX)/sprite/shader.vert
SFRAG:=$(PREFIX)/sprite/shader.frag
TVERT:=$(PREFIX)/text/shader.vert
TFRAG:=$(PREFIX)/text/shader.frag
VVERT:=$(PREFIX)/vg/shader.vert
VFRAG:=$(PREFIX)/vg/shader.frag

spv:
	glslangValidator -V $(SVERT) -o $(SVERT).spv
	glslangValidator -V $(SFRAG) -o $(SFRAG).spv
	glslangValidator -V $(TVERT) -o $(TVERT).spv
	glslangValidator -V $(TFRAG) -o $(TFRAG).spv
	glslangValidator -V $(VVERT) -o $(VVERT).spv
	glslangValidator -V $(VFRAG) -o $(VFRAG).spv

clean:
	rm -f $(SVERT).spv $(SFRAG).spv
	rm -f $(TVERT).spv $(TFRAG).spv
	rm -f $(VVERT).spv $(VFRAG).spv

vg:
	# i use kcachegrind
	valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes ./target/release/ex
