use algo_lib::strings::utils::vec2str;

use crate::{color::Color, rect_id::RectId, Point};

#[derive(Clone)]
pub enum Op {
    CutPoint(RectId, Point),
    Color(RectId, Color),
    CutY(RectId, i32),
    CutX(RectId, i32),
    Merge(RectId, RectId),
}

impl core::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CutPoint(arg0, arg1) => f.write_fmt(format_args!(
                "cut [{}] [{}, {}]",
                vec2str(&arg0),
                arg1.x,
                arg1.y
            )),
            Self::Color(arg0, arg1) => f.write_fmt(format_args!(
                "color [{}] [{}, {}, {}, {}]",
                vec2str(&arg0),
                arg1.0[0],
                arg1.0[1],
                arg1.0[2],
                arg1.0[3]
            )),
            Self::CutY(arg0, arg1) => {
                f.write_fmt(format_args!("cut [{}] [Y] [{}]", vec2str(&arg0), arg1))
            }
            Self::CutX(arg0, arg1) => {
                f.write_fmt(format_args!("cut [{}] [X] [{}]", vec2str(&arg0), arg1))
            }
            Self::Merge(arg0, arg1) => f.write_fmt(format_args!(
                "merge [{}] [{}]",
                vec2str(&arg0),
                vec2str(&arg1)
            )),
        }
    }
}
