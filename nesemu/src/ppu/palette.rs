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
    colors.insert(0x00, Color::RGB(84, 84, 84));
    colors.insert(0x01, Color::RGB(0, 30, 116));
    colors.insert(0x02, Color::RGB(8, 16, 144));
    colors.insert(0x03, Color::RGB(48, 0, 136));
    colors.insert(0x04, Color::RGB(68, 0, 100));
    colors.insert(0x05, Color::RGB(92, 0, 48));
    colors.insert(0x06, Color::RGB(84, 4, 0));
    colors.insert(0x07, Color::RGB(60, 24, 0));
    colors.insert(0x08, Color::RGB(32, 42, 0));
    colors.insert(0x09, Color::RGB(8, 58, 0));
    colors.insert(0x0A, Color::RGB(0, 64, 0));
    colors.insert(0x0B, Color::RGB(0, 60, 0));
    colors.insert(0x0C, Color::RGB(0, 50, 60));
    colors.insert(0x0D, Color::RGB(0, 0, 0));
    colors.insert(0x0E, Color::RGB(0, 0, 0));
    colors.insert(0x0F, Color::RGB(0, 0, 0));
    colors.insert(0x10, Color::RGB(152, 150, 152));
    colors.insert(0x11, Color::RGB(8, 76, 196));
    colors.insert(0x12, Color::RGB(48, 50, 236));
    colors.insert(0x13, Color::RGB(92, 30, 228));
    colors.insert(0x14, Color::RGB(136, 20, 176));
    colors.insert(0x15, Color::RGB(160, 20, 100));
    colors.insert(0x16, Color::RGB(152, 34, 32));
    colors.insert(0x17, Color::RGB(120, 60, 0));
    colors.insert(0x18, Color::RGB(84, 90, 0));
    colors.insert(0x19, Color::RGB(40, 114, 0));
    colors.insert(0x1A, Color::RGB(8, 124, 0));
    colors.insert(0x1B, Color::RGB(0, 118, 40));
    colors.insert(0x1C, Color::RGB(0, 102, 120));
    colors.insert(0x1D, Color::RGB(0, 0, 0));
    colors.insert(0x1E, Color::RGB(0, 0, 0));
    colors.insert(0x1F, Color::RGB(0, 0, 0));
    colors.insert(0x20, Color::RGB(236, 238, 236));
    colors.insert(0x21, Color::RGB(76, 154, 236));
    colors.insert(0x22, Color::RGB(120, 124, 236));
    colors.insert(0x23, Color::RGB(176, 98, 236));
    colors.insert(0x24, Color::RGB(228, 84, 236));
    colors.insert(0x25, Color::RGB(236, 88, 180));
    colors.insert(0x26, Color::RGB(236, 106, 100));
    colors.insert(0x27, Color::RGB(212, 136, 32));
    colors.insert(0x28, Color::RGB(160, 170, 0));
    colors.insert(0x29, Color::RGB(116, 196, 0));
    colors.insert(0x2A, Color::RGB(76, 208, 32));
    colors.insert(0x2B, Color::RGB(56, 204, 108));
    colors.insert(0x2C, Color::RGB(56, 180, 204));
    colors.insert(0x2D, Color::RGB(60, 60, 60));
    colors.insert(0x2E, Color::RGB(0, 0, 0));
    colors.insert(0x2F, Color::RGB(0, 0, 0));
    colors.insert(0x30, Color::RGB(236, 238, 236));
    colors.insert(0x31, Color::RGB(168, 204, 236));
    colors.insert(0x32, Color::RGB(188, 188, 236));
    colors.insert(0x33, Color::RGB(212, 178, 236));
    colors.insert(0x34, Color::RGB(236, 174, 236));
    colors.insert(0x35, Color::RGB(236, 174, 212));
    colors.insert(0x36, Color::RGB(236, 180, 176));
    colors.insert(0x37, Color::RGB(228, 196, 144));
    colors.insert(0x38, Color::RGB(204, 210, 120));
    colors.insert(0x39, Color::RGB(180, 222, 120));
    colors.insert(0x3A, Color::RGB(168, 226, 144));
    colors.insert(0x3B, Color::RGB(152, 226, 180));
    colors.insert(0x3C, Color::RGB(160, 214, 228));
    colors.insert(0x3D, Color::RGB(160, 162, 160));
    colors.insert(0x3E, Color::RGB(0, 0, 0));
    colors.insert(0x3F, Color::RGB(0, 0, 0));

    colors
}
