use super::*;

    //fn behavior(&self, ctx: &Context<D>, state: &mut UiState, checked: &mut Self::Model) -> Self::Event {

pub trait Window<D: ?Sized + Graphics> {
    fn header(&self, &Context<D>, state: &mut UiState);
    fn body(&self, &Context<D>, state: &mut UiState);
}

pub type WindowId = u16;

pub struct StackingWM {
    pub windows: Vec<(WindowId, Rect<f32>)>,
}

impl StackingWM {
    pub fn run() {}
}

pub fn run_floating() -> impl Iterator<Item=> {
}

pub struct Window<'a, D: ?Sized + Graphics> {
    pub ctx: Context<'a, D>,
    pub title: String,
}

impl Window {
    pub fn run<F>(&self, state: &mut UiState, f: F)
        where F: FnMut(Context<'a, D>, state: &mut UiState)
    {
        f(ctx, state);
    }
}
