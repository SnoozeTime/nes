use crate::graphic::Color;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Palette {
    pub background: Color,
    pub color1: Color,
    pub color2: Color,
    pub color3: Color,
}

pub fn get_bg_color(vram: &[u8], colors: &HashMap<u8, Color>) -> Color {
    *colors
        .get(&vram[0x00])
        .expect("Issue while fetching background color")
}

// palette number between 0 and 4 (exclusive)
// vram
// colors: Color map.
pub fn get_bg_palette(
    palette_number: u8,
    vram: &[u8],
    colors: &HashMap<u8, Color>,
) -> Option<Palette> {
    // only 4 palettes for background.
    assert!(palette_number < 4);

    let background = colors.get(&vram[0x00]);

    let (color1, color2, color3) = match palette_number {
        0 => (
            colors.get(&vram[0x01]),
            colors.get(&vram[0x2]),
            colors.get(&vram[0x03]),
        ),
        1 => (
            colors.get(&vram[0x05]),
            colors.get(&vram[0x06]),
            colors.get(&vram[0x07]),
        ),
        2 => (
            colors.get(&vram[0x09]),
            colors.get(&vram[0x0A]),
            colors.get(&vram[0x0B]),
        ),
        3 => (
            colors.get(&vram[0x0D]),
            colors.get(&vram[0x0E]),
            colors.get(&vram[0x0F]),
        ),
        _ => panic!("impossibru"),
    };

    if let (Some(bg), Some(c1), Some(c2), Some(c3)) = (background, color1, color2, color3) {
        Some(Palette {
            background: *bg,
            color1: *c1,
            color2: *c2,
            color3: *c3,
        })
    } else {
        None
    }
}

pub fn get_sprite_palette(
    palette_number: u8,
    vram: &[u8],
    colors: &HashMap<u8, Color>,
) -> Option<Palette> {
    let background = colors.get(&vram[0x00]);

    let (color1, color2, color3) = match palette_number {
        0 => (
            colors.get(&vram[0x11]),
            colors.get(&vram[0x12]),
            colors.get(&vram[0x13]),
        ),
        1 => (
            colors.get(&vram[0x15]),
            colors.get(&vram[0x16]),
            colors.get(&vram[0x17]),
        ),
        2 => (
            colors.get(&vram[0x19]),
            colors.get(&vram[0x1A]),
            colors.get(&vram[0x1B]),
        ),
        3 => (
            colors.get(&vram[0x1D]),
            colors.get(&vram[0x1E]),
            colors.get(&vram[0x1F]),
        ),
        _ => panic!("impossibru"),
    };

    if let (Some(bg), Some(c1), Some(c2), Some(c3)) = (background, color1, color2, color3) {
        Some(Palette {
            background: *bg,
            color1: *c1,
            color2: *c2,
            color3: *c3,
        })
    } else {
        // this is for debugging
        match palette_number {
            0 => {
                println!(
                    "palette 0: {:X} {:X} {:X}",
                    &vram[0x11], &vram[0x12], &vram[0x13]
                );
                println!(
                    "{:?} {:?} {:?}",
                    colors.get(&vram[0x11]),
                    colors.get(&vram[0x12]),
                    colors.get(&vram[0x13])
                );
            }
            1 => {
                println!(
                    "palette 1: {:X} {:X} {:X}",
                    &vram[0x15], &vram[0x16], &vram[0x17]
                );
                println!(
                    "{:?} {:?} {:?}",
                    colors.get(&vram[0x15]),
                    colors.get(&vram[0x16]),
                    colors.get(&vram[0x17])
                );
            }
            2 => {
                println!(
                    "palette 2: {:X} {:X} {:X}",
                    &vram[0x19], &vram[0x1A], &vram[0x1B]
                );
                println!(
                    "{:?} {:?} {:?}",
                    colors.get(&vram[0x19]),
                    colors.get(&vram[0x1A]),
                    colors.get(&vram[0x1B])
                );
            }
            3 => {
                println!(
                    "palette 3: {:X} {:X} {:X}",
                    &vram[0x1D], &vram[0x1E], &vram[0x1F]
                );
                println!(
                    "{:?} {:?} {:?}",
                    colors.get(&vram[0x1D]),
                    colors.get(&vram[0x1E]),
                    colors.get(&vram[0x1F])
                );
            }
            _ => panic!("impossibru"),
        };
        None
    }
}

// TODO load from file
//

