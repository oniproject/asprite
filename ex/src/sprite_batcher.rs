use vulkano::device::Queue;
use vulkano::sync::GpuFuture;
use vulkano::swapchain::PresentMode;

use std::sync::Arc;

use specs::*;

use super::*;

use math::*;
use app::Bundle;
use sprite::*;
use renderer::*;

fn terminus() -> Font<'static> {
	let font = include_bytes!("../../res/TerminusTTF-4.46.0.ttf");
	FontCollection::from_bytes(font as &[u8]).into_font().unwrap()
}

pub struct BatcherBundle<'a> {
	pub future: Future,
	pub batcher: Batcher<'a>,
}

impl<'a> BatcherBundle<'a> {
	pub fn new() -> (winit::EventsLoop, Self) {
		let (batcher, index_future, events_loop) = Batcher::new(BATCH_CAPACITY, TEXTURE_COUNT);

		let future = Future::new(index_future);
		(events_loop, Self { future, batcher })
	}
}

impl<'a, 'b> Bundle<'a, 'b> for BatcherBundle<'a> {
	fn bundle(self, world: &mut World, dispatcher: DispatcherBuilder<'a, 'b>) -> DispatcherBuilder<'a, 'b> {
		let Self { future, batcher } = self;

		world.register::<Sprite>();

		world.register::<tsys::Parent>();
		world.register::<Global>();
		world.register::<Local>();
		world.add_resource(future);
		world.add_resource(Vector2::new(1024.0f32, 786.0));

		dispatcher
			.add(SpriteSystem, "sprite", &["transform"])
			.add(batcher, "batcher", &["sprite"])
	}
}

pub struct Batcher<'a> {
	pub renderer: Renderer,
	pub chain: Chain<'a>,
	pub queue: Arc<Queue>,
	pub font: Font<'a>,
}

impl<'a> Batcher<'a> {
	pub fn new(capacity: usize, group_size: u32)
		-> (Self, Box<GpuFuture + Send + Sync>, winit::EventsLoop)
	{
		let (chain, images, events_loop) = Chain::new(|caps| {
			ChainConfig {
				format: caps.supported_formats[0].0,
				present_mode: PresentMode::Fifo,
			}
		});

		let queue = chain.queue.clone();
		let (renderer, index_future) =
			Renderer::new(
				queue.clone(),
				chain.swapchain.clone(),
				&images,
				capacity, group_size).unwrap();

		let font = terminus();
		(
			Self { queue, renderer, font, chain },
			Box::new(index_future),
			events_loop,
		)
	}
}

impl<'a, 'sys> System<'sys> for Batcher<'a> {
	type SystemData = (
		FetchMut<'sys, Future>,
		FetchMut<'sys, Vector2<f32>>,
		ReadStorage<'sys, Sprite>,
		Entities<'sys>,
		Fetch<'sys, Time>,
	);

	fn running_time(&self) -> RunningTime { RunningTime::Long }

