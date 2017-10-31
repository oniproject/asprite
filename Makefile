all: vs fs

vs:
	glslangValidator -V src/shader/spritebatch.vert -o src/shader/spritebatch.vert.spv

fs:
	glslangValidator -V src/shader/spritebatch.frag -o src/shader/spritebatch.frag.spv
