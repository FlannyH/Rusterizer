use std::cmp::Ordering;
use std::ops::Add;
use std::ops::Mul;

use glam::Mat4;
use glam::Vec3;
use glam::Vec3Swizzles;
use glam::Vec4Swizzles;

use crate::helpers::*;
use crate::mesh::Mesh;
use crate::mesh::Model;
use crate::structs::*;
use crate::texture::Texture;

pub struct Renderer {
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
}

fn lerp_bary<T: Mul<f32, Output = T> + Add<T, Output = T> + Copy>(
    bary: &Vec3,
    v0: &T,
    v1: &T,
    v2: &T,
) -> T {
    (*v0 * bary.x) + (*v1 * bary.y) + (*v2 * bary.z)
}

impl Renderer {
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
        Self::draw_line(
            v0.position.xy(),
            v1.position.xy(),
            colour_buffer,
            width,
            height,
        );
        Self::draw_line(
            v1.position.xy(),
            v2.position.xy(),
            colour_buffer,
            width,
            height,
        );
        Self::draw_line(
            v2.position.xy(),
            v0.position.xy(),
            colour_buffer,
            width,
            height,
        );
    }

    fn ndc_to_screen(v: FragIn, width: usize, height: usize) -> FragIn {
        let mut v_out = v;
        v_out.position.x = (v_out.position.x + 1.0) / 2.0 * width as f32;
        v_out.position.y = (-v_out.position.y + 1.0) / 2.0 * height as f32;
        v_out
    }

    pub fn draw_triangle_filled(
        v0: FragIn,
        v1: FragIn,
        v2: FragIn,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
        texture: Option<&Texture>,
    ) {
        let v0 = Self::ndc_to_screen(v0, width, height);
        let v1 = Self::ndc_to_screen(v1, width, height);
        let v2 = Self::ndc_to_screen(v2, width, height);
        // Get bounds of triangle
        let x_min = v0.position.x.min(v1.position.x).min(v2.position.x) as usize;
        let y_min = v0.position.y.min(v1.position.y).min(v2.position.y) as usize;
        let x_max = v0.position.x.max(v1.position.x).max(v2.position.x) as usize;
        let y_max = v0.position.y.max(v1.position.y).max(v2.position.y) as usize;
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                // Determine whether the point is on the triangle
                let coords = glam::vec2(x as f32, y as f32);
                let edge0 = edge_function(v1.position.xy(), v2.position.xy(), coords);
                let edge1 = edge_function(v2.position.xy(), v0.position.xy(), coords);
                let edge2 = edge_function(v0.position.xy(), v1.position.xy(), coords);
                let area =
                    1.0 / edge_function(v0.position.xy(), v1.position.xy(), v2.position.xy());

                //If so, interpolate the colours of the vertex
                if edge0 >= 0.0 && edge1 >= 0.0 && edge2 >= 0.0 {
                    // Depth clipping
                    let new_depth = v0.position.z / v0.position.w;
                    if new_depth > depth_buffer[x + y * width] {
                        continue;
                    }
                    depth_buffer[x + y * width] = new_depth;

                    //Get barycentric coordinates, texture coordinates, get the vertex colours, and sample the texture
                    let bary = glam::vec3(edge0 * area, edge1 * area, edge2 * area);
                    let tex_coords = lerp_bary(&bary, &v0.uv, &v1.uv, &v2.uv);
                    let normal = lerp_bary(&bary, &v0.normal, &v1.normal, &v2.normal);
                    let tangent = lerp_bary(&bary, &v0.tangent, &v1.tangent, &v2.tangent);
                    let mut colour = lerp_bary(&bary, &v0.colour, &v1.colour, &v2.colour);
                    colour = normal;
                    if let Some(tex) = texture {
                        let texture_sample = tex.argb_at_uv(tex_coords.x, tex_coords.y);
                        colour.x *= ((texture_sample) & 0xFF) as f32 / 255.0;
                        colour.y *= ((texture_sample >> 8) & 0xFF) as f32 / 255.0;
                        colour.z *= ((texture_sample >> 16) & 0xFF) as f32 / 255.0;
                    }
                    //*i = texture_sample;
                    colour_buffer[x + y * width] = colour_rgb(
                        (colour.x * 255.0) as u8,
                        (colour.y * 255.0) as u8,
                        (colour.z * 255.0) as u8,
                    );
                    //*i = colour_rgb((tex_coords.x * 255.0) as u8, (tex_coords.y * 255.0) as u8, 0);
                }
            }
        }
    }

    pub fn draw_mesh(
        &self,
        mesh: &Mesh,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
        texture: Option<&Texture>,
    ) {
        for i in 0..mesh.verts.len() / 3 {
            // Transform vertices
            let mut v0 = mesh.verts[i * 3];
            let mut v1 = mesh.verts[(i * 3) + 1];
            let mut v2 = mesh.verts[(i * 3) + 2];
            let v0pos = glam::vec4(v0.position.x, v0.position.y, v0.position.z, 1.0);
            let v1pos = glam::vec4(v1.position.x, v1.position.y, v1.position.z, 1.0);
            let v2pos = glam::vec4(v2.position.x, v2.position.y, v2.position.z, 1.0);
            let v0pos = self.view_matrix.mul_vec4(v0pos);
            let v1pos = self.view_matrix.mul_vec4(v1pos);
            let v2pos = self.view_matrix.mul_vec4(v2pos);
            let v0pos = self.projection_matrix.mul_vec4(v0pos);
            let v1pos = self.projection_matrix.mul_vec4(v1pos);
            let v2pos = self.projection_matrix.mul_vec4(v2pos);
            let v0 = FragIn {
                position: v0pos,
                normal: v0.normal,
                tangent: v0.tangent,
                colour: v0.colour,
                uv: v0.uv,
            };
            let v1 = FragIn {
                position: v1pos,
                normal: v1.normal,
                tangent: v1.tangent,
                colour: v1.colour,
                uv: v1.uv,
            };
            let v2 = FragIn {
                position: v2pos,
                normal: v2.normal,
                tangent: v2.tangent,
                colour: v2.colour,
                uv: v2.uv,
            };

            // Draw vertices
            Self::draw_triangle_filled(
                v0,
                v1,
                v2,
                colour_buffer,
                depth_buffer,
                width,
                height,
                texture,
            );
        }
    }

    pub fn draw_model(
        &self,
        model: &Model,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
        texture: Option<&Texture>,
    ) {
        for (tex_id, mesh) in &model.meshes {
            self.draw_mesh(mesh, colour_buffer, depth_buffer, width, height, texture);
        }
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
    }
    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
    }
}