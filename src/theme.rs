use ratatui::style::Color; // package for ratatui

pub const BG: Color = Color::Rgb(0x1B, 0x20, 0x21); // Background Color
pub const MUTED: Color = Color::Rgb(0x51, 0x51, 0x3D); // Muted Color
pub const ACCENT: Color = Color::Rgb(0xA6, 0xA8, 0x67); // Accent Color
pub const TEXT: Color = Color::Rgb(0xE3, 0xDC, 0x95);   // Text Color

pub const WEEKS: usize = 36; // For the size of the squares in heatmap

// This is for the gradient animation
pub const LEVELS: [Color; 5] = [     Color::Rgb(0x26, 0x2C, 0x2C),
    MUTED,
    Color::Rgb(0x7C, 0x7D, 0x52),
    ACCENT,
    TEXT,
];

// Blend between two numbers, t goes from 0.0 to 1.0
pub fn mix(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

// Dont know what this does but something with animation
pub fn hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(374761393) ^ b.wrapping_mul(668265263) ^ c.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

pub const LOGO: &str = r#"

   :::     :::     :::     :::    ::: :::    :::::::::::          ::::::::   ::::::::      :::     ::::    :::
  :+:     :+:   :+: :+:   :+:    :+: :+:        :+:             :+:    :+: :+:    :+:   :+: :+:   :+:+:   :+:
 +:+     +:+  +:+   +:+  +:+    +:+ +:+        +:+             +:+        +:+         +:+   +:+  :+:+:+  +:+
+#+     +:+ +#++:++#++: +#+    +:+ +#+        +#+             +#++:++#++ +#+        +#++:++#++: +#+ +:+ +#+
+#+   +#+  +#+     +#+ +#+    +#+ +#+        +#+                    +#+ +#+        +#+     +#+ +#+  +#+#+#
#+#+#+#   #+#     #+# #+#    #+# #+#        #+#             #+#    #+# #+#    #+# #+#     #+# #+#   #+#+#
 ###     ###     ###  ########  ########## ###              ########   ########  ###     ### ###    ####

"#; // Logo for the TUI
