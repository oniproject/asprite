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
			let lay: Vec<_> = self.renderer.text_lay(&self.font, font_size, &text, 100.0, 200.0 + font_size).collect();
			self.renderer.glyphs(cb, &lay, [0xFF; 4]).unwrap()
		};

		//let cb = draw_vg(cb, &mut self.renderer);
		let cb = draw_ui(cb, &mut self.renderer, &self.font, *mouse);

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


fn draw_ui<'a: 'font, 'font>(cb: CB, renderer: &mut Renderer, font: &'font Font<'a>, mouse: ui::Mouse) -> CB {
	#[cfg(feature = "profiler")] profile_scope!("ui");
	use std::cell::RefCell;
	use std::cell::Cell;
	use ui::Button;

	let gr = Graphics {
		cb: RefCell::new(cb),
		r: RefCell::new(renderer),
		sprite: Cell::new(false),
		font: font,
		font_size: 12.0,

		hovered_widget: Cell::new(None),
	};

	static mut STATE: ui::UiState = ui::UiState::new();
	let mut state = unsafe { &mut STATE };

	let rect = Rect::with_size(500.0, 80.0, 1000.0, 500.0);
	let btn = ui::ColorButton {
		normal: [0x99, 0x99, 0x99, 0x99],
		hovered: [0, 0, 0x99, 0x99],
		active: [0x99, 0, 0, 0x99],
		disabled: [0, 0xFF, 0xFF, 0xCC],
	};

	{
		let ctx = ui::Context::new(&gr, rect, mouse);
		if btn.run(&ctx, &mut state, true) {
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
			if btn.run(&ctx, &mut state, true) {
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
			let anchor = Rect {
				min: Point2::new(0.25, 0.25),
				max: Point2::new(0.25, 0.25),
			};
			let offset = Rect {
				min: Point2::new(0.0, 0.0),
				max: Point2::new(0.0, 0.0),
			};

			let axis = ui::Axis::Horizontal;
			let size = axis.measure(widgets);

			let ctx = ctx.sub().transform(anchor, offset).build();

			let anchor = Rect {
				min: Point2::new(0.5, 0.5),
				max: Point2::new(0.5, 0.5),
			};

			let rect = ctx.rect().dim(size);
			let iter = axis.layout(rect, widgets)
				.map(|offset| ctx.sub().transform(anchor, offset).build());

			let mut i = 0;
			for ctx in iter {
				if btn.run(&ctx, &mut state, i != 3) {
					println!("{} click", i);
				}
				i += 1;
			}
		}
	}

	gr.end()
}

struct Graphics<'a: 'font, 'font> {
	cb: ::std::cell::RefCell<::vulkano::command_buffer::AutoCommandBufferBuilder>,
	r: ::std::cell::RefCell<&'font mut Renderer>,
	sprite: ::std::cell::Cell<bool>,
	font: &'font Font<'a>,
	font_size: f32,

	hovered_widget: ::std::cell::Cell<Option<ui::Id>>,
}

#[inline]
pub fn gen_frame_uv(r: Rect<f32>, tw: f32, th: f32) -> [[u16;2]; 4] {
	let a = r.min.x / tw;
	let b = r.max.x / tw;
	let c = r.min.y / th;
	let d = r.max.y / th;

	[
		pack_uv(a, c),
		pack_uv(b, c),
		pack_uv(b, d),
		pack_uv(a, d),
	]
}

impl<'a, 'font> Graphics<'a, 'font> {
	pub fn end(self) -> ::vulkano::command_buffer::AutoCommandBufferBuilder {
		let Self { cb, sprite, r, .. } = self;

		let cb = cb.into_inner();
		if sprite.get() {
			r.borrow_mut().end_sprites(cb).unwrap()
		} else {
			cb
		}
	}
}

impl<'a, 'font> ui::Graphics for Graphics<'a, 'font> {
	type Texture = Texture;
	type Color = [u8; 4];

	#[inline]
	fn hovered_widget(&self) -> Option<ui::Id> {
		self.hovered_widget.get()
	}

	#[inline]
	fn set_hovered_widget(&self, id: ui::Id) {
		self.hovered_widget.set(Some(id));
	}

	#[inline]
	fn texture_dimensions(&self, texture: &Self::Texture) -> Vector2<f32> {
		let wh = texture.wh;
		Vector2::new(wh.0 as f32, wh.1 as f32)
	}

	#[inline]
	fn quad(&self, color: Self::Color, rect: &Rect<f32>) {
		let mut r = self.r.borrow_mut();
		temporarily_move_out(self.cb.borrow_mut(), |cb| {
			let cb = if self.sprite.get() { cb } else {
				self.sprite.set(true);
				r.start_sprites(cb).unwrap()
			};
			r.color_quad(cb, rect.min.to_vec(), rect.max.to_vec(), color).unwrap()
		});
	}

	#[inline]
	fn texture(&self, texture: &Self::Texture, rect: &Rect<f32>) {
		let &Rect { min, max } = rect;
		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];
		const COLOR: [u8; 4] = [0xFF; 4];
		const UV: [[u16; 2]; 4] = zero_uv();

		let mut r = self.r.borrow_mut();
		temporarily_move_out(self.cb.borrow_mut(), |cb| {
			let cb = if self.sprite.get() { cb } else {
				self.sprite.set(true);
				r.start_sprites(cb).unwrap()
			};
			r.texture_quad(cb, texture.clone(), COLOR, &pos, &UV).unwrap()
		});
	}

	#[inline]
	fn texture_frame(&self, texture: &Self::Texture, rect: &Rect<f32>, frame: &Rect<f32>) {
		let &Rect { min, max } = rect;
		let pos = [
			Vector2::new(min.x, min.y),
			Vector2::new(max.x, min.y),
			Vector2::new(max.x, max.y),
			Vector2::new(min.x, max.y),
		];
		const COLOR: [u8; 4] = [0xFF; 4];

		let uv = gen_frame_uv(*frame, texture.wh.0 as f32, texture.wh.1 as f32);

		let mut r = self.r.borrow_mut();
		temporarily_move_out(self.cb.borrow_mut(), |cb| {
			let cb = if self.sprite.get() { cb } else {
				self.sprite.set(true);
				r.start_sprites(cb).unwrap()
			};
			r.texture_quad(cb, texture.clone(), COLOR, &pos, &uv).unwrap()
		});
	}

	#[inline]
	fn measure_text(&self, _text: &str) -> Vector2<f32> {
		unimplemented!()
	}

	#[inline]
	fn text(&self, base: Point2<f32>, color: Self::Color, text: &str) {
		let mut r = self.r.borrow_mut();
		temporarily_move_out(self.cb.borrow_mut(), |cb| {
			let cb = if !self.sprite.get() { cb } else {
				self.sprite.set(false);
				r.end_sprites(cb).unwrap()
			};
			let lay: Vec<_> = r.text_lay(&self.font, self.font_size, &text, base.x, base.y).collect();
			r.glyphs(cb, &lay, color).unwrap()
		});
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