pub fn build_default_colors() -> HashMap<u8, Color> {
    let mut colors = HashMap::new();
    colors.insert(0x00, Color::rgb(84, 84, 84));
    colors.insert(0x01, Color::rgb(0, 30, 116));
    colors.insert(0x02, Color::rgb(8, 16, 144));
    colors.insert(0x03, Color::rgb(48, 0, 136));
    colors.insert(0x04, Color::rgb(68, 0, 100));
    colors.insert(0x05, Color::rgb(92, 0, 48));
    colors.insert(0x06, Color::rgb(84, 4, 0));
    colors.insert(0x07, Color::rgb(60, 24, 0));
    colors.insert(0x08, Color::rgb(32, 42, 0));
    colors.insert(0x09, Color::rgb(8, 58, 0));
    colors.insert(0x0A, Color::rgb(0, 64, 0));
    colors.insert(0x0B, Color::rgb(0, 60, 0));
    colors.insert(0x0C, Color::rgb(0, 50, 60));
    colors.insert(0x0D, Color::rgb(0, 0, 0));
    colors.insert(0x0E, Color::rgb(0, 0, 0));
    colors.insert(0x0F, Color::rgb(0, 0, 0));
    colors.insert(0x10, Color::rgb(152, 150, 152));
    colors.insert(0x11, Color::rgb(8, 76, 196));
    colors.insert(0x12, Color::rgb(48, 50, 236));
    colors.insert(0x13, Color::rgb(92, 30, 228));
    colors.insert(0x14, Color::rgb(136, 20, 176));
    colors.insert(0x15, Color::rgb(160, 20, 100));
    colors.insert(0x16, Color::rgb(152, 34, 32));
    colors.insert(0x17, Color::rgb(120, 60, 0));
    colors.insert(0x18, Color::rgb(84, 90, 0));
    colors.insert(0x19, Color::rgb(40, 114, 0));
    colors.insert(0x1A, Color::rgb(8, 124, 0));
    colors.insert(0x1B, Color::rgb(0, 118, 40));
    colors.insert(0x1C, Color::rgb(0, 102, 120));
    colors.insert(0x1D, Color::rgb(0, 0, 0));
    colors.insert(0x1E, Color::rgb(0, 0, 0));
    colors.insert(0x1F, Color::rgb(0, 0, 0));
    colors.insert(0x20, Color::rgb(236, 238, 236));
    colors.insert(0x21, Color::rgb(76, 154, 236));
    colors.insert(0x22, Color::rgb(120, 124, 236));
    colors.insert(0x23, Color::rgb(176, 98, 236));
    colors.insert(0x24, Color::rgb(228, 84, 236));
    colors.insert(0x25, Color::rgb(236, 88, 180));
    colors.insert(0x26, Color::rgb(236, 106, 100));
    colors.insert(0x27, Color::rgb(212, 136, 32));
    colors.insert(0x28, Color::rgb(160, 170, 0));
    colors.insert(0x29, Color::rgb(116, 196, 0));
    colors.insert(0x2A, Color::rgb(76, 208, 32));
    colors.insert(0x2B, Color::rgb(56, 204, 108));
    colors.insert(0x2C, Color::rgb(56, 180, 204));
    colors.insert(0x2D, Color::rgb(60, 60, 60));
    colors.insert(0x2E, Color::rgb(0, 0, 0));
    colors.insert(0x2F, Color::rgb(0, 0, 0));
    colors.insert(0x30, Color::rgb(236, 238, 236));
    colors.insert(0x31, Color::rgb(168, 204, 236));
    colors.insert(0x32, Color::rgb(188, 188, 236));
    colors.insert(0x33, Color::rgb(212, 178, 236));
    colors.insert(0x34, Color::rgb(236, 174, 236));
    colors.insert(0x35, Color::rgb(236, 174, 212));
    colors.insert(0x36, Color::rgb(236, 180, 176));
    colors.insert(0x37, Color::rgb(228, 196, 144));
    colors.insert(0x38, Color::rgb(204, 210, 120));
    colors.insert(0x39, Color::rgb(180, 222, 120));
    colors.insert(0x3A, Color::rgb(168, 226, 144));
    colors.insert(0x3B, Color::rgb(152, 226, 180));
    colors.insert(0x3C, Color::rgb(160, 214, 228));
    colors.insert(0x3D, Color::rgb(160, 162, 160));
    colors.insert(0x3E, Color::rgb(0, 0, 0));
    colors.insert(0x3F, Color::rgb(0, 0, 0));

    colors
}
