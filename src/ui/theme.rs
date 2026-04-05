use ratatui::style::Color;

pub(super) const BRAND: Color = Color::Indexed(45);
pub(super) const WARNING: Color = Color::Indexed(220);
pub(super) const ERROR: Color = Color::Indexed(196);
pub(super) const INFO: Color = Color::Indexed(201);
pub(super) const SELECT_BG: Color = Color::Indexed(24);
pub(super) const ALT_BG: Color = Color::Indexed(236);
pub(super) const DIM_GRAY: Color = Color::Indexed(244);
pub(super) const BORDER_GRAY: Color = Color::Indexed(240);
pub(super) const HEADER_BG: Color = Color::Indexed(238);
pub(super) const EXCUSED_BG: Color = Color::Indexed(35);
pub(super) const UNEXCUSED_BG: Color = Color::Indexed(167);
pub(super) const BRIGHT_WHITE: Color = Color::Indexed(15);
pub(super) const BLACK: Color = Color::Indexed(16);
pub(super) const NEUTRAL_LIGHT: Color = Color::Indexed(250);
pub(super) const NEUTRAL_MID: Color = Color::Indexed(251);
pub(super) const LESSON_DEFAULT_BG: Color = Color::Indexed(238);
pub(super) const LESSON_DEFAULT_FOCUS_BG: Color = Color::Indexed(236);
pub(super) const LESSON_SUBSTITUTION_BG: Color = Color::Indexed(35);
pub(super) const LESSON_SUBSTITUTION_FOCUS_BG: Color = Color::Indexed(28);
pub(super) const LESSON_EXAM_BG: Color = Color::Indexed(179);
pub(super) const LESSON_EXAM_FOCUS_BG: Color = Color::Indexed(172);
pub(super) const LESSON_CANCELLED_BG: Color = Color::Indexed(167);
pub(super) const LESSON_CANCELLED_FOCUS_BG: Color = Color::Indexed(124);
pub(super) const SUBJECT_STRIPE_COLORS: [Color; 7] = [
    Color::Indexed(45),
    Color::Indexed(41),
    Color::Indexed(220),
    Color::Indexed(201),
    Color::Indexed(39),
    Color::Indexed(196),
    Color::Indexed(15),
];
