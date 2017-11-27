use vulkano::device::Queue;
use vulkano::sync::GpuFuture;
use vulkano::swapchain::PresentMode;

use std::sync::Arc;

use specs::*;

use super::*;

use graphics::*;

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
	graphics: Graphics<'a>,
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
		let graphics = Graphics::new(font, 14.0);
		(
			Self { queue, renderer, chain, graphics },
			Box::new(index_future),
			events_loop,
		)
	}
}

impl<'a, 'sys> System<'sys> for Batcher<'a> {
	type SystemData = (
		FetchMut<'sys, Future>,
		FetchMut<'sys, Vector2<f32>>,
		Fetch<'sys, ui::Mouse>,
		ReadStorage<'sys, Sprite>,
		Entities<'sys>,
		Fetch<'sys, Time>,
	);

	fn running_time(&self) -> RunningTime { RunningTime::Long }

	fn run(&mut self, (mut future, mut wh, mouse, sprites, e, dt): Self::SystemData) {
		#[cfg(feature = "profiler")] profile_scope!("rendering");
		future.cleanup_finished();

		let (image_num, cb) = {
			#[cfg(feature = "profiler")] profile_scope!("swap");
			let ren = &mut self.renderer;
			match self.chain.run(|m| ren.refill(m)) {
				Some((num, sw_future)) => {
					future.join(sw_future);
					ren.set_num(num);

					let dim = self.chain.dim();
					ren.resize(dim).unwrap();
					*wh = dim;

					(num, ren.clear().unwrap())
				},
				None => return,
			}
		};

		let font_size = 24.0;

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

			let cb = if true {
				self.renderer.color_quad(cb,
					Vector2::new(100.0, 200.0),
					Vector2::new(600.0, 200.0 + font_size),
					[0xFF, 0, 0, 0xAA]
				).unwrap()
			} else {
				cb
			};

			self.renderer.end_sprites(cb).unwrap()
		};

		let cb = {
			#[cfg(feature = "profiler")] profile_scope!("text");
			let text = format!("count: {} ms: {:.4}", e.join().count(), dt.delta.seconds);
			let lay: Vec<_> = self.renderer.text_lay(&self.graphics.font, font_size, &text, 100.0, 200.0 + font_size).collect();
			self.renderer.glyphs(cb, &lay, [0xFF; 4]).unwrap()
		};

		draw_ui(&self.graphics, *mouse);

		let cb = draw_vg(cb, &mut self.renderer);
		let cb = self.graphics.run(cb, &mut self.renderer).unwrap();

		{
			#[cfg(feature = "profiler")] profile_scope!("end");
			future.then_execute(self.queue.clone(), cb.build().unwrap());
			future.then_swapchain_present(self.queue.clone(), self.chain.swapchain.clone(), image_num);
			future.then_signal_fence_and_flush();
		}
	}
}

use ::vulkano::command_buffer::AutoCommandBufferBuilder as CB;

#[allow(dead_code)]
fn draw_vg(cb: CB, renderer: &mut Renderer) -> CB {
	use math::Transform;
	#[cfg(feature = "profiler")] profile_scope!("vg");
	let min = Vector2::new(100.0, 300.0);
	let max = Vector2::new(600.0, 400.0);
	let color = [0xFF, 0xFF, 0, 0xAA];

	let cb = renderer.start_vg(cb).unwrap();
	let cb = renderer.x_quad(cb, min, max, color).unwrap();

	let mut proj = Affine::one();
	proj.scale(5.0, 5.0);
	proj.translate(150.0, 100.0);
	let color = [0, 0, 0, 0xAA];
	let mesh = build_lyon(proj, color);
	let cb = renderer.path(cb, &mesh.vertices, &mesh.indices).unwrap();

	renderer.end_vg(cb).unwrap()
}

fn draw_ui<'a>(gr: &Graphics<'a>, mouse: ui::Mouse) {
	#[cfg(feature = "profiler")] profile_scope!("ui");
	use ui::Button;
	use ui::Graphics;
	use ui::Toggle;

	static mut STATE: ui::UiState = ui::UiState::new();
	let state = unsafe { &mut STATE };

	let rect = Rect::with_size(500.0, 80.0, 1000.0, 500.0);
	let btn = ui::ColorButton {
		normal: [0x99, 0x99, 0x99, 0x99],
		hovered: [0, 0, 0x99, 0x99],
		pressed: [0x99, 0, 0, 0x99],
		disabled: [0, 0xFF, 0xFF, 0xCC],
	};

	let toggle = ui::ToggleStyle {
		checked: &ui::ColorButton {
			normal:   [0xFF, 0, 0, 0xCC],
			hovered:  [0xFF, 0, 0, 0x99],
			pressed:  [0xFF, 0, 0, 0x66],
			disabled: [0xFF, 0, 0, 0x33],
		},
		unchecked: &ui::ColorButton {
			normal:   [0xFF, 0xFF, 0xFF, 0xCC],
			hovered:  [0xFF, 0xFF, 0xFF, 0x99],
			pressed:  [0xFF, 0xFF, 0xFF, 0x66],
			disabled: [0xFF, 0xFF, 0xFF, 0x33],
		},
	};

	{
		let ctx = ui::Context::new(gr, rect, mouse);
		if btn.run(&ctx, state, true) {
			println!("X click");
		}
		{
			let anchor = Rect {
				min: Point2::new(0.5, 0.5),
				max: Point2::new(0.5, 0.5),
			};
			let offset = Rect {
				min: Point2::new(-10.0, -10.0),
				max: Point2::new(10.0, 10.0),
			};

			let ctx = ctx.sub().transform(anchor, offset).build();
			if btn.run(&ctx, state, true) {
				println!("Y click");
			}
		}

		let widgets = &[
			ui::Flow::with_wh(40.0, 40.0),
			ui::Flow::with_wh(40.0, 40.0),
			ui::Flow::with_wh(40.0, 40.0),
			ui::Flow::with_wh(40.0, 40.0),
			ui::Flow::with_wh(40.0, 40.0),
		];

		{
			let ctx = {
				let anchor = Rect {
					min: Point2::new(0.25, 0.25),
					max: Point2::new(0.75, 0.75),
				};
				let offset = Rect {
					min: Point2::new(0.0, 0.0),
					max: Point2::new(0.0, 0.0),
				};

				ctx.sub().transform(anchor, offset).build()
			};

			//println!("{:?}", ctx.rect());

			ctx.quad([0xFF, 0, 0, 0xCC], &ctx.rect());

			static mut TOGGLE_STATE: bool = false;
			let toggle_state = unsafe { &mut TOGGLE_STATE };

			let mut i = 0;
			for ctx in ctx.horizontal_flow(0.5, 0.0, widgets) {
				if i == 2 {
					toggle.toggle(&ctx, state, toggle_state, true);
				} else {
					if btn.run(&ctx, state, i != 3) {
						println!("{} click", i);
					}
				}
				i += 1;
			}
		}
	}
}

pub fn build_lyon(proj: Affine<f32>, color: [u8; 4])
	-> lyon::tessellation::geometry_builder::VertexBuffers<::renderer::vg::Vertex>
{
	#![allow(unused_imports)]

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

