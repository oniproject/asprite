use math::*;

use std::ops::{Generator, GeneratorState};

#[derive(Copy, Clone, Debug)]
pub struct Flow {
    pub measured_size: Vector2<f32>,

    pub along_weight: f32,

    pub expand_along: bool,
    pub shrink_along: bool,
    pub expand_across: bool,
    pub shrink_across: bool,

    pub skip: bool,
}

impl Flow {
    pub const fn with_width(w: f32) -> Self {
        Self::with_size(Vector2 {
            x: w,
            y: 0.0,
        })
    }
    pub const fn with_height(h: f32) -> Self {
        Self::with_size(Vector2 {
            x: 0.0,
            y: h,
        })
    }
    pub const fn with_wh(w: f32, h: f32) -> Self {
        Self::with_size(Vector2 {
            x: w,
            y: h,
        })
    }

    pub const fn auto(along_weight: f32) -> Self {
        Self {
            along_weight,

            measured_size: Vector2 {
                x: 0.0,
                y: 0.0,
            },
            expand_along: true,
            shrink_along: true,
            expand_across: true,
            shrink_across: true,

            skip: false,
        }
    }

    pub const fn with_size(size: Vector2<f32>) -> Self {
        Self {
            measured_size: size,
            along_weight: 0.0,
            expand_along: false,
            shrink_along: false,
            expand_across: false,
            shrink_across: false,

            skip: false,
        }
    }

    pub const fn skip(self) -> Self {
        Self {
            skip: true,
            .. self
        }
    }

    pub const fn along_weight(self, w: f32) -> Self {
        Self {
            along_weight: w,
            .. self
        }
    }
    pub const fn expand_along(self) -> Self {
        Self {
            expand_along: true,
            .. self
        }
    }
    pub const fn shrink_along(self) -> Self {
        Self {
            shrink_along: true,
            .. self
        }
    }
    pub const fn expand_across(self) -> Self {
        Self {
            expand_across: true,
            .. self
        }
    }
    pub const fn shrink_across(self) -> Self {
        Self {
            shrink_across: true,
            .. self
        }
    }
}

#[inline]
fn stretch_across(child: f32, parent: f32, expand: bool, shrink: bool) -> f32 {
    if (expand && child < parent) || (shrink && child > parent) {
        parent
    } else {
        child
    }
}

#[test]
fn gen_measure() {
    let widgets = &[
        Flow::with_wh(10.0, 5.0),
        Flow::with_wh(1.0, 15.0),
    ];

    {
        let axis = Axis::Horizontal;
        let size = measure(axis, widgets);
        assert_eq!(size, Vector2::new(11.0, 15.0));

        let mut lay = layout(axis, size, widgets);

        assert_eq!(lay.next(), Some(Rect { min: Point2::new(0.0, 0.0),  max: Point2::new(10.0, 5.0) }));
        assert_eq!(lay.next(), Some(Rect { min: Point2::new(10.0, 0.0), max: Point2::new(11.0, 15.0) }));
        assert_eq!(lay.next(), None);
    }

    {
        let axis = Axis::Vertical;
        let size = measure(axis, widgets);
        assert_eq!(size, Vector2::new(10.0, 20.0));

        let mut lay = layout(axis, size, widgets);

        assert_eq!(lay.next(), Some(Rect { min: Point2::new(0.0, 0.0), max: Point2::new(10.0, 5.0) }));
        assert_eq!(lay.next(), Some(Rect { min: Point2::new(0.0, 5.0), max: Point2::new(1.0, 20.0) }));
        assert_eq!(lay.next(), None);
    }
}

pub fn measure(axis: Axis, widgets: &[Flow]) -> Vector2<f32> {
    let i = widgets.iter().map(|c| c.measured_size);
    match axis {
    Axis::Horizontal => i.fold(Vector2::zero(), move |s, v| Vector2::new(s.x + v.x, s.y.max(v.y))),
    Axis::Vertical   => i.fold(Vector2::zero(), move |s, v| Vector2::new(s.x.max(v.x), s.y + v.y)),
    }
}

pub fn layout<'a>(axis: Axis, size: Vector2<f32>, widgets: &'a [Flow]) -> impl Iterator<Item=Rect<f32>> + 'a {
    let mut extra = match axis {
        Axis::Horizontal => size.x,
        Axis::Vertical   => size.y,
    };

    let mut total_expand_weight = 0.0;
    let mut total_shrink_weight = 0.0;

    for c in widgets.iter() {
        if c.along_weight < 0.0 {
            continue;
        }
        if c.expand_along {
            total_expand_weight += c.along_weight;
        }
        if c.shrink_along {
            total_shrink_weight += c.along_weight;
        }

        let size = c.measured_size;
        extra -= match axis {
            Axis::Horizontal => size.x,
            Axis::Vertical   => size.y,
        };
    }

    let expand = extra > 0.0 && total_expand_weight != 0.0;
    let shrink = extra < 0.0 && total_shrink_weight != 0.0;

    let mut total_weight = 0.0;
    if expand {
        total_weight = total_expand_weight;
    }
    if shrink {
        total_weight = total_shrink_weight;
    }

    let gen = move || {
        let mut p = Point2::new(0.0, 0.0);
        let mut q = Point2::new(0.0, 0.0);
        for c in widgets.into_iter() {
            match axis {
                Axis::Horizontal => p.x = q.x,
                Axis::Vertical   => p.y = q.y,
            }

            q = p + c.measured_size;

            if c.along_weight > 0.0 && (expand && c.expand_along || shrink && c.shrink_along) {
                let delta = extra * c.along_weight / total_weight;
                extra -= delta;
                total_weight -= c.along_weight;
                match axis {
                    Axis::Horizontal => {
                        q.x += delta;
                        q.x = q.x.max(p.x);
                    }
                    Axis::Vertical => {
                        q.y += delta;
                        q.y = q.y.max(p.y);
                    }
                }
            }

            match axis {
                Axis::Horizontal => q.y = stretch_across(q.y, size.y, c.expand_across, c.shrink_across),
                Axis::Vertical   => q.x = stretch_across(q.x, size.x, c.expand_across, c.shrink_across),
            }

            if !c.skip {
                yield Rect::from_min_max(p, q);
            }
        }
    };

    LayoutIter { gen }
}

struct LayoutIter<G>
    where G: Generator<Yield=Rect<f32>, Return=()>
{
    gen: G,
}

impl<G> Iterator for LayoutIter<G>
    where G: Generator<Yield=Rect<f32>, Return=()>
{
    type Item = Rect<f32>;
    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.gen.resume() } {
            GeneratorState::Yielded(rect) => Some(rect),
            GeneratorState::Complete(()) => None,
        }
    }
}
