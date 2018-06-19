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

            if ctx.is_cursor_in_rect(rect) || state.active_widget == Some(id) {
                model.open_root = Some((id, rect));
                state.active_widget = Some(id);
                ctx.set_hovered();
                ctx.quad(self.hover_bg, rect);
                ctx.label_rect(rect, align, self.hover_color, label);
            } else {
                ctx.label_rect(rect, align, self.normal_color, label);
            }
        }
    }
}
