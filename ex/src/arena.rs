use rand::distributions::{IndependentSample, Range};
use rand::{thread_rng, Rng};
use math::Vector2;
use specs::*;
use std::sync::Arc;
use vulkano::device::Queue;

use super::*;
use winit::Event;
use ui::Mouse;
use renderer::*;
use sprite::*;
use app::*;
use state;
use toml;
use math::*;

pub struct Velocity {
	pub vel: Vector2<f32>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

pub struct Scene {
	pub textures: Vec<Texture>,
	pub queue: Arc<Queue>,
}

/*
pub trait Format {
	type Options: Clone + Send + Sync + 'static;

	fn import(&self, bytes: Vec<u8>, options: Self::Options) -> Result<A::Data>;
}
*/

use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};


error_chain! {
	foreign_links {
		Io(::std::io::Error);
		TomlDe(::toml::de::Error);
	}
}

#[derive(Debug)]
pub struct Directory {
	loc: PathBuf,
}

impl Directory {
	pub fn new<P: Into<PathBuf>>(loc: P) -> Self {
		Self { loc: loc.into() }
	}
}

impl Source for Directory {
	fn path(&self, s_path: &Path) -> PathBuf {
		let mut path = self.loc.clone();
		path.push(s_path);
		path
	}
}

pub trait Source {
	fn path(&self, &Path) -> PathBuf;
}

pub trait Loader<'a> {
	type Asset;
	type Options: 'a;
	fn load<P: AsRef<Path>>(src: &Source, path: P, options: Self::Options) -> Self::Asset;
}

struct TextureFormat;

impl<'a> Loader<'a> for TextureFormat {
	type Asset = Result<Vec<Texture>>;
	type Options = (&'a mut Future, Arc<Queue>);

	fn load<P: AsRef<Path>>(src: &Source, path: P, mut options: Self::Options) -> Self::Asset {
		#[derive(Debug, Deserialize)]
		struct Assets {
			images: Vec<String>,
		}

		let path = src.path(path.as_ref());
		let mut f = File::open(path)?;
		let mut buffer = Vec::new();
		f.read_to_end(&mut buffer)?;

		let decoded: Assets = toml::from_slice(&buffer)?;
		println!("{:#?}", decoded);
		Ok(
			decoded.images.iter()
				.map(|name| src.path(name.as_ref()))
				.map(|name| Texture::join_load(&mut options.0, options.1.clone(), name))
				.map(|t| t.unwrap())
				.collect()
		)
	}
}

impl state::State<World, Event> for Scene {
	fn switch(&mut self, world: &mut World, event: state::ExecEvent) {
		use state::ExecEvent::*;
		println!("arena state change event: {:?}", event);
		match event {
			Start | Resume => {
				if self.textures.is_empty() {
					let mut future = world.write_resource::<Future>();
					let src = Directory::new("./res");
					let options = (&mut *future, self.queue.clone());
					let iter = TextureFormat::load(&src, "res.toml", options);
					self.textures = iter.unwrap();
				};
				for _ in 0..300 {
					spawn(world, &self.textures);
				}
			}
			Stop | Pause => {
				world.delete_all();
			}
		}
	}

	fn update(&mut self, world: &mut World) -> SceneTransition<Event> {
		let add = {
			let count = world.entities().join().count();
			let wh = world.read_resource::<Vector2<f32>>();
			let mouse = world.read_resource::<Mouse>();
			let graphics = world.read_resource::<graphics::Graphics>();
			let dt = world.read_resource::<Time>().delta.seconds;
			draw_ui(&graphics, *wh, *mouse, count, dt)
		};

		for _ in 0..add {
			spawn(world, &self.textures);
		}

		None
	}

	fn late_update(&mut self, world: &mut World) -> SceneTransition<Event> {
		world.write_resource::<Mouse>().cleanup();
		None
	}

	fn fixed_update(&mut self, world: &mut World) -> SceneTransition<Event> {
		let gravity = 0.75;

		let dt = world.read_resource::<Time>().fixed.seconds;
		let size = *world.read_resource::<Vector2<f32>>();

		let mut speed = world.write::<Velocity>();
		let mut sprites = world.write::<Local>();

		let dt = dt * 50.0;
		let between = Range::new(0.0, 10.0);

		//use rayon::prelude::*;
		(&mut speed, &mut sprites).join().for_each(|(speed, sprite)| {
			let sprite = &mut sprite.0;
			let speed = &mut speed.vel;
			sprite.t += *speed * dt;
			speed.y += gravity * dt;

			if sprite.t.x > size.x {
				speed.x *= -1.0;
				sprite.t.x = size.x;
			} else if sprite.t.x < 0.0 {
				speed.x *= -1.0;
				sprite.t.x = 0.0;
			}

			if sprite.t.y > size.y {
				speed.y *= -0.85;
				sprite.t.y = size.y;
				let mut rng = thread_rng();
				if rng.gen() {
					speed.y -= between.ind_sample(&mut rng);
				}
			} else if sprite.t.y < 0.0 {
				speed.y = 0.0;
				sprite.t.y = 0.0;
			}
		});
		None
	}

