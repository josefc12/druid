// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A radio button widget.

use crate::debug_state::DebugState;
use crate::kurbo::Circle;
use crate::widget::prelude::*;
use crate::widget::{Axis, CrossAxisAlignment, Flex, Label, LabelText};
use crate::{theme, Data, LinearGradient, UnitPoint};
use tracing::{instrument, trace};

//const DEFAULT_RADIO_RADIUS: f64 = 7.0;
//const INNER_CIRCLE_RADIUS: f64 = 2.0;
/// A group of radio buttons
#[derive(Debug, Clone)]
pub struct RadioGroup;

impl RadioGroup {
    /// Given a vector of `(label_text, enum_variant)` tuples, create a group of Radio buttons
    /// along the vertical axis.
    pub fn column<T: Data + PartialEq>(
        variants: impl IntoIterator<Item = (impl Into<LabelText<T>> + 'static, T, f64, f64)>,
    ) -> impl Widget<T> {
        RadioGroup::for_axis(Axis::Vertical, variants)
    }

    /// Given a vector of `(label_text, enum_variant)` tuples, create a group of Radio buttons
    /// along the horizontal axis.
    pub fn row<T: Data + PartialEq>(
        variants: impl IntoIterator<Item = (impl Into<LabelText<T>> + 'static, T, f64, f64)>,
    ) -> impl Widget<T> {
        RadioGroup::for_axis(Axis::Horizontal, variants)
    }

    /// Given a vector of `(label_text, enum_variant)` tuples, create a group of Radio buttons
    /// along the specified axis.
    pub fn for_axis<T: Data + PartialEq>(
        axis: Axis,
        variants: impl IntoIterator<Item = (impl Into<LabelText<T>> + 'static, T, f64, f64)>,
    ) -> impl Widget<T> {
        let mut col = Flex::for_axis(axis).cross_axis_alignment(CrossAxisAlignment::Start);
        let mut is_first = true;
        for (label, variant, size, padding) in variants.into_iter() {
            if !is_first {
                col.add_default_spacer();
            }
            let radio = Radio::new(label, variant, size, padding);
            col.add_child(radio);
            is_first = false;
        }
        col
    }
}

/// A single radio button
pub struct Radio<T> {
    variant: T,
    child_label: Label<T>,
    size: f64,
    padding: f64
}

impl<T: Data> Radio<T> {
    /// Create a lone Radio button from label text and an enum variant
    pub fn new(label: impl Into<LabelText<T>>, variant: T, size: f64, padding: f64) -> Radio<T> {
        Radio {
            variant,
            child_label: Label::new(label).with_text_size(size),
            size: size,
            padding: padding
        }
    }
}

impl<T: Data + PartialEq> Widget<T> for Radio<T> {
    #[instrument(name = "Radio", level = "trace", skip(self, ctx, event, data, _env))]
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                if !ctx.is_disabled() {
                    ctx.set_active(true);
                    ctx.request_paint();
                    trace!("Radio button {:?} pressed", ctx.widget_id());
                }
            }
            Event::MouseUp(_) => {
                if ctx.is_active() && !ctx.is_disabled() {
                    if ctx.is_hot() {
                        *data = self.variant.clone();
                    }
                    ctx.request_paint();
                    trace!("Radio button {:?} released", ctx.widget_id());
                }
                ctx.set_active(false);
            }
            _ => (),
        }
    }

    #[instrument(name = "Radio", level = "trace", skip(self, ctx, event, data, env))]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child_label.lifecycle(ctx, event, data, env);
        if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
            ctx.request_paint();
        }
    }

    #[instrument(name = "Radio", level = "trace", skip(self, ctx, old_data, data, env))]
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.child_label.update(ctx, old_data, data, env);
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

    #[instrument(name = "Radio", level = "trace", skip(self, ctx, bc, data, env))]
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("Radio");

        let label_size = self.child_label.layout(ctx, bc, data, env);
        let radio_height = self.size;//env.get(theme::BASIC_WIDGET_HEIGHT);
        let x_padding = 3.;//env.get(theme::WIDGET_CONTROL_COMPONENT_PADDING);

        let desired_size = Size::new(
            label_size.width + radio_height + x_padding,
            radio_height//radio_diam.max(label_size.height),
        );
        let size = bc.constrain(desired_size);
        trace!("Computed size: {}", size);
        size
    }

    #[instrument(name = "Radio", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let size = self.size;//env.get(theme::BASIC_WIDGET_HEIGHT);
        let x_padding = self.padding;//env.get(theme::WIDGET_CONTROL_COMPONENT_PADDING);

        let circle = Circle::new((size / 2., size / 2.), size/2.);

        // Paint the background
        let background_gradient = LinearGradient::new(
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
            (
                env.get(theme::BACKGROUND_LIGHT),
                env.get(theme::BACKGROUND_DARK),
            ),
        );

        ctx.fill(circle, &background_gradient);

        let border_color = if ctx.is_hot() && !ctx.is_disabled() {
            env.get(theme::BORDER_LIGHT)
        } else {
            env.get(theme::BORDER_DARK)
        };

        ctx.stroke(circle, &border_color, 1.);

        // Check if data enum matches our variant
        if *data == self.variant {
            let inner_circle = Circle::new((size / 2., size / 2.), (size/4.).ceil());

            let fill = if ctx.is_disabled() {
                env.get(theme::DISABLED_TEXT_COLOR)
            } else {
                env.get(theme::CURSOR_COLOR)
            };

            ctx.fill(inner_circle, &fill);
        }

        // Paint the text label
        self.child_label.draw_at(ctx, (size + x_padding, (size/(size/2.0)*-1.0+(size/(size/2.0)/2.0).ceil())));
    }

    fn debug_state(&self, data: &T) -> DebugState {
        let value_text = if *data == self.variant {
            format!("[X] {}", self.child_label.text())
        } else {
            self.child_label.text().to_string()
        };
        DebugState {
            display_name: self.short_type_name().to_string(),
            main_value: value_text,
            ..Default::default()
        }
    }
}
