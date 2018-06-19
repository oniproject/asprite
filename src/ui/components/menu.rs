use super::*;
use math::*;

use std::marker::PhantomData;

pub enum Item<'a, T: Clone + 'a> {
    Text(T, &'a str, &'a str),
    Separator,
    Menu(&'a [Item<'a, T>]),
}

pub struct ItemStyle<D: ?Sized + Graphics> {
    pub label: D::Color,
    pub shortcut: D::Color,
    pub bg: D::Color,
}

pub enum MenuEvent<T> {
    Clicked(T),
    Nothing,
    Exit,
}

pub struct Menu<D: ?Sized + Graphics, T = usize> {
    pub normal: ItemStyle<D>,
    pub hovered: ItemStyle<D>,

    pub separator: D::Color,

    pub width: f32,
    pub text_height: f32,
    pub text_inset: f32,
    pub sep_height: f32,
    pub sep_inset: f32,

    pub marker: PhantomData<T>,
}

impl<T: Clone, D: ?Sized + Graphics> Menu<D, T> {
    pub fn run<'a, 'b, 'c>(&self,
        ctx: &Context<'a, D>, state: &mut UiState,
        id: Id, base_rect: Rect<f32>, items: &'b [Item<'c, T>],
    ) -> MenuEvent<T> {
        let mut min = Point2::new(base_rect.min.x, base_rect.max.y);

        let mut any_hovering = false;

        let label_align = Vector2::new(0.0, 0.5);
        let shortcut_align = Vector2::new(1.0, 0.5);

        let mut event = None;
        for item in items.iter() {
            let rect = match item {
                &Item::Text(ref id, name, shortcut) => {
                    let rect = Rect { min, max: Point2::new(min.x + self.width, min.y + self.text_height) };
                    let style = if ctx.is_cursor_in_rect(rect) {
                        if ctx.was_released() {
                            event = Some(id.clone());
                        }
                        &self.hovered
                    } else {
                        &self.normal
                    };
                    ctx.quad(style.bg, rect);
                    let inset = rect.pad_x(self.text_inset);
                    ctx.label_rect(inset, label_align, style.label, name);
                    ctx.label_rect(inset, shortcut_align, style.shortcut, shortcut);
                    rect
                }
                &Item::Separator => {
                    let rect = Rect { min, max: Point2::new(min.x + self.width, min.y + self.sep_height) };
                    ctx.quad(self.normal.bg, rect);
                    ctx.quad(self.separator, rect.pad_y(self.sep_inset));
                    rect
                }
                &Item::Menu(_) => unimplemented!(),
            };

            min.y += rect.dy();

            any_hovering = any_hovering || ctx.is_cursor_in_rect(rect);
        }

        if let Some(item) = event {
            state.active_widget = None;
            MenuEvent::Clicked(item)
        } else if !any_hovering && !ctx.is_cursor_in_rect(base_rect) {
            state.active_widget = None;
            MenuEvent::Exit
        } else {
            state.active_widget = Some(id);
            MenuEvent::Nothing
        }
    }
}
