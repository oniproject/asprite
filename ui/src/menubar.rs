use super::*;
use math::*;

pub struct MenuBar<D: ?Sized + Graphics> {
	pub normal_color: D::Color,
	pub hover_color: D::Color,
	pub hover_bg: D::Color,
}

pub struct MenuBarModel {
	pub open_root: Option<(Id, Rect<f32>)>,
}

impl<D: ?Sized + Graphics> MenuBar<D> {
	pub fn run<'a>(&self,
		ctx: &Context<'a, D>, state: &mut UiState,
		model: &mut MenuBarModel, labels: &[(Id, &str)],
	) {
		let rect = ctx.rect();
		let align = Vector2::new(0.5, 0.5);

		let mut cursor = 0.0;

		for &(id, label) in labels.iter() {
			let rect = {
				let min = Point2::new(cursor, rect.min.y);
				let size = ctx.measure_text(label);
				cursor += size.y;
				cursor += size.x;
				cursor += size.y;
				let max = Point2::new(cursor, rect.max.y);
				Rect { min, max }
			};

			if ctx.is_cursor_in_rect(&rect) || state.active_widget == Some(id) {
				model.open_root = Some((id, rect));
				state.active_widget = Some(id);
				ctx.set_hovered();
				ctx.quad(self.hover_bg, &rect);
				ctx.label_rect(rect, align, self.hover_color, label);
			} else {
				ctx.label_rect(rect, align, self.normal_color, label);
			}
		}
	}
}


pub enum Item<'a> {
	Text(&'a str, &'a str),
	Separator,
	Menu(&'a [Item<'a>]),
}

pub struct ItemStyle<D: ?Sized + Graphics> {
	pub label: D::Color,
	pub shortcut: D::Color,
	pub bg: D::Color,
}

pub struct Menu<D: ?Sized + Graphics> {
	pub normal: ItemStyle<D>,
	pub hovered: ItemStyle<D>,

	pub separator: D::Color,

	pub width: f32,
	pub text_height: f32,
	pub text_inset: f32,
	pub sep_height: f32,
	pub sep_inset: f32,
}

impl<D: ?Sized + Graphics> Menu<D> {
	pub fn run<'a, 'b, 'c>(&self,
		ctx: &Context<'a, D>, state: &mut UiState,
		id: Id, base_rect: Rect<f32>, items: &'b [Item<'c>],
	) -> bool {
		let mut min = Point2::new(base_rect.min.x, base_rect.max.y);

		let mut any_hovering = false;

		let label_align = Vector2::new(0.0, 0.5);
		let shortcut_align = Vector2::new(1.0, 0.5);

		for item in items.iter() {
			let rect = match item {
				&Item::Text(name, shortcut) => {
					let rect = Rect { min, max: Point2::new(min.x + self.width, min.y + self.text_height) };
					let style = if ctx.is_cursor_in_rect(&rect) {
						&self.hovered
					} else {
						&self.normal
					};
					ctx.quad(style.bg, &rect);
					let inset = rect.inset_x(self.text_inset);
					ctx.label_rect(inset, label_align, style.label, name);
					ctx.label_rect(inset, shortcut_align, style.shortcut, shortcut);
					rect
				}
				&Item::Separator => {
					let rect = Rect { min, max: Point2::new(min.x + self.width, min.y + self.sep_height) };
					ctx.quad(self.normal.bg, &rect);
					ctx.quad(self.separator, &rect.inset_y(self.sep_inset));
					rect
				}
				&Item::Menu(_) => unimplemented!(),
			};

			min.y += rect.dy();

			any_hovering = any_hovering || ctx.is_cursor_in_rect(&rect);
		}

		if !any_hovering && ! ctx.is_cursor_in_rect(&base_rect) {
			state.active_widget = None;
			true
		} else {
			state.active_widget = Some(id);
			false
		}
	}
}
