use std::cmp::Ordering;

use glam::Vec3Swizzles;

use crate::helpers::*;
use crate::structs::*;
use crate::texture::Texture;

pub fn draw_line(
    pos1: glam::Vec2,
    pos2: glam::Vec2,
    buffer: &mut [u32],
    width: usize,
    height: usize,
) {
    let mut x = pos1.x as i32;
    let mut y = pos1.y as i32;
    let dx = (pos2.x - pos1.x).abs() as i32;
    let dy = (pos2.y - pos1.y).abs() as i32;
    let add_x = 1 - 2 * ((pos1.x > pos2.x) as i32);
    let add_y = 1 - 2 * ((pos1.y > pos2.y) as i32);

    //Loop
    match dx.cmp(&dy) {
        Ordering::Less => {
            let mut p = 2 * dx - dy;
            for _i in 0..dy {
                y += add_y;
                if p < 0 {
                    p += 2 * (dx);
                } else {
                    p += 2 * (dx - dy);
                    x += add_x;
                }
                if x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32) {
                    buffer[coords_to_index(x as u32, y as u32, width as u32)] =
                        colour_rgb(0, 255, 0);
                }
            }
        }
        Ordering::Greater => {
            let mut p = 2 * dy - dx;
            for _i in 0..dx {
                x += add_x;
                if p < 0 {
                    p += 2 * (dy);
                } else {
                    p += 2 * (dy - dx);
                    y += add_y;
                }
                if x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32) {
                    buffer[coords_to_index(x as u32, y as u32, width as u32)] =
                        colour_rgb(255, 0, 0);
                }
            }
        }
        Ordering::Equal => {
            for _i in 0..dy {
                if x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32) {
                    buffer[coords_to_index(x as u32, y as u32, width as u32)] =
                        colour_rgb(0, 0, 255);
                }
                y += add_y;
                x += add_x;
            }
        }
    }

    if x != (pos2.x as i32) || y != (pos2.y as i32) {
        dbg!("Error! Positions are not equal");
    }
}

pub fn draw_triangle_wireframe(
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    colour_buffer: &mut Vec<u32>,
    width: usize,
    height: usize,
) {
    draw_line(
        v0.position.xy(),
        v1.position.xy(),
        colour_buffer,
        width,
        height,
    );
    draw_line(
        v1.position.xy(),
        v2.position.xy(),
        colour_buffer,
        width,
        height,
    );
    draw_line(
        v2.position.xy(),
        v0.position.xy(),
        colour_buffer,
        width,
        height,
    );
}

pub fn draw_triangle_filled(
    v0: Vertex,
    v1: Vertex,
    v2: Vertex,
    colour_buffer: &mut [u32],
    width: usize,
    texture: Option<&Texture>,
) {
    //For every pixel in on the screen
    for (count, i) in colour_buffer.iter_mut().enumerate() {
        //Determine whether the point is on the triangle
        let coords = index_to_coords(count, width);
        let edge0 = edge_function(v1.position.xy(), v2.position.xy(), coords);
        let edge1 = edge_function(v2.position.xy(), v0.position.xy(), coords);
        let edge2 = edge_function(v0.position.xy(), v1.position.xy(), coords);
        let area = 1.0 / edge_function(v0.position.xy(), v1.position.xy(), v2.position.xy());

        //If so, interpolate the colours of the vertex
        if edge0 >= 0.0 && edge1 >= 0.0 && edge2 >= 0.0 {
            //Get barycentric coordinates, texture coordinates, get the vertex colours, and sample the texture
            let bary = glam::vec3(edge0 * area, edge1 * area, edge2 * area);
            let tex_coords = bary.x * v0.uv + bary.y * v1.uv + bary.z * v2.uv;
            let mut colour = v0.colour * edge0 + v1.colour * edge1 + v2.colour * edge2;
            colour *= area;
            if let Some(tex) = texture {
                let texture_sample = tex.argb_at_uv(tex_coords.x, tex_coords.y);
                colour.x *= ((texture_sample) & 0xFF) as f32 / 255.0;
                colour.y *= ((texture_sample >> 8) & 0xFF) as f32 / 255.0;
                colour.z *= ((texture_sample >> 16) & 0xFF) as f32 / 255.0;
            }
            //*i = texture_sample;
            *i = colour_rgb(
                (colour.x * 255.0) as u8,
                (colour.y * 255.0) as u8,
                (colour.z * 255.0) as u8,
            );
            //*i = colour_rgb((tex_coords.x * 255.0) as u8, (tex_coords.y * 255.0) as u8, 0);
        }
    }
}
