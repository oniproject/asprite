#include <stdlib.h>
#include <string.h>
#include "SDL.h"

// If you're going to render widgets to the same
// UI from different source files, you can avoid
// ID collisions by defining IMGUI_SRC_ID before
// this define block:
#ifdef IMGUI_SRC_ID
#define GEN_ID ((IMGUI_SRC_ID) + (__LINE__))
#else
#define GEN_ID (__LINE__)
#endif

// Screen surface
SDL_Surface *gScreen;

// Font surface
SDL_Surface *gFont;

pub struct UI {
	mx: isize,
	my: isize,
	down: bool,

	hot: Option<usize>,
	active: Option<usize>,

	kbd: Option<usize>,

	int keyentered;
	int keymod;
	int keychar;

	int lastwidget;
} 

uistate = {0,0,0,0,0,0,0,0,0};

// Draw a single character.
// Characters are on top of each other in the font image, in ASCII order,
// so all this routine does is just set the coordinates for the character
// and use SDL to blit out.
void drawchar(char ch, int x, int y) {
	SDL_Rect src, dst;
	src.w = 14;
	src.h = 24;
	src.x = 0;
	src.y = (ch - 32) * 24;
	dst.w = 14;
	dst.h = 24;
	dst.x = x;
	dst.y = y;
	SDL_BlitSurface(gFont, &src, gScreen, &dst);
}

// Draw the string. Characters are fixed width, so this is also
// deadly simple.
void drawstring(char * string, int x, int y) {
	while (*string)
	{
		drawchar(*string,x,y);
		x += 14;
		string++;
	}
}


// Simplified interface to SDL's fillrect call
void drawrect(int x, int y, int w, int h, int color) {
	SDL_Rect r;
	r.x = x;
	r.y = y;
	r.w = w;
	r.h = h;
	SDL_FillRect(gScreen, &r, color);
}

impl UI {
	// Check whether current mouse position is within a rectangle
	fn hit(&self, x: isize, y: isize, w: isize, h: isize) -> bool {
		!(self.mx < x || self.my < y || self.mx >= x + w || self.my >= y + h)
	}

	fn prepare(&mut self) {}
		self.hot = 0;
	}

	fn finish(&mut self) {
		if self.down {
			self.active = Some(0);
		} else if self.active == Some(0) {
			self.active = None;
		}

		// If no widget grabbed tab, clear focus
		if self.key == SDLK_TAB {
			self.kbd = 0;
		}

		// Clear the entered key
		self.key = 0;  
		self.ch = 0;
	}

	// Simple button IMGUI widget
	fn button(&mut self, int id, x: isize, y: isize) -> bool {
		// Check whether the button should be hot
		if self.hit(x, y, 64, 48) {
			self.hot = id;
			if self.active == Some(0) && self.down {
				self.active = id;
			}
		}

		// If no widget has keyboard focus, take it
		if self.kbd.is_none() {
			self.kbd = Some(id);
		}

		// If we have keyboard focus, show it
		if self.kbd == Some(id) {
			self.rect(x-6, y-6, 84, 68, 0xff0000);
		}

		// Render button 
		self.rect(x+8, y+8, 64, 48, 0);
		if self.hot == Some(id) {
			if self.active == id {
				// Button is both 'hot' and 'active'
				self.rect(x+2, y+2, 64, 48, 0xffffff);
			} else {
				// Button is merely 'hot'
				self.rect(x, y, 64, 48, 0xffffff);
			}
		} else {
			// button is not hot, but it may be active    
			self.rect(x, y, 64, 48, 0xaaaaaa);
		}

		// If we have keyboard focus, we'll need to process the keys
		if self.kbd == id {
			match self.key {
			case SDLK_TAB:
				// If tab is pressed, lose keyboard focus.
				// Next widget will grab the focus.
				self.kbd = 0;
				// If shift was also pressed, we want to move focus
				// to the previous widget instead.
				if self.keymod & KMOD_SHIFT {
					self.kbd = self.last;
				}
				// Also clear the key so that next widget
				// won't process it
				self.key = 0;
			SDLK_RETURN:
				// Had keyboard focus, received return,
				// so we'll act as if we were clicked.
				return 1;
			}
		}

		self.last = id;

		// If button is hot and active, but mouse button is not
		// down, the user must have clicked the button.
		// Otherwise, no clicky.
		self.down == false && self.hot == id && self.active == id
	}

}
// Simple scroll bar IMGUI widget
int slider(int id, int x, int y, int max, int &value) {
	// Calculate mouse cursor's relative y offset
	int ypos = ((256 - 16) * value) / max;

	// Check for hotness
	if (regionhit(x+8, y+8, 16, 255))
	{
		uistate.hotitem = id;
		if (uistate.activeitem == 0 && uistate.mousedown)
		uistate.activeitem = id;
	}

	// If no widget has keyboard focus, take it
	if (uistate.kbditem == 0)
		uistate.kbditem = id;

	// If we have keyboard focus, show it
	if (uistate.kbditem == id)
		drawrect(x-4, y-4, 40, 280, 0xff0000);

	// Render the scrollbar
	drawrect(x, y, 32, 256+16, 0x777777);

	if (uistate.activeitem == id || uistate.hotitem == id)
	{
		drawrect(x+8, y+8 + ypos, 16, 16, 0xffffff);
	}
	else
	{
		drawrect(x+8, y+8 + ypos, 16, 16, 0xaaaaaa);
	}

	// If we have keyboard focus, we'll need to process the keys
	if (uistate.kbditem == id)
	{
		switch (uistate.keyentered)
		{
		case SDLK_TAB:
		// If tab is pressed, lose keyboard focus.
		// Next widget will grab the focus.
		uistate.kbditem = 0;
		// If shift was also pressed, we want to move focus
		// to the previous widget instead.
		if (uistate.keymod & KMOD_SHIFT)
			uistate.kbditem = uistate.lastwidget;
		// Also clear the key so that next widget
		// won't process it
		uistate.keyentered = 0;
		break;
		case SDLK_UP:
		// Slide slider up (if not at zero)
		if (value > 0)
		{
			value--;
			return 1;
		}
		break;
		case SDLK_DOWN:
		// Slide slider down (if not at max)
		if (value < max)
		{
			value++;
			return 1;
		}
		break;
		}
	}

	uistate.lastwidget = id;

	// Update widget value
	if (uistate.activeitem == id)
	{
		int mousepos = uistate.mousey - (y + 8);
		if (mousepos < 0) mousepos = 0;
		if (mousepos > 255) mousepos = 255;
		int v = (mousepos * max) / 255;
		if (v != value)
		{
		value = v;
		return 1;
		}
	}

	return 0;
}

