use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

const HUE_BINS: usize = 72;
const NEIGHBOUR_BINS: usize = 2;
const MIN_LIGHT_SATURATION: f32 = 0.45;
const SATURATION_BOOST: f32 = 1.15;
const THUMBNAIL_SIZE: u32 = 48;
const MIN_SATURATION: f32 = 0.25;
const MIN_VALUE: f32 = 0.2;
const MIN_COLORFUL_SHARE: f32 = 0.03;
const IMAGE_EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];

#[derive(Default)]
pub struct ColorCache {
    colors: Mutex<HashMap<String, Option<[u8; 3]>>>,
}

impl ColorCache {
    pub fn color_of(&self, path: &str) -> Option<[u8; 3]> {
        if let Some(cached) = self.colors.lock().ok().and_then(|colors| colors.get(path).copied()) {
            return cached;
        }
        let color = vivid_color(Path::new(path));
        if let Ok(mut colors) = self.colors.lock() {
            colors.insert(path.to_string(), color);
        }
        color
    }
}

pub fn vivid_color(path: &Path) -> Option<[u8; 3]> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    if !IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return None;
    }
    let image = image::open(path).ok()?.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE).to_rgb8();
    let pixels: Vec<[u8; 3]> = image.pixels().map(|pixel| pixel.0).collect();
    dominant_color(&pixels).map(light_tone)
}

pub fn hue_color(hue: f32) -> [u8; 3] {
    hsv_to_rgb(hue.rem_euclid(360.0), 1.0, 1.0)
}

fn hue_bin(hue: f32) -> usize {
    (hue / 360.0 * HUE_BINS as f32) as usize % HUE_BINS
}

fn bin_distance(a: usize, b: usize) -> usize {
    let gap = a.abs_diff(b);
    gap.min(HUE_BINS - gap)
}

pub fn dominant_color(pixels: &[[u8; 3]]) -> Option<[u8; 3]> {
    let colorful: Vec<([u8; 3], usize, f32)> = pixels
        .iter()
        .filter_map(|pixel| {
            let (hue, saturation, value) = rgb_to_hsv(*pixel);
            (saturation >= MIN_SATURATION && value >= MIN_VALUE).then(|| (*pixel, hue_bin(hue), saturation * value))
        })
        .collect();
    let mut weights = [0f32; HUE_BINS];
    for (_, bin, weight) in &colorful {
        weights[*bin] += weight;
    }
    if weights.iter().sum::<f32>() < pixels.len() as f32 * MIN_COLORFUL_SHARE {
        return None;
    }
    let around = |center: usize| {
        (0..HUE_BINS).filter(|bin| bin_distance(*bin, center) <= NEIGHBOUR_BINS).map(|bin| weights[bin]).sum::<f32>()
    };
    let best = (0..HUE_BINS).max_by(|a, b| around(*a).total_cmp(&around(*b)))?;
    let mut sums = [0f32; 3];
    let mut total = 0f32;
    for (pixel, _, weight) in colorful.iter().filter(|(_, bin, _)| bin_distance(*bin, best) <= NEIGHBOUR_BINS) {
        for (sum, channel) in sums.iter_mut().zip(pixel) {
            *sum += f32::from(*channel) * weight;
        }
        total += weight;
    }
    (total > 0.0).then(|| sums.map(|sum| (sum / total).round() as u8))
}

fn light_tone(color: [u8; 3]) -> [u8; 3] {
    let (hue, saturation, _) = rgb_to_hsv(color);
    hsv_to_rgb(hue, (saturation * SATURATION_BOOST).clamp(MIN_LIGHT_SATURATION, 1.0), 1.0)
}

fn rgb_to_hsv([red, green, blue]: [u8; 3]) -> (f32, f32, f32) {
    let (r, g, b) = (f32::from(red) / 255.0, f32::from(green) / 255.0, f32::from(blue) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation, max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> [u8; 3] {
    let chroma = value * saturation;
    let x = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match (hue / 60.0) as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let base = value - chroma;
    let channel = |part: f32| ((part + base) * 255.0).round() as u8;
    [channel(r), channel(g), channel(b)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "reads the real Steam artwork on this PC"]
    fn prints_the_color_of_each_local_game() {
        for game in crate::steam::scan(None) {
            let color = game.images.header.as_deref().filter(|path| !path.starts_with("http")).and_then(|path| vivid_color(Path::new(path)));
            println!("{:<32} {color:?}", game.name);
        }
    }

    #[test]
    fn finds_the_main_color_of_an_image() {
        let mut pixels = vec![[200, 30, 30]; 60];
        pixels.extend(vec![[20, 40, 200]; 30]);
        pixels.extend(vec![[128, 128, 128]; 200]);
        assert_eq!(dominant_color(&pixels), Some([200, 30, 30]));
    }

    #[test]
    fn keeps_the_shade_instead_of_a_pure_hue() {
        let teal = dominant_color(&vec![[40, 170, 150]; 50]).unwrap();
        let light = light_tone(teal);
        assert_eq!(light[0].max(light[1]).max(light[2]), 255);
        assert!(light[1] > light[2] && light[2] > light[0], "expected teal, got {light:?}");
        assert!(light[0] > 0, "expected a soft shade, got {light:?}");
    }

    #[test]
    fn has_no_color_for_a_grey_image() {
        assert_eq!(dominant_color(&vec![[90, 90, 90]; 100]), None);
        assert_eq!(dominant_color(&vec![[5, 0, 8]; 100]), None);
    }

    #[test]
    fn turns_a_hue_into_a_vivid_color() {
        assert_eq!(hue_color(0.0), [255, 0, 0]);
        assert_eq!(hue_color(120.0), [0, 255, 0]);
        assert_eq!(hue_color(240.0), [0, 0, 255]);
        assert_eq!(hue_color(-120.0), [0, 0, 255]);
    }

    #[test]
    fn reads_hue_saturation_and_brightness() {
        let (hue, saturation, value) = rgb_to_hsv([0, 128, 255]);
        assert!((hue - 210.0).abs() < 1.0);
        assert!((saturation - 1.0).abs() < 0.01);
        assert!((value - 1.0).abs() < 0.01);
    }
}
