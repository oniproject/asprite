#![allow(dead_code)]

use super::theme::*;

use render::Canvas;

use math::*;
use ui::*;

use std::mem::replace;

const INDENT: f32 = 10.0;
const LINE_HEIGHT: f32 = 20.0;

const LINE_PAD_X: f32 = 8.0;

pub struct EditorLayout<'a, 'state> {
    pub state: &'state mut UiState,
    pub ctx: Context<'a, Canvas>,
    cursor: Vector2<f32>,
    indent: usize,
    update: bool,
}

impl<'a, 'state> EditorLayout<'a, 'state> {
    pub fn new(ctx: Context<'a, Canvas>, state: &'state mut UiState) -> Self {
        Self {
            ctx: ctx.sub_range(0xFFFF),
            state,
            cursor: Vector2::zero(),
            indent: 0,
            update: false,
        }
    }

    pub fn take_update(&mut self) -> bool {
        replace(&mut self.update, false)
    }

    pub fn indent(&self) -> usize { self.indent }
    pub fn incr_indent(&mut self) {
        self.indent += 1;
        self.cursor.x = self.indent as f32 * INDENT;
    }
    pub fn decr_indent(&mut self) {
        self.indent -= 1;
        self.cursor.x = self.indent as f32 * INDENT;
    }

    pub fn reserve(&mut self, height: f32, pad_x: f32) -> Context<'a, Canvas> {
        let rect = self.ctx.rect().pad_x(pad_x);

        let min = rect.min + self.cursor;
        let max = Point2::new(rect.max.x, min.y + height);

        self.cursor.y += height;

        let rect = Rect { min, max };
        self.ctx.sub_rect(rect)
    }

    pub fn line(&mut self) -> Context<'a, Canvas> {
        self.reserve(LINE_HEIGHT, LINE_PAD_X)
    }

    pub fn skip_line(&mut self) {
        self.cursor.y += LINE_HEIGHT;
    }

    pub fn tree<F>(&mut self, label: &str, cb: F)
        where F: FnOnce(&mut Self)
    {
        self.label(label);
        self.incr_indent();
        cb(self);
        self.decr_indent();
    }

    pub fn header(&mut self, label: &str) {
        let ctx = self.line();
        ctx.quad(BAR_TITLE_BG, ctx.rect().pad_x(-LINE_PAD_X));
        ctx.label(0.0, 0.5, WHITE, label);
    }

    pub fn header_checkbox(&mut self, label: &str, v: &mut bool) -> bool {
        let ctx = self.reserve(LINE_HEIGHT, 0.0);
        ctx.quad(BAR_TITLE_BG, ctx.rect());
        let update = checkbox(ctx, &mut self.state, v, label);
        self.update |= update;
        update
    }

    pub fn label(&mut self, label: &str) {
        let ctx = self.line();
        ctx.label(0.0, 0.5, WHITE, label);
    }

    fn line_prop(&mut self, label: &str) -> Context<'a, Canvas> {
        let (label_ctx, ret) = self.line().split_x(0.3);
        label_ctx.label(0.0, 0.5, WHITE, label);
        ret
    }

    pub fn angle_slider(&mut self, label: &str, angle: &mut f32) {
        let ctx = self.line_prop(label);

        let mut slider = SliderModel {
            min: -f32::PI,
            max: f32::PI,
            // because [-pi, pi] vs [-pi, pi)
            current: if *angle > f32::PI || *angle < -f32::PI {
                angle.normalize_angle(0.0)
            } else { *angle },
        };

        let rect = ctx.rect();
        let pad = (rect.dy() - 2.0) / 2.0;
        ctx.quad(rgba(0xAAAAAA_AA), rect.pad_y(pad));

        HSLIDER.behavior(&ctx, &mut self.state, &mut slider);

        let start = *angle;
        *angle = slider.current;
        self.update |= *angle == start;
    }

    pub fn num_base<T, F>(&mut self, label: &str, sub: &str, v: &mut T, filter: F) -> bool
        where T: BaseNumExt + ToString, F: FnOnce(bool, &mut T) + Copy
    {
        let ctx = self.line_prop(label);
        let update = edit_base(ctx, &mut self.state, v, sub, filter);
        self.update |= update;
        update
    }

    pub fn checkbox(&mut self, label: &str, v: &mut bool) -> bool {
        let ctx = self.line();
        let update = checkbox(ctx, &mut self.state, v, label);
        self.update |= update;
        update
    }

    pub fn toggle_prop(&mut self, label: &str, v: &mut bool) -> bool {
        let ctx = self.line_prop(label);
        let update = checkbox_inner(ctx, &mut self.state, v, None);
        self.update |= update;
        update
    }

    pub fn num<T, A, B>(&mut self, label: &str, sub: &str, v: &mut T, scale: T, min: A, max: B) -> bool
        where T: BaseNumExt + ToString,
              A: Into<Option<T>> + Copy,
              B: Into<Option<T>> + Copy,
    {
        self.num_base(label, sub, v, |is, v| {
            if is { *v += scale } else { *v -= scale }
            if let Some(min) = min.into() {
                *v = partial_max(*v, min);
            }
            if let Some(max) = max.into() {
                *v = partial_min(*v, max);
            }
        })
    }

    pub fn vector2_base<T, F>(&mut self, label: &str, v: &mut Vector2<T>, filter: F) -> bool
        where T: BaseNumExt + ToString, F: FnOnce(bool, &mut T) + Copy
    {
        let ctx = self.line_prop(label);
        let (x, y) = ctx.split_x(0.5);
        let mut update = false;
        update |= edit_base(x, &mut self.state, &mut v.x, "X", filter);
        update |= edit_base(y, &mut self.state, &mut v.y, "Y", filter);
        self.update |= update;
        update
    }

    pub fn vector2<T, A, B>(&mut self, label: &str, v: &mut Vector2<T>, scale: T, min: A, max: B) -> bool
        where T: BaseNumExt + ToString,
              A: Into<Option<T>> + Copy,
              B: Into<Option<T>> + Copy,
    {
        self.vector2_base(label, v, |is, v| {
            if is { *v += scale } else { *v -= scale }
            if let Some(min) = min.into() {
                *v = partial_max(*v, min);
            }
            if let Some(max) = max.into() {
                *v = partial_min(*v, max);
            }
        })
    }
}

