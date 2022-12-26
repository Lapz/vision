use std::fmt::Display;

pub struct Color {
    is_bold: bool,
    code: usize,
}

pub const Blue: Color = Color {
    code: 34,
    is_bold: false,
};

pub const Purple: Color = Color {
    code: 35,
    is_bold: false,
};

pub const Red: Color = Color {
    code: 31,
    is_bold: false,
};

pub const Yellow: Color = Color {
    code: 33,
    is_bold: false,
};

pub struct Fixed(pub usize);

impl Color {
    pub fn paint<T: Into<String> + Display>(self, msg: T) -> String {
        let prefix = if self.is_bold { "\u{001b}[1m" } else { "" };

        format!("{}\u{001b}[{}m{}\u{001b}[0m", prefix, self.code, msg)
    }

    pub fn bold(self) -> Color {
        Color {
            is_bold: true,
            code: self.code,
        }
    }
}

impl Fixed {
    pub fn bold(self) -> Color {
        Color {
            is_bold: false,
            code: self.0,
        }
    }
}
