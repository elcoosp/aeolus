//! The GPUI transform lane (architecture plan §3.7's transform row, the
//! motion lane's own shape): the seven 2D longhands the registry lowers
//! into (`--tw-translate-x/-y`, `--tw-rotate`, `--tw-scale-x/-y`,
//! `--tw-skew-x/-y`) accumulate into a [`TransformState`] and compose
//! into GPUI's OWN [`gpui::TransformationMatrix`] — the 2×2
//! rotation-scale + translation the pinned renderer carries.
//!
//! The honest boundary, verified at the `gpui-pre` 0.3.12 pin: the
//! pinned `Styled` surface carries NO transform methods and the
//! `StyleRefinement` no transform slot — the only transformed paint in
//! the crate is the svg element's `with_transformation` (and the
//! host's own animator closures, the motion lane's boundary). So the
//! lane is the HOST-FACING composition (the `animation_keyword`
//! precedent): the static emit gate keeps refusing the transform
//! families typed (no per-element slot to freeze them into), and the
//! host applies the composed matrix itself — an
//! `svg().with_transformation(...)`, a `with_animation` animator, or
//! the transformed paint surface its own element stack carries. A
//! percent translation (`translate-x-1/2`) rides the element's own
//! box, which the compose cannot know — it stays `None` (the honest
//! miss, never a guessed width).
//!
//! The composition order is the CSS individual-transform order:
//! translate outermost, then rotate, then scale, the skew folded into
//! the 2×2 (gpui's `compose` applies the right-hand matrix first, so
//! `translate.compose(rotate).compose(scale_and_skew)` reads
//! scale/skew → rotate → translate — the CSS property order).

use gpui::{Radians, TransformationMatrix, radians};

use aeolus_compile::Env;
use aeolus_ir::{Cond, Prop, Val};
use aeolus_rt::StaticProgram;

/// One walk's 2D transform state — the seven longhands' cascade
/// winners, last-write-wins exactly like every other accumulator (the
/// ring/gradient precedent: base layers first, then the fired variant
/// layers, a later `rotate-90` replaces an earlier `rotate-45`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransformState {
    translate_x: Option<Translate>,
    translate_y: Option<Translate>,
    rotate_deg: Option<f32>,
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    skew_x_deg: Option<f32>,
    skew_y_deg: Option<f32>,
}

/// A translate axis' value: the rem form (the spacing scale's own
/// unit) or the raw px an arbitrary value carries. The percent form is
/// NOT carried — it reads the element's own box, which a matrix
/// compose cannot know (the documented boundary above).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Translate {
    Rem(f32),
    Px(f32),
}

impl TransformState {
    /// Record one longhand the walk saw. The value shapes are the
    /// registry rows' own: the translate family lowers spacing
    /// multiples (`Val::Rem`) and arbitrary px (`Val::Px`); rotate and
    /// the skew family lower degrees (`Val::Num`); the scale family
    /// lowers the percent scale (`scale-95` → `Val::Num(95.0)`, the
    /// plan bridge's own `/100` reading).
    pub fn apply(&mut self, prop: Prop, val: Val) {
        match (prop, val) {
            (Prop::TranslateX, Val::Rem(n)) => self.translate_x = Some(Translate::Rem(n)),
            (Prop::TranslateX, Val::Px(n)) => self.translate_x = Some(Translate::Px(n)),
            (Prop::TranslateY, Val::Rem(n)) => self.translate_y = Some(Translate::Rem(n)),
            (Prop::TranslateY, Val::Px(n)) => self.translate_y = Some(Translate::Px(n)),
            (Prop::Rotate, Val::Num(n)) => self.rotate_deg = Some(n),
            (Prop::ScaleX, Val::Num(n)) => self.scale_x = Some(n / 100.0),
            (Prop::ScaleY, Val::Num(n)) => self.scale_y = Some(n / 100.0),
            (Prop::SkewX, Val::Num(n)) => self.skew_x_deg = Some(n),
            (Prop::SkewY, Val::Num(n)) => self.skew_y_deg = Some(n),
            _ => {}
        }
    }

