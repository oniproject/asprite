PREFIX:=src/batcher/spritebatch
VERT:=$(PREFIX).vert
FRAG:=$(PREFIX).frag

spv:
	glslangValidator -V $(VERT) -o $(VERT).spv
	glslangValidator -V $(FRAG) -o $(FRAG).spv
