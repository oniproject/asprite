pub trait Widget {
    /*
    type State = ();
    type Element = ();
    //type Event = ();
    //type Context = ();

    fn state(&self) -> Self::State;
    //fn update(&self, ctx: Self::Context, state: &mut Self::State, event: Self::Event) {}
    fn element(&self) -> Self::Element;
    */
}

/*
pub enum Msg {
    Increment,
    Decrement,
}

pub struct Model {
    counter: isize,
}

pub struct Counter;

impl Widget for Counter {
    type State = Model;
    type Event = Msg;
    type Context = ();

    fn state(&self) -> Self::State {
        Model { counter: 0 }
    }

    fn update(&self, _ctx: Self::Context, state: &mut Self::State, event: Self::Event) {
        match event {
            Msg::Increment => state.counter += 1,
            Msg::Decrement => state.counter -= 1,
        }
    }
}
*/

pub struct Size {
    pub w: f32,
    pub h: f32,
}

struct Owner;
struct RenderObject;

struct Element {
    depth: usize,
    dirty: bool,
    owner: Box<Owner>,
    render: Box<RenderObject>,
    size: Size,
    slot: usize,
    widget: Box<Widget>,
}
