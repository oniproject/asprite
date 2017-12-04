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

use editor::*;


pub struct Velocity {
	pub vel: Vector2<f32>,
}

impl Component for Velocity {
	type Storage = VecStorage<Self>;
}

pub struct Scene {
	pub textures: Vec<Texture>,
	pub queue: Arc<Queue>,
	pub area: Rect<f32>,
	pub inspector: Inspector,
	pub up: bool,
}

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
			Start => {
				if self.textures.is_empty() {
					let mut future = world.write_resource::<Future>();
					let src = Directory::new("./res");
					let options = (&mut *future, self.queue.clone());
					let iter = TextureFormat::load(&src, "res.toml", options);
					self.textures = iter.unwrap();
				};
				for _ in 0..30 {
					spawn(world, &self.textures);
				}

				let wh = *world.read_resource::<Vector2<f32>>();
				self.area = Rect {
					min: Point2::new(0.0, 0.0),
					max: Point2::new(wh.x, wh.y),
				};

				let pos = Vector2::new(300.0, 300.0);
				self.inspector.set_entity(Some(spawn_fixed(world, &self.textures, pos)));
			}
			Stop => world.delete_all(),
			Resume | Pause => (),
		}
	}

	fn update(&mut self, world: &mut World) -> SceneTransition<Event> {
		self.draw_ui(world);
		None
	}

	fn late_update(&mut self, world: &mut World) -> SceneTransition<Event> {
		world.write_resource::<Mouse>().cleanup();
		None
	}

	fn fixed_update(&mut self, world: &mut World) -> SceneTransition<Event> {
		let gravity = 0.75;

		let dt = world.read_resource::<Time>().fixed.seconds;
		let mut speed = world.write::<Velocity>();
		let mut sprites = world.write::<Local>();

		let dt = dt * 50.0;
		let between = Range::new(0.0, 10.0);

		let area = self.area;

		//use rayon::prelude::*;
		(&mut speed, &mut sprites).join().for_each(|(speed, sprite)| {
			let sprite = &mut sprite.position;
			let speed = &mut speed.vel;

			*sprite += *speed * dt;
			speed.y += gravity * dt;

			if sprite.x > area.max.x {
				speed.x *= -1.0;
				sprite.x = area.max.x;
			} else if sprite.x < area.min.x {
				speed.x *= -1.0;
				sprite.x = area.min.x;
			}

			if sprite.y > area.max.y {
				speed.y *= -0.85;
				sprite.y = area.max.y;
				let mut rng = thread_rng();
				if rng.gen() {
					speed.y -= between.ind_sample(&mut rng);
				}
			} else if sprite.y < area.min.y {
				speed.y = 0.0;
				sprite.y = area.min.y;
			}
		});
		None
	}

	fn event(&mut self, world: &mut World, event: Event) -> SceneTransition<Event> {
		match event {
			Event::WindowEvent { event, .. } => {
				use winit::WindowEvent::*;
				match event {
					Closed => return Some(state::Transition::Pop),
					MouseInput { state, button, .. } => {
						mouse_event_buttons(&mut world.write_resource::<Mouse>(), state, button);
					}
					CursorMoved { position, .. } => {
						mouse_event_movement(&mut world.write_resource::<Mouse>(), position);
					}
					KeyboardInput { input: winit::KeyboardInput { state, virtual_keycode: Some(key), .. }, .. } => {
						use winit::VirtualKeyCode as K;
						use winit::ElementState::*;
						match key {
							K::W => self.up = state == Pressed,
							_ => (),
						}
					}
					_ => (),
				}
			}
			_ => (),
		}
		None
	}
}

