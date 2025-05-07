
use crate::renderers::{Renderer, Viewport};

mod mono_color_renderer;
pub use mono_color_renderer::MonoColorRenderer;

/// A renderer that does nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullRenderer;

impl Renderer for NullRenderer {
    fn set_viewport(&mut self, _viewport: Viewport) {}
    fn render(&self) {}
}

/// Defines the split point of a split renderer.
/// Use a negative value to specify a split point relative to the far edge of the viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitPoint {
    /// Absolute split point in pixels.
    Absolute(i32),

    /// Ratio of the viewport size (0.0 to 1.0).
    Ratio(f32),
}

impl SplitPoint {
    pub fn to_absolute(&self, viewport_size: i32) -> i32 {
        let mut sp = match self {
            SplitPoint::Absolute(x) => *x,
            SplitPoint::Ratio(r) => (viewport_size as f32 * r) as i32,
        };

        if sp < 0 {
            sp = viewport_size + sp
        }

        sp.clamp(0, viewport_size)
    }
}

struct SplitRenderer<R1: Renderer, R2: Renderer> {
    viewport: Viewport,
    horizontal: bool,
    split_point: SplitPoint,
    r1: R1,
    r2: R2,
}

impl<R1: Renderer, R2: Renderer> SplitRenderer<R1, R2> {
    pub fn new(horizontal: bool, split_point: SplitPoint, r1: R1, r2: R2) -> Self {
        let mut self_ = Self {
            viewport: Viewport::default(),
            horizontal,
            split_point,
            r1,
            r2,
        };

        self_.reset_subrenderer_viewports();
        self_
    }

    pub fn get_r1(&self) -> &R1 {
        &self.r1
    }

    pub fn get_r2(&self) -> &R2 {
        &self.r2
    }

    pub fn get_r1_mut(&mut self) -> &mut R1 {
        &mut self.r1
    }

    pub fn get_r2_mut(&mut self) -> &mut R2 {
        &mut self.r2
    }

    pub fn set_split_point(&mut self, split_point: SplitPoint) {
        self.split_point = split_point;
        self.reset_subrenderer_viewports();
    }

    fn reset_subrenderer_viewports(&mut self) {
        let (r1v, r2v) = if self.horizontal {
            let sp = self.split_point.to_absolute(self.viewport.size[0]);
            let r1v = Viewport {
                pos: self.viewport.pos,
                size: [sp, self.viewport.size[1]],
            };
            let r2v = Viewport {
                pos: [self.viewport.pos[0] + sp, self.viewport.pos[1]],
                size: [self.viewport.size[0] - sp, self.viewport.size[1]],
            };

            (r1v, r2v)
        } else {
            let sp = self.split_point.to_absolute(self.viewport.size[1]);
            let r1v = Viewport {
                pos: self.viewport.pos,
                size: [self.viewport.size[0], sp],
            };
            let r2v = Viewport {
                pos: [self.viewport.pos[0], self.viewport.pos[1] + sp],
                size: [self.viewport.size[0], self.viewport.size[1] - sp],
            };

            (r1v, r2v)
        };

        self.r1.set_viewport(r1v);
        self.r2.set_viewport(r2v);
    }
}

impl<R1: Renderer, R2: Renderer> Renderer for SplitRenderer<R1, R2> {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.reset_subrenderer_viewports();
    }

    fn render(&self) {
        self.r1.render();
        self.r2.render();
    }
}

/// Splits the viewport between a left and right renderer.
pub struct HSplitRenderer<Left: Renderer, Right: Renderer> {
    split_renderer: SplitRenderer<Left, Right>,
}

impl<Left: Renderer, Right: Renderer> HSplitRenderer<Left, Right> {
    pub fn new(split_point: SplitPoint, left: Left, right: Right) -> Self {
        let split_renderer = SplitRenderer::new(true, split_point, left, right);
        Self { split_renderer }
    }

    pub fn get_left(&self) -> &Left {
        self.split_renderer.get_r1()
    }

    pub fn get_right(&self) -> &Right {
        self.split_renderer.get_r2()
    }

    pub fn get_left_mut(&mut self) -> &mut Left {
        self.split_renderer.get_r1_mut()
    }

    pub fn get_right_mut(&mut self) -> &mut Right {
        self.split_renderer.get_r2_mut()
    }

    pub fn set_split_point(&mut self, split_point: SplitPoint) {
        self.split_renderer.set_split_point(split_point);
    }
}

impl<Left: Renderer, Right: Renderer> Renderer for HSplitRenderer<Left, Right> {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.split_renderer.set_viewport(viewport);
    }

    fn render(&self) {
        self.split_renderer.render();
    }
}

/// Splits the viewport between a top and bottom renderer.
pub struct VSplitRenderer<Top: Renderer, Bottom: Renderer> {
    split_renderer: SplitRenderer<Top, Bottom>,
}

