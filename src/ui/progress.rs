use super::*;
use math::*;

#[derive(Clone, Copy)]
pub struct Progress<BG, F> {
    pub background: BG,
    pub fill: F,
    pub axis: Axis,
}

impl<'a, D, BG, F> Component<Context<'a, D>, UiState> for Progress<BG, F>
    where
        D: ?Sized + Graphics + 'a,
        BG: FrameDrawer<D>,
        F: FrameDrawer<D>,
{
    type Event = ();
    type Model = f32;
    fn behavior(&self, ctx: &Context<D>, _state: &mut UiState, model: &mut Self::Model) -> Self::Event {
        let rect = ctx.rect();
        self.background.draw_frame(ctx.draw(), rect);

        let rect = match self.axis {
            Axis::Horizontal => {
                let w = rect.dx() * *model;
                Rect { max: Point2::new(rect.min.x + w, rect.max.y), .. rect }
            }
            Axis::Vertical => {
                let h = rect.dy() * *model;
                Rect { max: Point2::new(rect.max.x, rect.min.y + h), .. rect }
            }
        };
        self.fill.draw_frame(ctx.draw(), rect);
    }
}
