use super::*;
use math::*;

pub struct SliderModel {
    pub current: f32,
    pub min: f32,
    pub max: f32,
}

impl SliderModel {
    pub fn set_percent(&mut self, value: f32) {
        self.current = lerp(self.min, self.max, value);
    }
    pub fn percent(&mut self) -> f32 {
        clamp01((self.current - self.min) / (self.max - self.min))
    }
}

pub struct Slider<H> {
    pub pressed: H,
    pub hovered: H,
    pub normal: H,
    pub axis: Axis,
}

impl<'a, D, H> Component<Context<'a, D>, UiState> for Slider<H>
    where
        D: ?Sized + Graphics + 'a,
        H: FrameDrawer<D>,
{
    type Event = ();
    type Model = SliderModel;
    fn behavior(&self, ctx: &Context<D>, state: &mut UiState, model: &mut Self::Model) -> Self::Event {
        let axis = self.axis;

        let id = ctx.reserve_widget_id();
        let hovered = ctx.is_cursor_hovering();

        let rect = ctx.rect();

        let handle = if state.active_widget == Some(id) {
            ctx.set_hovered();

            let (pos, min, delta) = match axis {
                Axis::Vertical => (ctx.cursor().y, rect.min.y, rect.dy()),
                Axis::Horizontal => (ctx.cursor().x, rect.min.x, rect.dx()),
            };

            model.set_percent(clamp01((pos - min) / delta));

            if ctx.was_released() {
                state.active_widget = None;
            }
            &self.pressed
        } else if hovered && state.active_widget.is_none() {
            ctx.set_hovered();

            if ctx.was_pressed() {
                state.active_widget = Some(id);
            }
            &self.hovered
        } else {
            &self.normal
        };

        let percent = model.percent();
        handle.draw_frame(ctx.draw(), match axis {
            Axis::Vertical => {
                let delta = rect.dx() / 2.0;
                let p = Point2::new(
                    rect.min.x + delta,
                    lerp(rect.min.y + delta, rect.max.y - delta, percent),
                );
                (Rect { min: p, max: p }).pad(-delta)
            }
            Axis::Horizontal => {
                let delta = rect.dy() / 2.0;
                let p = Point2::new(
                    lerp(rect.min.x + delta, rect.max.x - delta, percent),
                    rect.min.y + delta,
                );
                (Rect { min: p, max: p }).pad(-delta)
            }
        });
    }
}
