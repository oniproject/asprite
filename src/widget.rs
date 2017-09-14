use math::*;

pub enum Align {
	Left,
	Right,
	Center,
}

pub enum Event {
	Press,
	Release,

	MouseEnter,
	MouseLeave,
}

pub trait Widget<Color> {
	//type State;
	//type Style;

	//fn style(theme: Theme) -> Style;
	//fn style(theme: Theme) -> Style;

	//fn draw<R>(&self, r: &R, state: &State, style: Style);

	fn bounds(&self) -> Rect<i16>;

	type Result;
	fn update(&mut self, e: Event) -> Self::Result;

	fn draw<R: Render<Color=Color>>(&self, r: &R);
}


pub trait Render {
	type Color;
	//fn bounds(&self) -> Rect<i16>;

	fn pixel(&self, p: Point<i16>, color: Self::Color);
	fn line(&self, start: Point<i16>, end: Point<i16>, color: Self::Color);
	fn rect(&self, r: Rect<i16>, color: &Self::Color);
	fn outline(&self, r: Rect<i16>, color: Self::Color);
	fn icon(&self, r: Rect<i16>, index: usize);
	fn text(&mut self, r: Rect<i16>, align: Align, color: Self::Color, s: &str);

	//fn bezier(&self, vx: &[i16], vy: &[i16], s: i32, color: u32);
}

#[derive(Clone, Copy)]
pub enum State {
	Normal,
	Active,
	Hovered,
}

pub struct FrameStyle<Color> {
	pub normal: Color,
	pub hovered: Color,
	pub active: Color,
}

impl<Color> FrameStyle<Color> {
	pub fn from_state(&self, state: State) -> &Color {
		match state {
			State::Normal  => &self.normal,
			State::Active  => &self.active,
			State::Hovered => &self.hovered,
		}
	}
}

pub struct Theme {
/*
float       Alpha;                      // Global alpha applies to everything in ImGui
ImVec2      WindowPadding;              // Padding within a window
ImVec2      WindowMinSize;              // Minimum window size
float       WindowRounding;             // Radius of window corners rounding. Set to 0.0f to have rectangular windows
ImVec2      WindowTitleAlign;           // Alignment for title bar text. Defaults to (0.0f,0.5f) for left-aligned,vertically centered.
float       ChildWindowRounding;        // Radius of child window corners rounding. Set to 0.0f to have rectangular windows
ImVec2      FramePadding;               // Padding within a framed rectangle (used by most widgets)
float       FrameRounding;              // Radius of frame corners rounding. Set to 0.0f to have rectangular frame (used by most widgets).
ImVec2      ItemSpacing;                // Horizontal and vertical spacing between widgets/lines
ImVec2      ItemInnerSpacing;           // Horizontal and vertical spacing between within elements of a composed widget (e.g. a slider and its label)
ImVec2      TouchExtraPadding;          // Expand reactive bounding box for touch-based system where touch position is not accurate enough. Unfortunately we don't sort widgets so priority on overlap will always be given to the first widget. So don't grow this too much!
float       IndentSpacing;              // Horizontal indentation when e.g. entering a tree node. Generally == (FontSize + FramePadding.x*2).
float       ColumnsMinSpacing;          // Minimum horizontal spacing between two columns
float       ScrollbarSize;              // Width of the vertical scrollbar, Height of the horizontal scrollbar
float       ScrollbarRounding;          // Radius of grab corners for scrollbar
float       GrabMinSize;                // Minimum width/height of a grab box for slider/scrollbar.
float       GrabRounding;               // Radius of grabs corners rounding. Set to 0.0f to have rectangular slider grabs.
ImVec2      ButtonTextAlign;            // Alignment of button text when button is larger than text. Defaults to (0.5f,0.5f) for horizontally+vertically centered.
ImVec2      DisplayWindowPadding;       // Window positions are clamped to be visible within the display area by at least this amount. Only covers regular windows.
ImVec2      DisplaySafeAreaPadding;     // If you cannot see the edge of your screen (e.g. on a TV) increase the safe area padding. Covers popups/tooltips as well regular windows.
bool        AntiAliasedLines;           // Enable anti-aliasing on lines/borders. Disable if you are really tight on CPU/GPU.
bool        AntiAliasedShapes;          // Enable anti-aliasing on filled shapes (rounded rectangles, circles, etc.)
float       CurveTessellationTol;       // Tessellation tolerance. Decrease for highly tessellated curves (higher quality, more polygons), increase to reduce quality.
ImVec4 Colors[ImGuiCol_COUNT];
*/
	//pub frame: FrameStyle<Color>,
}

pub struct Frame<Color> {
	pub r: Rect<i16>,
	pub style: FrameStyle<Color>,
	pub state: State,
}

impl<Color> Widget<Color> for Frame<Color> {
	type Result = ();

	fn bounds(&self) -> Rect<i16> { self.r }

	fn update(&mut self, e: Event) -> Self::Result {
		self.state = match (e, self.state) {
		(Event::Press, _) => State::Active,
		(Event::Release, State::Active) => State::Hovered,

		(Event::MouseEnter, State::Normal) => State::Hovered,
		(Event::MouseLeave, _) => State::Normal,
		_ => self.state,
		}
	}

	fn draw<R: Render<Color=Color>>(&self, r: &R) {
		r.rect(self.r, self.style.from_state(self.state));
	}
}