impl<Top: Renderer, Bottom: Renderer> VSplitRenderer<Top, Bottom> {
    pub fn new(split_point: SplitPoint, top: Top, bottom: Bottom) -> Self {
        let split_renderer = SplitRenderer::new(false, split_point, top, bottom);
        Self { split_renderer }
    }

    pub fn get_top(&self) -> &Top {
        self.split_renderer.get_r1()
    }

    pub fn get_bottom(&self) -> &Bottom {
        self.split_renderer.get_r2()
    }

    pub fn get_top_mut(&mut self) -> &mut Top {
        self.split_renderer.get_r1_mut()
    }

    pub fn get_bottom_mut(&mut self) -> &mut Bottom {
        self.split_renderer.get_r2_mut()
    }

    pub fn set_split_point(&mut self, split_point: SplitPoint) {
        self.split_renderer.set_split_point(split_point);
    }
}

impl<Top: Renderer, Bottom: Renderer> Renderer for VSplitRenderer<Top, Bottom> {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.split_renderer.set_viewport(viewport);
    }

    fn render(&self) {
        self.split_renderer.render();
    }
}

/// Renders one renderer inside another, with a specified inset.
/// The inset is the distance from the edge of the viewport to the edge of the inner renderer.
pub struct InsetRenderer<Outer: Renderer, Inner: Renderer> {
    viewport: Viewport,
    inset: i32,
    outer: Outer,
    inner: Inner,
}

impl<Outer: Renderer, Inner: Renderer> InsetRenderer<Outer, Inner> {
    pub fn new(inset: i32, outer: Outer, inner: Inner) -> Self {
        let mut self_ = Self {
            viewport: Viewport::default(),
            inset,
            outer,
            inner,
        };

        self_.reset_subrenderer_viewports();
        self_
    }

    pub fn get_outer(&self) -> &Outer {
        &self.outer
    }

    pub fn get_inner(&self) -> &Inner {
        &self.inner
    }

    pub fn get_outer_mut(&mut self) -> &mut Outer {
        &mut self.outer
    }

    pub fn get_inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    pub fn set_inset(&mut self, inset: i32) {
        self.inset = inset;
        self.reset_subrenderer_viewports();
    }

    fn reset_subrenderer_viewports(&mut self) {
        let ix = self.viewport.pos[0] + self.inset;
        let iy = self.viewport.pos[1] + self.inset;
        let iw = self.viewport.size[0] - 2 * self.inset;
        let ih = self.viewport.size[1] - 2 * self.inset;
        let isize_ = if iw < 0 || ih < 0 {
            [0, 0]
        } else {
            [iw, ih]
        };

        let irect = Viewport {
            pos: [ix, iy],
            size: isize_,
        };

        self.outer.set_viewport(self.viewport);
        self.inner.set_viewport(irect);
    }
}

impl<Outer: Renderer, Inner: Renderer> Renderer for InsetRenderer<Outer, Inner> {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.reset_subrenderer_viewports();
    }

    fn render(&self) {
        self.outer.render();
        self.inner.render();
    }
}

pub struct FixedAspectRatioRenderer<R: Renderer> {
    viewport: Viewport,
    aspect_ratio: f32,
    renderer: R,
}

impl<R: Renderer> FixedAspectRatioRenderer<R> {
    pub fn new(aspect_ratio: f32, renderer: R) -> Self {
        let mut self_ = Self {
            viewport: Viewport::default(),
            aspect_ratio,
            renderer,
        };

        self_.reset_subrenderer_viewports();
        self_
    }

    fn reset_subrenderer_viewports(&mut self) {
        let viewport_size = self.viewport.size;
        let new_width = (viewport_size[1] as f32 * self.aspect_ratio) as i32;
        let new_height = (viewport_size[0] as f32 / self.aspect_ratio) as i32;

        let new_size = if new_width < viewport_size[0] {
            [new_width, viewport_size[1]]
        } else {
            [viewport_size[0], new_height]
        };

        let new_pos = [
            self.viewport.pos[0] + (viewport_size[0] - new_size[0]) / 2,
            self.viewport.pos[1] + (viewport_size[1] - new_size[1]) / 2,
        ];

        let new_viewport = Viewport {
            pos: new_pos,
            size: new_size,
        };

        self.renderer.set_viewport(new_viewport);
    }

    pub fn get_subrenderer(&self) -> &R {
        &self.renderer
    }

    pub fn get_subrenderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.reset_subrenderer_viewports();
    }
}

impl<R: Renderer> Renderer for FixedAspectRatioRenderer<R> {
    fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.reset_subrenderer_viewports();
    }

    fn render(&self) {
        self.renderer.render();
    }
}
