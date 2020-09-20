/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[derive(Debug)]
pub struct DestBuffer<'a> {
    pub buffer: &'a mut [u8],
    pub width: usize,
    pub height: usize,
    pub num_channels: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceBitmap<'a> {
    pub buffer: &'a [bool],
    pub width: usize,
    pub height: usize,
}

#[derive(Debug)]
pub struct DestImage<'a> {
    pub buf: DestBuffer<'a>,
    pub padding: f64,
    pub channel: usize,
    pub x_offset: usize,
    pub y_offset: usize,
    pub width: usize,
    pub height: usize,
}

struct EdgePixels {
    edge_pixels_in: Vec<(f64, f64)>,
    edge_pixels_out: Vec<(f64, f64)>,
}

fn get_edge_pixels(source: SourceBitmap<'_>) -> EdgePixels
{
    let mut edge_pixels_in = Vec::new();
    let mut edge_pixels_out = Vec::new();
    let xlim = (source.width as usize) - 1;
    let ylim = (source.height as usize) - 1;
    let bitmap = source.buffer;
    let pitch = source.width;
    for y in 0..=ylim {
        for x in 0..=xlim {
            let index = y * pitch + x;
            let state = bitmap[index];
            // find neighbors, if we're at edge of image, outside is 'false'
            let others = [
                (y > 0 && bitmap[index - pitch]),
                (y < ylim && bitmap[index + pitch]),
                (x > 0 && bitmap[index - 1]),
                (x < xlim - 1 && bitmap[index + 1]),
            ];
            // if any of our neighbors are different than us,
            // we are on an edge; add to appropriate list
            if others.iter().any(|b| *b != state) {
                let list = if state {
                    &mut edge_pixels_in
                } else {
                    &mut edge_pixels_out
                };
                list.push((x as f64, y as f64));
            }
        }
    }
    EdgePixels { edge_pixels_in, edge_pixels_out }
}

pub fn render_sdf(dest: DestImage<'_>, source: SourceBitmap<'_>) {
    let edge_pixels = get_edge_pixels(source);
    let EdgePixels { edge_pixels_in, edge_pixels_out } = edge_pixels;
    // 'inner' = dest space size w/o padding
    let inner_width = (dest.width as f64) - (2.0 * dest.padding);
    let inner_height = (dest.height as f64) - (2.0 * dest.padding);
    let size_ratio = inner_width / (source.width as f64);
    // converts between dest space and source space
    let to_source_single = {
        let padding = dest.padding;
        move |val: usize, inner_len: f64, src_len: usize| {
            let val = (val as f64) - padding;
            let val = val / inner_len;
            val * (src_len as f64)
        }
    };
    for y in 0..dest.height {
        let src_y = (to_source_single)(y, inner_height, source.height);
        for x in 0..dest.width {
            let src_x = (to_source_single)(x, inner_width, source.width);
            let px = dest.x_offset + x;
            let py = dest.y_offset + y;
            // find the correct sub-pixel to write to
            let coord = py * dest.buf.width + px;
            let sub_coord = coord * dest.buf.num_channels + dest.channel;
            let dest_px = &mut dest.buf.buffer[sub_coord];
            // get state here, if outside the image, assume 'false'
            let state = if src_x < 0.0 || src_y < 0.0 {
                false
            } else {
                let src_x = src_x.trunc() as usize;
                let src_y = src_y.trunc() as usize;
                if src_x > source.width || src_y > source.height {
                    false
                } else {
                    source.buffer[src_y * source.width + src_x]
                }
            };
            // compare to pixels of the opposite state
            let edge_list = if state {
                &edge_pixels_out
            } else {
                &edge_pixels_in
            };
            let min_dist = edge_list.iter()
                .map(|(dx, dy)| {
                    let a2 = (src_x - dx).powi(2);
                    let b2 = (src_y - dy).powi(2);
                    a2 + b2
                })
                .min_by(|a, b| {
                    a.partial_cmp(b).expect("Somehow got a NaN distance")
                })
                .map_or(f64::INFINITY, f64::sqrt);
            // convert from distance in source units to dest units 0..255
            let scaled_dist = min_dist * size_ratio;
            let signed_dist = if state { scaled_dist } else { -scaled_dist };
            let norm_dist = signed_dist / (dest.padding as f64);
            let unorm_dist = (norm_dist + 1.0) / 2.0;
            let unorm_dist  = unorm_dist.max(0.0).min(1.0);
            *dest_px = (unorm_dist * 255.0).floor() as u8;
        }
    }
}