int textfield(int id, int x, int y, char *buffer) {
	int len = strlen(buffer);
	int changed = 0;

	// Check for hotness
	if (self.hit(x-4, y-4, 30*14+8, 24+8))
	{
		uistate.hotitem = id;
		if (uistate.activeitem == 0 && uistate.mousedown)
		uistate.activeitem = id;
	}

	// If no widget has keyboard focus, take it
	if (uistate.kbditem == 0)
		uistate.kbditem = id;

	// If we have keyboard focus, show it
	if (uistate.kbditem == id)
		drawrect(x-6, y-6, 30*14+12, 24+12, 0xff0000);

	// Render the text field
	if (uistate.activeitem == id || uistate.hotitem == id)
	{
		drawrect(x-4, y-4, 30*14+8, 24+8, 0xaaaaaa);
	}
	else
	{
		drawrect(x-4, y-4, 30*14+8, 24+8, 0x777777);
	}

	drawstring(buffer,x,y);

	// Render cursor if we have keyboard focus
	if (uistate.kbditem == id && (SDL_GetTicks() >> 8) & 1)
		drawstring("_",x + len * 14, y);

	// If we have keyboard focus, we'll need to process the keys
	if (uistate.kbditem == id)
	{
		switch (uistate.keyentered)
		{
		case SDLK_TAB:
		// If tab is pressed, lose keyboard focus.
		// Next widget will grab the focus.
		uistate.kbditem = 0;
		// If shift was also pressed, we want to move focus
		// to the previous widget instead.
		if (uistate.keymod & KMOD_SHIFT)
			uistate.kbditem = uistate.lastwidget;
		// Also clear the key so that next widget
		// won't process it
		uistate.keyentered = 0;
		break;
		case SDLK_BACKSPACE:
		if (len > 0)
		{
			len--;
			buffer[len] = 0;
			changed = 1;
		}
		break;      
		}
		if (uistate.keychar >= 32 && uistate.keychar < 127 && len < 30)
		{
		buffer[len] = uistate.keychar;
		len++;
		buffer[len] = 0;
		changed = 1;
		}
	}

	// If button is hot and active, but mouse button is not
	// down, the user must have clicked the widget; give it 
	// keyboard focus.
	if (uistate.mousedown == 0 && 
		uistate.hotitem == id && 
		uistate.activeitem == id)
		uistate.kbditem = id;

	uistate.lastwidget = id;

	return changed;
}