	fn run(&mut self, (mut future, mut wh, sprites, e, dt): Self::SystemData) {
		#[cfg(feature = "profiler")] profile_scope!("rendering");
		future.cleanup_finished();

		let (image_num, cb) = {
			#[cfg(feature = "profiler")] profile_scope!("swap");
			let ren = &mut self.renderer;
			match self.chain.run(|m| ren.refill(m)) {
				Some((num, sw_future)) => {
					future.join(sw_future);
					ren.set_num(num);
					(num, ren.clear().unwrap())
				},
				None => return,
			}
		};

		let font_size = 24.0;

		{
			let dim = self.chain.dim();
			self.renderer.resize(dim).unwrap();
			*wh = dim;
		}

		let cb = {
			#[cfg(feature = "profiler")] profile_scope!("sprites");

			let mut cb = self.renderer.start_sprites(cb).unwrap();
			{
				for sprite in sprites.join() {
					cb = self.renderer.texture_quad(cb,
						sprite.texture.clone(),
						sprite.color,
						&sprite.pos, &sprite.uv).unwrap();
				}
			}

			let cb = self.renderer.color_quad(cb,
				Vector2::new(100.0, 200.0),
				Vector2::new(600.0, 200.0 + font_size),
				[0xFF, 0, 0, 0xAA]
			).unwrap();

			self.renderer.end_sprites(cb).unwrap()
		};

		let cb = {
			use math::Transform;
			#[cfg(feature = "profiler")] profile_scope!("vg");
			let min = Vector2::new(100.0, 300.0);
			let max = Vector2::new(600.0, 400.0);
			let color = [0xFF, 0xFF, 0, 0xAA];

			let mut cb = self.renderer.start_vg(cb).unwrap();
			let cb = self.renderer.x_quad(cb, min, max, color).unwrap();

			let mut proj = Affine::one();
			proj.scale(5.0, 5.0);
			proj.translate(150.0, 100.0);
			let color = [0, 0, 0, 0xAA];
			let mesh = build_lyon(proj, color);
			let cb = self.renderer.path(cb, &mesh.vertices, &mesh.indices).unwrap();

			self.renderer.end_vg(cb).unwrap()
		};

		let cb = {
			#[cfg(feature = "profiler")] profile_scope!("text");

			let dt = dt.delta.seconds;

			let text = format!("count: {} ms: {:.4}", e.join().count(), dt);
			let lay: Vec<_> = self.renderer.text_lay(&self.font, font_size, &text, 100.0, 200.0 + font_size).collect();

			self.renderer.glyphs(cb, &lay, [0xFF; 4]).unwrap()
		};

		{
			#[cfg(feature = "profiler")] profile_scope!("end");
			future.then_execute(self.queue.clone(), cb.build().unwrap());
			future.then_swapchain_present(self.queue.clone(), self.chain.swapchain.clone(), image_num);
			future.then_signal_fence_and_flush();
		}
	}
}

pub fn build_lyon(proj: Affine<f32>, color: [u8; 4])
	-> lyon::tessellation::geometry_builder::VertexBuffers<::renderer::vg::Vertex>
{
	use lyon::extra::rust_logo::build_logo_path;
	use lyon::path_builder::*;
	use lyon::math::*;
	use lyon::tessellation::geometry_builder::{VertexConstructor, VertexBuffers, BuffersBuilder};
	use lyon::tessellation::geometry_builder::simple_builder;
	use lyon::tessellation::{FillTessellator, FillOptions};
	use lyon::tessellation::FillVertex;
	use lyon::tessellation;
	use lyon::path::Path;

	use math::Transform;

	let mut builder = SvgPathBuilder::new(Path::builder());
	if true {
		build_logo_path(&mut builder);
	} else {
		builder.move_to(point(0.0, 0.0));
		builder.line_to(point(1.0, 0.0));
		builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
		builder.cubic_bezier_to(point(1.0, 1.0), point(0.0, 1.0), point(0.0, 0.0));
		builder.close();
	}

	let path = builder.build();

	let mut tessellator = FillTessellator::new();

	let mut mesh = VertexBuffers::new();

	{
		tessellator.tessellate_path(
			path.path_iter(),
			&FillOptions::tolerance(0.01),
			//&mut vertex_builder,
			&mut BuffersBuilder::new(&mut mesh, VertexCtor {
				aff: proj,
				color,
			}),
		).unwrap();
	}

	/*
	println!(" -- {} vertices {} indices",
		mesh.vertices.len(),
		mesh.indices.len()
	);
	*/

	mesh
}


type GpuFillVertex = ::renderer::vg::Vertex;

#[derive(Clone)]
struct VertexCtor {
	pub aff: Affine<f32>,
	pub color: [u8; 4],
}
impl lyon::tessellation::geometry_builder::VertexConstructor<lyon::tessellation::FillVertex, GpuFillVertex> for VertexCtor {
	fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> GpuFillVertex {
		use math::Transform;

		debug_assert!(!vertex.position.x.is_nan());
		debug_assert!(!vertex.position.y.is_nan());
		let position = Vector2::new(vertex.position.x, vertex.position.y);
		let position = self.aff.transform_vector(position).into();
		GpuFillVertex {
			position,
			color: self.color,
		}
	}
}