fn spawn_fixed(world: &mut World, textures: &[Texture], pos: Vector2<f32>) -> specs::Entity {
	let mut rng = thread_rng();
	let tex = Range::new(0, textures.len());

	let tex = tex.ind_sample(&mut rng);
	let t = &textures[tex];

	let mut sprite = Sprite::new(t.clone());
	sprite.anchor.y = 1.0;
	sprite.size.x = t.wh.0 as f32;
	sprite.size.y = t.wh.1 as f32;

	let mut local = Local::default();
	local.position = pos;

	let global = Global::default();

	sprite.recalc_pos(&global.0);

	world.create_entity()
		.with(sprite)
		.with(local)
		.with(global)
		.build()
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

impl Scene {
	fn draw_ui(&mut self, world: &mut specs::World) {
		#[cfg(feature = "profiler")] profile_scope!("ui");
		use ui::*;

		let entity_count = world.entities().join().count();
		let wh = *world.read_resource::<Vector2<f32>>();
		let mouse = *world.read_resource::<Mouse>();
		let graphics = world.read_resource::<Arc<graphics::Graphics>>().clone();
		let dt = world.read_resource::<Time>().delta.seconds;

		let area = &mut self.area;

		static mut STATE: UiState = UiState::new();
		let state = unsafe { &mut STATE };
		let last_active = state.active_widget();

		let rect = Rect::from_min_dim(Point2::new(0.0, 0.0), wh);

		let ctx = Context::new(&*graphics, rect, mouse);

		let widgets = [
			Flow::with_height(MENUBAR_HEIGHT).expand_across(),
			Flow::with_height(TOOLBAR_HEIGHT).expand_across(),
			Flow::auto(1.0),
			Flow::with_height(200.0).expand_across(),
			Flow::with_height(STATUSBAR_HEIGHT).expand_across(),
		];

		let colors = [
			MENUBAR_BG,
			TOOLBAR_BG,
			[0u8; 4],
			[0x3a, 0x43, 0x51, 0xFF],
			STATUSBAR_BG,
		];

		let mut iter = ctx.vertical_flow(0.0, 0.0, &widgets)
			.zip(colors.iter().cloned())
			.map(|(ctx, color)| {
				ctx.quad(color, &ctx.rect());
				ctx
			});

		if let Some(ctx) = iter.next() {
			menubar(&ctx, state);
		}

		if let Some(ctx) = iter.next() {
			let mut add = 0;
			toolbar(&ctx, state, &mut add);
			for _ in 0..add {
				spawn(world, &self.textures);
			}
		}

		if let Some(ctx) = iter.next() {
			*area = ctx.rect();
			let hvr = Rect {
				min: Point2::new(0.0, 200.0),
				max: Point2::new(300.0, 400.0),
			};
			let id = ctx.reserve_widget_id();
			ctx.onhover(id, hvr, state,
				|| println!("hover start {:?} {:?}", id, hvr),
				|| println!("hover end   {:?} {:?}", id, hvr),
			);

			let clk = Rect {
				min: Point2::new(0.0, 400.0),
				max: Point2::new(300.0, 600.0),
			};

			let id = ctx.reserve_widget_id();
			ctx.onclick(id, clk, state, || println!("click {:?} {:?}", id, clk));

			ctx.quad([0x99; 4], &hvr);
			ctx.quad([0xAA; 4], &clk);

			let (_, insp) = ctx.split_x(0.85);
			self.inspector.run(&insp, state, world);
		}

		if let Some(ctx) = iter.next() {
			xbar(&ctx, state);
		}

		if let Some(ctx) = iter.next() {
			let text = format!("count: {} last: {:?} ms: {:}", entity_count, last_active, dt);
			ctx.label(0.0, 0.5, [0xFF; 4], &text);
		}

		if true {
			let widgets = &[
				Flow::with_wh(60.0, 40.0),
				Flow::with_wh(60.0, 40.0),
				Flow::with_wh(60.0, 40.0).along_weight(1.0).expand_along(),
				Flow::with_wh(40.0, 40.0),
				Flow::with_wh(40.0, 40.0).expand_across(),
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

			static mut SLIDER_H: SliderModel = SliderModel {
				min: 1.0,
				current: 2.7,
				max: 3.0,
			};

			static mut SLIDER_V: SliderModel = SliderModel {
				min: 1.0,
				current: 2.7,
				max: 3.0,
			};

			let toggle_state = unsafe { &mut TOGGLE_STATE };
			let slider_state_h = unsafe { &mut SLIDER_H };
			let slider_state_v = unsafe { &mut SLIDER_V };

			let mut i = 0;
			for ctx in ctx.horizontal_flow(0.0, 0.0, widgets) {
				match i {
				1 => {
					TOGGLE.behavior(&ctx, state, toggle_state);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("tgl{}", i));
				}
				2 => {
					HSLIDER.behavior(&ctx, state, slider_state_h);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("val{}: {}", i, slider_state_h.current));
				}
				4 => {
					VSLIDER.behavior(&ctx, state, slider_state_v);
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("val{}: {}", i, slider_state_v.current));
				}
				_ => {
					if BTN.behavior(&ctx, state, &mut ()) {
						println!("{} click", i);
					}
					ctx.label(0.5, 0.5, [0xFF; 4], &format!("btn{}", i));
				}
				}

				i += 1;
			}
		}


		second_menubar(&ctx, state);
	}
}


