use common::*;

use super::render::*;

pub trait Widget {
	//type State;
	//type Style;

	//fn style(theme: Theme) -> Style;
	//fn style(theme: Theme) -> Style;

	//fn draw<R>(&self, r: &R, state: &State, style: Style);

	fn bounds(&self) -> Rect<i16>;

	type Result;
	fn update(&mut self, e: Event) -> Self::Result;

	fn draw<R: Render>(&self, r: &R);
}

#[derive(Clone, Copy)]
pub enum State {
	Normal,
	Active,
	Hovered,
}

pub struct FrameStyle {
	pub normal: u32,
	pub hovered: u32,
	pub active: u32,
}

impl FrameStyle {
	pub fn from_state(&self, state: State) -> u32 {
		match state {
			State::Normal  => self.normal,
			State::Active  => self.active,
			State::Hovered => self.hovered,
		}
	}
}

/*
pub struct Theme {
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
	//pub frame: FrameStyle<Color>,
}
*/

pub struct Frame {
	pub r: Rect<i16>,
	pub style: FrameStyle,
	pub state: State,
}

impl Widget for Frame {
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

	fn draw<R: Render>(&self, r: &R) {
		r.rect(self.r, self.style.from_state(self.state));
	}
}