pub fn edit_base<T, F>(ctx: Context<Canvas>, state: &mut UiState, v: &mut T, label: &str, filter: F) -> bool
    where T: BaseNumExt + ToString, F: FnOnce(bool, &mut T)
{
    match edit_num(ctx, state, *v, label) {
        Some(is)  => { filter(is, v); true },
        None => false,
    }
}

pub fn edit_f<T>(ctx: Context<Canvas>, state: &mut UiState, v: &mut T, label: &str, scale: T) -> bool
    where T: BaseNumExt + ToString
{
    edit_base(ctx, state, v, label, |is, v| if is { *v += scale } else { *v -= scale })
}

pub fn checkbox_inner<T>(ctx: Context<Canvas>, state: &mut UiState, v: &mut bool, theme: T) -> bool
    where T: Into<Option<(usize, usize)>>
{
    let update = TOGGLE.behavior(&ctx, state, v);
    let theme = theme.into().unwrap_or((ICON_CHECK_ON, ICON_CHECK_OFF));
    ctx.texture(if *v { theme.0 } else { theme.1 }, ctx.rect());
    update
}

pub fn checkbox(ctx: Context<Canvas>, state: &mut UiState, v: &mut bool, label: &str) -> bool {
    let label_size = ctx.measure_text(label);
    let widgets = [
        Flow::with_wh(LINE_HEIGHT, LINE_HEIGHT),
        Flow::with_size(label_size).along_weight(1.0).expand_along().expand_across(),
    ];

    let mut iter = ctx.horizontal_flow(0.0, 0.0, &widgets);

    let icon = iter.next().unwrap();
    iter.next().unwrap().label(0.0, 0.5, WHITE, label);
    checkbox_inner(icon, state, v, None)
}

pub fn edit_num<T>(ctx: Context<Canvas>, state: &mut UiState, v: T, label: &str) -> Option<bool>
    where T: ToString
{
    let wh = ctx.rect().dy();
    let label_size = ctx.measure_text(label) + Vector2::new(wh, 0.0);
    let widgets = [
        // base label
        Flow::with_size(label_size).along_weight(1.0).expand_along().expand_across(),
        Flow::with_wh(wh, wh), // -
        Flow::with_wh(wh, wh).along_weight(1.0).expand_along().expand_across(),
        Flow::with_wh(wh, wh), // +
    ];

    let mut iter = ctx.horizontal_flow(0.0, 0.0, &widgets);

    iter.next().unwrap().label(0.5, 0.5, WHITE, label);

    let sub = &iter.next().unwrap();
    let value = &iter.next().unwrap();
    let add = &iter.next().unwrap();

    let v = v.to_string();
    value.label(0.5, 0.5, WHITE, &v.to_string());

    let mut flag = None;
    if BTN.behavior(add, state, &mut ()) {
        flag = Some(true);
    }
    if BTN.behavior(sub, state, &mut ()) {
        flag = Some(false);
    }

    add.label(0.5, 0.5, WHITE, "+");
    sub.label(0.5, 0.5, WHITE, "-");

    flag
}