    /// The composed 2D matrix, or `None` when the state carries no
    /// expressible transform (an empty state, or only percent
    /// translations — the element-box boundary above). `rem_size` is
    /// the host's rem (the window's, [`gpui::Window::rem_size`]) — the
    /// spacing translates resolve against it.
    pub fn transformation(&self, rem_size: f32) -> Option<TransformationMatrix> {
        let translate = |axis: Option<Translate>| {
            axis.map(|t| match t {
                Translate::Rem(n) => n * rem_size,
                Translate::Px(n) => n,
            })
        };
        let (tx, ty) = (translate(self.translate_x), translate(self.translate_y));
        let has_translate = tx.is_some() || ty.is_some();
        let has_rotate = self.rotate_deg.is_some();
        let has_scale = self.scale_x.is_some() || self.scale_y.is_some();
        let has_skew = self.skew_x_deg.is_some() || self.skew_y_deg.is_some();
        if !has_translate && !has_rotate && !has_scale && !has_skew {
            return None;
        }
        // The unit matrix, transformed in the CSS individual-property
        // order: scale (with the skew folded into the 2×2), then
        // rotate, then translate outermost.
        let mut matrix = TransformationMatrix::unit();
        if has_scale || has_skew {
            let (sx, sy) = (self.scale_x.unwrap_or(1.0), self.scale_y.unwrap_or(1.0));
            let (kx, ky) = (
                self.skew_x_deg.unwrap_or(0.0).to_radians().tan(),
                self.skew_y_deg.unwrap_or(0.0).to_radians().tan(),
            );
            // scale ∘ skew as one 2×2: the skew matrix
            // [[1, kx], [ky, 1]] composed with the diagonal scale.
            let skew = TransformationMatrix {
                rotation_scale: [[1.0, kx], [ky, 1.0]],
                translation: [0.0, 0.0],
            };
            let scale = TransformationMatrix {
                rotation_scale: [[sx, 0.0], [0.0, sy]],
                translation: [0.0, 0.0],
            };
            matrix = matrix.compose(skew).compose(scale);
        }
        if let Some(deg) = self.rotate_deg {
            matrix = matrix.compose(TransformationMatrix::unit().rotate(radians_from(deg)));
        }
        if has_translate {
            matrix = matrix.compose(TransformationMatrix::unit().translate(gpui::Point {
                x: gpui::ScaledPixels(tx.unwrap_or(0.0)),
                y: gpui::ScaledPixels(ty.unwrap_or(0.0)),
            }));
        }
        Some(matrix)
    }

    /// True when the state composes nothing.
    pub fn is_empty(&self) -> bool {
        self.transformation(16.0).is_none()
    }
}

/// Degrees → [`Radians`] (the pinned `radians` constructor takes the
/// radian measure directly).
fn radians_from(deg: f32) -> Radians {
    radians(deg.to_radians())
}

/// The lane's frozen read over a program (the motion lane's
/// `animation_keyword` precedent): the base + environment-matched
/// layers' transform longhands, one [`TransformState`] — the host
/// composes the matrix from it at the frame it applies the transform.
pub fn transform_state(program: &StaticProgram, env: &Env) -> TransformState {
    let mut state = TransformState::default();
    for layer in program.layers {
        if !matches!(
            layer.cond,
            Cond::Base | Cond::Dark | Cond::Bp(_) | Cond::Min(_) | Cond::Max(_)
        ) {
            // The lane reads the frame-state families the env answers
            // (the base walk's own layer set); interaction/attribute
            // layers ride the element's state closures, not a matrix.
            continue;
        }
        let fires = match layer.cond {
            Cond::Base => true,
            Cond::Dark => env.dark,
            Cond::Bp(bp) | Cond::Min(bp) => env.viewport_rem >= bp.rem(),
            Cond::Max(bp) => env.viewport_rem < bp.rem(),
            _ => unreachable!("the matches! guard above scoped the arm set"),
        };
        if !fires {
            continue;
        }
        for decl in layer.decls {
            state.apply(decl.prop, decl.val);
        }
    }
    state
}
