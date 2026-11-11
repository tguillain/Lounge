use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use image::{GenericImageView, RgbaImage};

const SAMPLE_SIZE: u32 = 64;
const OPAQUE_ALPHA: u8 = 128;
const VISIBLE_ALPHA: u8 = 16;
const DARK_LUMINANCE: f32 = 0.3;
const MAX_WIDTH: u32 = 1040;
const MAX_HEIGHT: u32 = 400;

pub struct PreparedLogo {
    pub path: PathBuf,
    pub is_dark: bool,
}

pub fn prepare(source: &Path, cache_dir: &Path, key: &str) -> Option<PreparedLogo> {
    let dark_path = cache_dir.join(format!("{key}-dark.png"));
    let light_path = cache_dir.join(format!("{key}-light.png"));
    let source_modified = modified(source)?;
    for (path, is_dark) in [(&dark_path, true), (&light_path, false)] {
        if modified(path).is_some_and(|cached| cached >= source_modified) {
            return Some(PreparedLogo { path: path.clone(), is_dark });
        }
    }

    let image = image::open(source).ok()?.to_rgba8();
    let (x, y, width, height) = opaque_bounds(&image)?;
    let trimmed = image::DynamicImage::ImageRgba8(image.view(x, y, width, height).to_image());
    let resized = if width > MAX_WIDTH || height > MAX_HEIGHT {
        trimmed.thumbnail(MAX_WIDTH, MAX_HEIGHT)
    } else {
        trimmed
    };
    let sample = resized.thumbnail(SAMPLE_SIZE, SAMPLE_SIZE).to_rgba8();
    let is_dark = mostly_dark(sample.pixels().map(|pixel| pixel.0));

    let (target, stale) = if is_dark { (dark_path, light_path) } else { (light_path, dark_path) };
    let _ = fs::remove_file(stale);
    resized.save(&target).ok()?;
    Some(PreparedLogo { path: target, is_dark })
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

fn opaque_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut left, mut top, mut right, mut bottom) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] > VISIBLE_ALPHA {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
    }
    (left <= right && top <= bottom).then(|| (left, top, right - left + 1, bottom - top + 1))
}

fn mostly_dark(pixels: impl Iterator<Item = [u8; 4]>) -> bool {
    let (mut opaque, mut dark) = (0u32, 0u32);
    for [red, green, blue, alpha] in pixels {
        if alpha < OPAQUE_ALPHA {
            continue;
        }
        opaque += 1;
        let luminance = (0.2126 * f32::from(red) + 0.7152 * f32::from(green) + 0.0722 * f32::from(blue)) / 255.0;
        if luminance < DARK_LUMINANCE {
            dark += 1;
        }
    }
    opaque > 0 && dark * 2 > opaque
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    const NAVY: [u8; 4] = [20, 24, 60, 255];
    const WHITE: [u8; 4] = [250, 250, 250, 255];
    const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

    #[test]
    fn flags_a_mostly_navy_logo() {
        let pixels = [NAVY, NAVY, NAVY, WHITE, TRANSPARENT, TRANSPARENT];
        assert!(mostly_dark(pixels.into_iter()));
    }

    #[test]
    fn keeps_a_light_logo_as_is() {
        let pixels = [WHITE, WHITE, NAVY, TRANSPARENT];
        assert!(!mostly_dark(pixels.into_iter()));
    }

    #[test]
    fn ignores_transparent_background() {
        let pixels = [TRANSPARENT, TRANSPARENT, TRANSPARENT, WHITE];
        assert!(!mostly_dark(pixels.into_iter()));
    }

    #[test]
    fn finds_the_visible_area_inside_transparent_padding() {
        let mut image = RgbaImage::from_pixel(10, 8, Rgba(TRANSPARENT));
        image.put_pixel(2, 3, Rgba(WHITE));
        image.put_pixel(6, 5, Rgba(WHITE));
        assert_eq!(opaque_bounds(&image), Some((2, 3, 5, 3)));
    }

    #[test]
    fn reports_nothing_for_an_empty_logo() {
        let image = RgbaImage::from_pixel(4, 4, Rgba(TRANSPARENT));
        assert_eq!(opaque_bounds(&image), None);
    }
}
