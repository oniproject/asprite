
pub fn create_indices_for_quads(indices: &mut Vec<u16>, size: usize) {
	// the total number of indices in our array, there are 6 points per quad.
	let total = size * 6;
	assert!(total == indices.len());

	//const indices = new Uint16Array(totalIndices);

	// fill the indices with the quads to draw
	let mut i = 0;
	let mut j = 0;
	while i < total {
		indices[i + 0] = j + 0;
		indices[i + 1] = j + 1;
		indices[i + 2] = j + 2;
		indices[i + 3] = j + 0;
		indices[i + 4] = j + 2;
		indices[i + 5] = j + 3;
		i += 6;
		j += 4;
	}
}
