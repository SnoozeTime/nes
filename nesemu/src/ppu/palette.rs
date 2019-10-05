use crate::graphic::Color;
pub const BLACK_INDEX: u8 = 0x0D;
#[derive(Debug)]
pub struct Palette {
    pub background: Color,
    pub color1: Color,
    pub color2: Color,
    pub color3: Color,
}

pub fn get_bg_color(vram: &[u8], colors: &[Color; 64]) -> Color {
    colors[vram[0x00] as usize]
}

pub fn get_color_index_bg(palette_number: u16, vram: &[u8], pixel_value: u16) -> u8 {
    if pixel_value == 0 {
        unsafe { *vram.get_unchecked(0x00) }
    } else {
        let idx = (4 * palette_number + pixel_value) as usize;
        unsafe { *vram.get_unchecked(idx) }
    }
}

pub fn get_color_index_sprite(palette_number: u8, vram: &[u8], pixel_value: u8) -> u8 {
    let idx = ((0x10 + 4 * palette_number + pixel_value) & 0b11111) as usize;
    unsafe { *vram.get_unchecked(idx) & 0b111111 }
}

pub fn get_bg_palette(
    palette_number: u16,
    vram: &[u8],
    colors: &[Color; 64],
    pixel_value: u16,
) -> Color {
    // Background color is repeated multiple time in the vram (at 0x00, 0x04, 0x08 and 0xC)
    let idx = ((4 * palette_number + pixel_value) & 0b11111) as usize;
    let vram_value = unsafe { *vram.get_unchecked(idx) & 0b111111 } as usize;
    let color = unsafe { *colors.get_unchecked(vram_value) };
    color
}

pub fn get_sprite_palette(palette_number: u8, vram: &[u8], colors: &[Color; 64]) -> Palette {
    assert!(palette_number < 4);

    let background = colors[vram[0x00] as usize];

    let idx = 0x10 + 4 * (palette_number as usize) + 1;
    let (color1, color2, color3) = (
        colors[vram[idx] as usize],
        colors[vram[idx + 1] as usize],
        colors[vram[idx + 2] as usize],
    );

    Palette {
        background,
        color1,
        color2,
        color3,
    }
}

pub fn build_default_colors() -> [Color; 64] {
    let mut colors = [Color::rgb(0, 0, 0); 64];
    colors[0x00] = Color::rgb(84, 84, 84);
    colors[0x01] = Color::rgb(0, 30, 116);
    colors[0x02] = Color::rgb(8, 16, 144);
    colors[0x03] = Color::rgb(48, 0, 136);
    colors[0x04] = Color::rgb(68, 0, 100);
    colors[0x05] = Color::rgb(92, 0, 48);
    colors[0x06] = Color::rgb(84, 4, 0);
    colors[0x07] = Color::rgb(60, 24, 0);
    colors[0x08] = Color::rgb(32, 42, 0);
    colors[0x09] = Color::rgb(8, 58, 0);
    colors[0x0A] = Color::rgb(0, 64, 0);
    colors[0x0B] = Color::rgb(0, 60, 0);
    colors[0x0C] = Color::rgb(0, 50, 60);
    colors[0x0D] = Color::rgb(0, 0, 0);
    colors[0x0E] = Color::rgb(0, 0, 0);
    colors[0x0F] = Color::rgb(0, 0, 0);
    colors[0x10] = Color::rgb(152, 150, 152);
    colors[0x11] = Color::rgb(8, 76, 196);
    colors[0x12] = Color::rgb(48, 50, 236);
    colors[0x13] = Color::rgb(92, 30, 228);
    colors[0x14] = Color::rgb(136, 20, 176);
    colors[0x15] = Color::rgb(160, 20, 100);
    colors[0x16] = Color::rgb(152, 34, 32);
    colors[0x17] = Color::rgb(120, 60, 0);
    colors[0x18] = Color::rgb(84, 90, 0);
    colors[0x19] = Color::rgb(40, 114, 0);
    colors[0x1A] = Color::rgb(8, 124, 0);
    colors[0x1B] = Color::rgb(0, 118, 40);
    colors[0x1C] = Color::rgb(0, 102, 120);
    colors[0x1D] = Color::rgb(0, 0, 0);
    colors[0x1E] = Color::rgb(0, 0, 0);
    colors[0x1F] = Color::rgb(0, 0, 0);
    colors[0x20] = Color::rgb(236, 238, 236);
    colors[0x21] = Color::rgb(76, 154, 236);
    colors[0x22] = Color::rgb(120, 124, 236);
    colors[0x23] = Color::rgb(176, 98, 236);
    colors[0x24] = Color::rgb(228, 84, 236);
    colors[0x25] = Color::rgb(236, 88, 180);
    colors[0x26] = Color::rgb(236, 106, 100);
    colors[0x27] = Color::rgb(212, 136, 32);
    colors[0x28] = Color::rgb(160, 170, 0);
    colors[0x29] = Color::rgb(116, 196, 0);
    colors[0x2A] = Color::rgb(76, 208, 32);
    colors[0x2B] = Color::rgb(56, 204, 108);
    colors[0x2C] = Color::rgb(56, 180, 204);
    colors[0x2D] = Color::rgb(60, 60, 60);
    colors[0x2E] = Color::rgb(0, 0, 0);
    colors[0x2F] = Color::rgb(0, 0, 0);
    colors[0x30] = Color::rgb(236, 238, 236);
    colors[0x31] = Color::rgb(168, 204, 236);
    colors[0x32] = Color::rgb(188, 188, 236);
    colors[0x33] = Color::rgb(212, 178, 236);
    colors[0x34] = Color::rgb(236, 174, 236);
    colors[0x35] = Color::rgb(236, 174, 212);
    colors[0x36] = Color::rgb(236, 180, 176);
    colors[0x37] = Color::rgb(228, 196, 144);
    colors[0x38] = Color::rgb(204, 210, 120);
    colors[0x39] = Color::rgb(180, 222, 120);
    colors[0x3A] = Color::rgb(168, 226, 144);
    colors[0x3B] = Color::rgb(152, 226, 180);
    colors[0x3C] = Color::rgb(160, 214, 228);
    colors[0x3D] = Color::rgb(160, 162, 160);
    colors[0x3E] = Color::rgb(0, 0, 0);
    colors[0x3F] = Color::rgb(0, 0, 0);

    colors
}
