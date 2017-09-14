
//#include "stk_widget.h"
//#include "stk_image.h"
//#include "stk_label.h"

pub enum State {
	Down,
	Up,
	Highlighted,
	Focused,
}

struct Button {
	widget: Widget,
	image: image,
	state: State,
	label: Label, // XXX ptr
	flag: u32,
}

/*

STK_Button *STK_ButtonNew(Uint16 x, Uint16 y, Uint16 w, Uint16 h, char *caption);
void STK_ButtonDraw(STK_Widget *widget);
void STK_ButtonClose(STK_Widget *widget);
void STK_ButtonFilling(STK_Button *button, Uint32 pattern);
void STK_ButtonFillLabel(STK_Button *button);

int STK_ButtonRegisterType();
int STK_ButtonSetText(STK_Button *button, char *str);
*/