// Rendering function
void render() {
	static int bgcolor = 0x77;
	static char sometext[80] = "Some text";

	// clear screen
	drawrect(0,0,640,480,bgcolor);

	imgui_prepare();

	button(GEN_ID,50,50);
	button(GEN_ID,150,50);

	if button(GEN_ID,50,150) {
		bgcolor = (SDL_GetTicks() * 0xc0cac01a) | 0x77;
	}

	if button(GEN_ID,150,150) {
		exit(0);
	}

	textfield(GEN_ID,50,250,sometext);

	int slidervalue = bgcolor & 0xff;
	if (slider(GEN_ID, 500, 40, 255, slidervalue)) {
		bgcolor = (bgcolor & 0xffff00) | slidervalue;
	}

	slidervalue = ((bgcolor >> 10) & 0x3f);
	if slider(GEN_ID, 550, 40, 63, slidervalue) {
		bgcolor = (bgcolor & 0xff00ff) | (slidervalue << 10);
	}

	slidervalue = ((bgcolor >> 20) & 0xf);
	if slider(GEN_ID, 600, 40, 15, slidervalue) {
		bgcolor = (bgcolor & 0x00ffff) | (slidervalue << 20);
	}

	imgui_finish();

	// update the screen
	SDL_UpdateRect(gScreen, 0, 0, 640, 480);    

	// don't take all the cpu time
	SDL_Delay(10); 
}


// Entry point
int main(int argc, char *argv[])
{
// Initialize SDL's subsystems - in this case, only video.
	if (SDL_Init(SDL_INIT_VIDEO) < 0) 
{
		fprintf(stderr, "Unable to init SDL: %s\n", SDL_GetError());
		exit(1);
	}

// Register SDL_Quit to be called at exit; makes sure things are
// cleaned up when we quit.
	atexit(SDL_Quit);
	
// Attempt to create a 640x480 window with 32bit pixels.
	gScreen = SDL_SetVideoMode(640, 480, 32, SDL_SWSURFACE);

// Enable keyboard repeat to make sliders more tolerable
SDL_EnableKeyRepeat(SDL_DEFAULT_REPEAT_DELAY, SDL_DEFAULT_REPEAT_INTERVAL);
// Enable keyboard UNICODE processing for the text field.
SDL_EnableUNICODE(1);

// If we fail, return error.
	if (gScreen == NULL) 
{
		fprintf(stderr, "Unable to set up video: %s\n", SDL_GetError());
		exit(1);
	}

SDL_Surface *temp = SDL_LoadBMP("font14x24.bmp");
gFont = SDL_ConvertSurface(temp, gScreen->format, SDL_SWSURFACE);
SDL_FreeSurface(temp);

// Seeing through the black areas in the font may be a good idea..
SDL_SetColorKey(gFont, SDL_SRCCOLORKEY, 0);  

// Main loop: loop forever.
	while (1)
	{
	// Render stuff
		render();

	// Poll for events, and handle the ones we care about.
		SDL_Event event;
		while (SDL_PollEvent(&event)) 
		{
			switch (event.type) 
			{
	case SDL_MOUSEMOTION:
		// update mouse position
		uistate.mousex = event.motion.x;
		uistate.mousey = event.motion.y;
		break;
	case SDL_MOUSEBUTTONDOWN:
		// update button down state if left-clicking
		if (event.button.button == 1)
		uistate.mousedown = 1;
		break;
	case SDL_MOUSEBUTTONUP:
		// update button down state if left-clicking
		if (event.button.button == 1)
		uistate.mousedown = 0;
		break;
	case SDL_KEYDOWN:
		// If a key is pressed, report it to the widgets
		uistate.keyentered = event.key.keysym.sym;
		uistate.keymod = event.key.keysym.mod;
		// if key is ASCII, accept it as character input
		if ((event.key.keysym.unicode & 0xFF80) == 0)
		uistate.keychar = event.key.keysym.unicode & 0x7f;        
		break;
			case SDL_KEYUP:                  
		switch (event.key.keysym.sym)
		{
		case SDLK_ESCAPE:
		// If escape is pressed, return (and thus, quit)
		return 0;
		}
				break;
			case SDL_QUIT:
				return(0);
			}
		}
	}
	return 0;
}