	fn event(&mut self, world: &mut World, event: Event) -> SceneTransition<Event> {
		match event {
			Event::WindowEvent { event, .. } => {
				use winit::WindowEvent::*;
				match event {
					Closed => {
						return Some(state::Transition::Pop);
					}
					MouseInput { state, button, .. } => {
						mouse_event_buttons(&mut world.write_resource::<Mouse>(), state, button);
					}
					MouseMoved { position, .. } => {
						mouse_event_movement(&mut world.write_resource::<Mouse>(), position);
					}
					_ => (),
				}
			}
			_ => (),
		}
		None
	}
}

fn spawn(world: &mut World, textures: &[Texture]) {
	let mut rng = thread_rng();
	let between = Range::new(0.0, 10.0);
	let tex = Range::new(0, textures.len());

	let x = between.ind_sample(&mut rng);
	let y = between.ind_sample(&mut rng) - 5.0;

	let tex = tex.ind_sample(&mut rng);
	let t = &textures[tex];

	let mut sprite = Sprite::new(t.clone());
	sprite.anchor.y = 1.0;
	sprite.size.x = t.wh.0 as f32;
	sprite.size.y = t.wh.1 as f32;

	let speed = Velocity {
		vel: Vector2::new(x, y),
	};

	let local = Local::default();
	let global = Global::default();

	world.create_entity()
		.with(sprite)
		.with(local)
		.with(global)
		.with(speed)
		.build();
}

fn draw_ui(gr: &graphics::Graphics, wh: Vector2<f32>, mouse: ui::Mouse, entity_count: usize, dt: f32) -> usize {
	#[cfg(feature = "profiler")] profile_scope!("ui");
	use ui::*;

	static mut STATE: ui::UiState = ui::UiState::new();
	let state = unsafe { &mut STATE };

	let rect = Rect::with_size(0.0, 0.0, wh.x, wh.y);
	let btn = ColorButton {
		normal:  ColorDrawer::new([0x99, 0x99, 0x99, 0x99]),
		hovered: ColorDrawer::new([0, 0, 0x99, 0x99]),
		pressed: ColorDrawer::new([0x99, 0, 0, 0x99]),
	};

	let toggle = Toggle {
		checked: ColorButton {
			normal:   ColorDrawer::new([0xFF, 0, 0, 0xCC]),
			hovered:  ColorDrawer::new([0xFF, 0, 0, 0x99]),
			pressed:  ColorDrawer::new([0xFF, 0, 0, 0x66]),
		},
		unchecked: ColorButton {
			normal:   ColorDrawer::new([0xFF, 0xFF, 0xFF, 0xCC]),
			hovered:  ColorDrawer::new([0xFF, 0xFF, 0xFF, 0x99]),
			pressed:  ColorDrawer::new([0xFF, 0xFF, 0xFF, 0x66]),
		},
	};

	let mut add = 0;
	{
		let ctx = ui::Context::new(gr, rect, mouse);


		{
			//let menubar = Rect::with_size(0, 0, width, 20);
			//let contextbar = Rect::with_size(0, 20, width, 40);
			//let timeline = Rect::with_size(0, height - 20 - 200, width, 200);
			//let statusbar = Rect::with_size(0, height - 20, width, 20);
			//
			//let palette = Rect::with_size(0, 60 + 50, 250, 500);

			let widgets = &[
				ui::Flow::with_wh(1000.0, 20.0).expand_across(),
				ui::Flow::with_wh(1000.0, 40.0).expand_across(),
				ui::Flow::with_wh(1000.0, 7.0).along_weight(1.0).expand_along().expand_across(),
				ui::Flow::with_wh(1000.0, 200.0).expand_across(),
				ui::Flow::with_wh(1000.0, 20.0).expand_across(),
			];

			/*
			pub const STATUSBAR_BG: u32 = 0x3F4350_FF;
			pub const STATUSBAR_COLOR: u32 = 0xA7A8AE_FF;
			pub const MENUBAR_BG: u32 = 0x222833_FF;
			pub const BAR_BG: u32 = 0x3f4957_FF;
			pub const TIMELINE_BG: u32 = 0x3a4351_FF;
			*/

			let colors = [
				[0x22, 0x28, 0x33, 0xFF],
				[0x3f, 0x49, 0x57, 0xFF],
				[0u8; 4],
				[0x3a, 0x43, 0x51, 0xFF],
				[0x3F, 0x43, 0x50, 0xFF],
			];
			for (i, (ctx, color)) in ctx.vertical_flow(0.0, 0.0, widgets).zip(colors.iter().cloned()).enumerate() {
				ctx.quad(color, &ctx.rect());
				if i == 4 {
					let text = format!("count: {} ms: {:}", entity_count, dt);
					ctx.label(0.0, 0.5, [0xFF; 4], &text);
				}
			}
		}

		if true {
			use math::Transform;
			let mut proj = Affine::one();
			//proj.scale(5.0, 5.0);
			proj.scale(50.0, 50.0);
			proj.translate(150.0, 100.0);

			ctx.fill([0xFF, 0xFF, 0xFF, 0xAA], proj, {
				use lyon::math::*;
				use lyon::path::*;
				use lyon::path_builder::*;

				let mut builder = SvgPathBuilder::new(Path::builder());
				if false {
					lyon::extra::rust_logo::build_logo_path(&mut builder);
				} else {
					builder.move_to(point(0.0, 0.0));
					builder.line_to(point(1.0, 0.0));
					builder.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
					builder.cubic_bezier_to(point(1.0, 1.0), point(0.0, 1.0), point(0.0, 0.0));
					builder.close();
				}
				builder.build()
			});
		}

		{
			let widgets = &[
				ui::Flow::with_wh(60.0, 40.0),
				ui::Flow::with_wh(60.0, 40.0),
				ui::Flow::with_wh(60.0, 40.0).along_weight(1.0).expand_along(),
				ui::Flow::with_wh(40.0, 40.0),
				ui::Flow::with_wh(40.0, 40.0).expand_across(),
			];

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

			static mut SLIDER_H: SliderModel = SliderModel {
				min: 1.0,
				current: 2.7,
				max: 3.0,
				vertical: false,
			};
			let slider_state_h = unsafe { &mut SLIDER_H };

			static mut SLIDER_V: SliderModel = SliderModel {
				min: 1.0,
				current: 2.7,
				max: 3.0,
				vertical: true,
			};
			let slider_state_v = unsafe { &mut SLIDER_V };

			let slider = Slider {
				progress: Progress {
					background: ColorDrawer::new([0xFF, 0xFF, 0xFF, 0xCC]),
					fill: ColorDrawer::new([0, 0, 0, 0xCC]),
				},
				normal_handle:   ColorDrawer::new([0xFF, 0, 0xFF, 0xFF]),
				hovered_handle:  ColorDrawer::new([0xFF, 0, 0xFF, 0xCC]),
				pressed_handle:  ColorDrawer::new([0xFF, 0, 0, 0xFF]),
			};

			let mut i = 0;
			for ctx in ctx.horizontal_flow(0.0, 0.0, widgets) {
				match i {
				1 => {
					toggle.behavior(&ctx, state, toggle_state);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("tgl{}", i));
				}
				2 => {
					slider.behavior(&ctx, state, slider_state_h);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("val{}: {}", i, slider_state_h.current));
				}
				4 => {
					slider.behavior(&ctx, state, slider_state_v);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("val{}: {}", i, slider_state_v.current));
				}
				_ => {
					if btn.behavior(&ctx, state, &mut ()) {
						println!("{} click", i);
					}
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("btn{}", i));
				}
				}

				i += 1;
			}

		}


		{
			let widgets = &[
				ui::Flow::with_wh(80.0, 40.0),
				ui::Flow::with_wh(80.0, 40.0),
				ui::Flow::with_wh(80.0, 40.0),
				ui::Flow::with_wh(80.0, 40.0),
			];

			let mut to_add = 1;
			for ctx in ctx.vertical_flow(0.0, 0.0, widgets) {
				if btn.behavior(&ctx, state, &mut ()) {
					add = to_add;
				}
				ctx.label(0.5, 0.5, [0xFF; 4], &format!("add {}", to_add));
				to_add *= 10;
			}
		}


	}

	add
}
