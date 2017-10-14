pub struct Terrain {
	pub id: isize,
	pub tileset: Rc<Tileset>,
	pub name: String,
	pub image_tile_id: isize,
	pub transition: Vec<isize>,
}