static mut MENUBAR_MODEL: ui::menubar::MenuBarModel = ui::menubar::MenuBarModel {
	open_root: None,
};

fn menubar(ctx: &ui::Context<graphics::Graphics>, state: &mut ui::UiState) {
	MENUBAR.run(&ctx, state, unsafe { &mut MENUBAR_MODEL }, &[
		(ctx.reserve_widget_id(), "File"),
		(ctx.reserve_widget_id(), "Edit"),
		(ctx.reserve_widget_id(), "View"),
		(ctx.reserve_widget_id(), "Tools"),
		(ctx.reserve_widget_id(), "Help"),
	]);
}

fn toolbar(ctx: &ui::Context<graphics::Graphics>, state: &mut ui::UiState, add: &mut usize) {
	use ui::*;
	let widgets = &[
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
		Flow::with_wh(80.0, 40.0),
	];
	let mut to_add = 1;
	for ctx in ctx.horizontal_flow(0.2, 0.0, widgets) {
		if BTN.behavior(&ctx, state, &mut ()) {
			*add = to_add;
		}
		ctx.label(0.5, 0.5, [0xFF; 4], &format!("add {}", to_add));
		to_add *= 10;
	}
}

fn xbar(ctx: &ui::Context<graphics::Graphics>, _state: &mut ui::UiState) {
	use ui::*;
	use math::*;
	use graphics::CustomCmd;

	let pos = ctx.rect().min;

	let mut proj = Affine::one();
	//proj.scale(5.0, 5.0);
	proj.scale(50.0, 50.0);
	proj.translate(pos.x + 150.0, pos.y + 30.0);

	ctx.custom(CustomCmd::Fill([0xFF, 0xFF, 0xFF, 0xAA], proj, {
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
	}));
}

fn second_menubar(ctx: &ui::Context<graphics::Graphics>, state: &mut ui::UiState) {
	use ui::menubar::*;
	use ui::menubar::MenuEvent::*;
	let items = [
		Item::Text(Command::New, "New", "Ctrl-N"),
		Item::Text(Command::Open, "Open", "Ctrl-O"),
		Item::Text(Command::Recent, "Recent", ">"),
		Item::Separator,
		Item::Text(Command::Save, "Save", "Ctrl-S"),
		Item::Text(Command::SaveAs, "Save as...", "Shift-Ctrl-S"),
		Item::Separator,
		Item::Text(Command::Quit, "Quit", "Ctrl-Q"),
	];

	let menubar = unsafe { &mut MENUBAR_MODEL };
	if let Some((id, base_rect)) = menubar.open_root {
		let exit = match MENU.run(&ctx, state, id, base_rect, &items) {
			Nothing => false,
			Exit => true,
			Clicked(id) => {
				println!("click: {:?}", id);
				true
			}
		};
		if exit {
			unsafe { MENUBAR_MODEL.open_root = None; }
		}
	}
}

/*
enum ViewMode {
	Icons,
	List,
	Media,
	Tree,
}
*/


/*
fn refiler() {
	use ui::*;
	const TOOL: f32 = 20.0;

	let toolbar_width = 250.0;

	// tabbar
	{
		let sep = Flow::with_wh(2.0, 0.0);
		let btn = Flow::with_wh(18.0, 18.0);

		let toolbar_up = &[
			sep,
			// prev next
			btn, sep, btn, // prev
		];

		let widgets_right = &[
			// for toolbar_up
			Flow::with_wh(toolbar_width, 0.0),

			sep,

			// view mode
			btn, btn, btn, btn,

			sep, btn, // arrange
			sep, btn, // action
			sep, btn, // share
			sep, btn, // edit tags

			sep.along_weight(1.0).expand_along(),

			// tools
			btn, sep,
			// search bar
			sep.along_weight(1.0).expand_along(),
		];
	}
}
*/
