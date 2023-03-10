use std::cmp::Ordering;
use std::collections::HashMap;
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
use crate::texture::Material;

pub struct Renderer {
    pub projection_matrix: Mat4,
    pub view_matrix: Mat4,
    pub materials: HashMap<String, Material>,
}

fn lerp_bary<T: Mul<f32, Output = T> + Add<T, Output = T> + Copy>(
    bary: &Vec3,
    v0: &T,
    v1: &T,
    v2: &T,
    correction: Option<f32>,
) -> T {
    ((*v0 * bary.x) + (*v1 * bary.y) + (*v2 * bary.z)) * correction.unwrap_or(1.0)
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
                        buffer[coords_to_index(x as usize, y as usize, width)] =
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
                        buffer[coords_to_index(x as usize, y as usize, width)] =
                            colour_rgb(255, 0, 0);
                    }
                }
            }
            Ordering::Equal => {
                for _i in 0..dy {
                    if x >= 0 && x < (width as i32) && y >= 0 && y < (height as i32) {
                        buffer[coords_to_index(x as usize, y as usize, width)] =
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
        colour_buffer: &mut [u32],
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

    // From [-1.0, +1.0] -> [0, screen_width or screen_height]
    fn ndc_to_screen(v: FragIn, width: usize, height: usize) -> FragIn {
        let mut v_out = v;
        v_out.position.x = (v_out.position.x + 1.0) / 2.0 * width as f32;
        v_out.position.y = (-v_out.position.y + 1.0) / 2.0 * height as f32;
        v_out
    }

    pub fn draw_triangle_filled(
        v0_in: FragIn,
        v1_in: FragIn,
        v2_in: FragIn,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
        material: Option<&Material>,
    ) {
        // Get mutable copies of vertices
        let mut v0 = v0_in;
        let mut v1 = v1_in;
        let mut v2 = v2_in;

        // Get reciprocals
        let rec0 = 1.0 / v0.position.w;
        let rec1 = 1.0 / v1.position.w;
        let rec2 = 1.0 / v2.position.w;

        // Perspective division on all attributes
        v0.normal *= rec0;
        v0.tangent *= rec0;
        v0.colour *= rec0;
        v0.uv *= rec0;
        v1.normal *= rec1;
        v1.tangent *= rec1;
        v1.colour *= rec1;
        v1.uv *= rec1;
        v2.normal *= rec2;
        v2.tangent *= rec2;
        v2.colour *= rec2;
        v2.uv *= rec2;

        // Map to screen
        v0 = Self::ndc_to_screen(v0, width, height);
        v1 = Self::ndc_to_screen(v1, width, height);
        v2 = Self::ndc_to_screen(v2, width, height);

        // Get bounds of triangle
        let x_min =
            (v0.position.x.min(v1.position.x).min(v2.position.x) as usize).clamp(0, width - 1);
        let y_min =
            (v0.position.y.min(v1.position.y).min(v2.position.y) as usize).clamp(0, height - 1);
        let x_max =
            (v0.position.x.max(v1.position.x).max(v2.position.x) as usize).clamp(0, width - 1);
        let y_max =
            (v0.position.y.max(v1.position.y).max(v2.position.y) as usize).clamp(0, height - 1);

        // Don't render off screen triangles
        if (y_max as i32 - y_min as i32) <= 0 {
            return;
        }
        if (x_max as i32 - x_min as i32) <= 0 {
            return;
        }
        let area = edge_function(v0.position.xy(), v1.position.xy(), v2.position.xy()) * 0.5;
        let inv_area = 1.0 / area;

        // Calculate mip level
        let mut mip_level = 0.0;
        let mut texture = None;
        let mut is_mag = false;
        if let Some(material) = material {
            // Calculate the area of the part of the texture that is on screen
            texture = Some(&material.texture);
            let texture = texture.unwrap();
            let texture_size = glam::vec2(texture.width as f32, texture.height as f32);
            let texture_area = edge_function(
                v0_in.uv * texture_size,
                v1_in.uv * texture_size,
                v2_in.uv * texture_size,
            );
            is_mag = texture_area.abs() > area.abs();

            // Calculate the mip level by comparing the area of the texture pixels and the area of the screen pixels
            let tex_area_log2 = texture_area.abs().log2();
            let area_log2 = area.abs().log2();
            mip_level = tex_area_log2 - area_log2;
            mip_level *= 1.0; // Some manual tweaking to make it look better
            mip_level = mip_level.clamp(0.0, (texture.mipmap_offsets.len() - 2) as f32);
        }

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                // Determine whether the point is on the triangle
                let coords = glam::vec2(x as f32, y as f32);
                let edge0 = edge_function(v1.position.xy(), v2.position.xy(), coords);
                let edge1 = edge_function(v2.position.xy(), v0.position.xy(), coords);
                let edge2 = edge_function(v0.position.xy(), v1.position.xy(), coords);

                //If so, interpolate the colours of the vertex
                if edge0 >= 0.0 && edge1 >= 0.0 && edge2 >= 0.0 {
                    //Get barycentric coordinates, texture coordinates, get the vertex colours, and sample the texture
                    let bary = glam::vec3(edge0 * inv_area, edge1 * inv_area, edge2 * inv_area);
                    let position = lerp_bary(&bary, &v0.position, &v1.position, &v2.position, None);

                    // Calculate depth of current pixel
                    let new_depth = position.z / position.w;

                    // Make depth influence mip level
                    mip_level *= 1.0 - new_depth;

                    // Depth testing
                    if new_depth < depth_buffer[x + y * width] {
                        continue;
                    }

                    // Frustrum culling
                    if !(0.0..=1.0).contains(&new_depth) {
                        continue;
                    }

                    let correction = bary.x * rec0 + bary.y * rec1 + bary.z * rec2;
                    let correction = 1.0 / correction;
                    let tex_coords = lerp_bary(&bary, &v0.uv, &v1.uv, &v2.uv, Some(correction));
                    let normal =
                        lerp_bary(&bary, &v0.normal, &v1.normal, &v2.normal, Some(correction));
                    let _tangent = lerp_bary(
                        &bary,
                        &v0.tangent,
                        &v1.tangent,
                        &v2.tangent,
                        Some(correction),
                    ); // not used, but included in case I have time to add normal mapping
                    let mut colour =
                        lerp_bary(&bary, &v0.colour, &v1.colour, &v2.colour, Some(correction));
                    if false {
                        colour.x = normal.x * 0.5 + 0.5;
                        colour.y = normal.y * 0.5 + 0.5;
                        colour.z = normal.z * 0.5 + 0.5;
                    }
                    if false {
                        colour.x = 1.0;
                        colour.y = 1.0;
                        colour.z = 1.0;
                    }
                    if true {
                        // Very basic lighting NdotL
                        colour *= normal.dot(glam::vec3(1.0, 0.5, 0.0).normalize()) * 0.5 + 0.5;
                    }
                    if let Some(tex) = texture {
                        // Sample texture
                        let texture_sample = tex.argb_at_uv(
                            tex_coords.x,
                            tex_coords.y,
                            mip_level as usize,
                            is_mag,
                            material.unwrap(),
                        );
                        colour.x *= ((texture_sample) & 0xFF) as f32 / 255.0;
                        colour.y *= ((texture_sample >> 8) & 0xFF) as f32 / 255.0;
                        colour.z *= ((texture_sample >> 16) & 0xFF) as f32 / 255.0;
                        if (((texture_sample >> 24) & 0xFF) as f32 / 255.0) < 0.5 {
                            continue;
                        }
                    }
                    //*i = texture_sample;
                    colour_buffer[x + y * width] = colour_rgb(
                        (colour.x * 255.0) as u8,
                        (colour.y * 255.0) as u8,
                        (colour.z * 255.0) as u8,
                    );
                    // Write to depth buffer
                    depth_buffer[x + y * width] = new_depth;
                    //*i = colour_rgb((tex_coords.x * 255.0) as u8, (tex_coords.y * 255.0) as u8, 0);
                }
            }
        }
    }

    fn vertex_shader(&self, vert: &Vertex, model_matrix: &Mat4) -> FragIn {
        let mut v = glam::vec4(vert.position.x, vert.position.y, vert.position.z, 1.0);
        v = model_matrix.mul_vec4(v);
        v = self.view_matrix.mul_vec4(v);
        v = self.projection_matrix.mul_vec4(v);
        FragIn {
            position: v,
            normal: vert.normal,
            tangent: vert.tangent,
            colour: vert.colour,
            uv: vert.uv,
        }
    }

    pub fn draw_mesh(
        &self,
        mesh: &Mesh,
        model_matrix: &Transform,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
        material: Option<&Material>,
    ) {
        for i in (0..mesh.verts.len()).step_by(3) {
            // Transform vertices
            let v0 = mesh.verts[i];
            let v1 = mesh.verts[i + 1];
            let v2 = mesh.verts[i + 2];
            let v0 = self.vertex_shader(&v0, &model_matrix.trans_matrix());
            let v1 = self.vertex_shader(&v1, &model_matrix.trans_matrix());
            let v2 = self.vertex_shader(&v2, &model_matrix.trans_matrix());

            // Create the vector for output triangles
            let mut new_triangles = Vec::<FragIn>::new();

            // Clip against near plane
            {
                // Check how many triangles are behind the near plane
                let mut n_outside = 0;
                if v0.position.z < 0.0 {
                    n_outside += 1;
                }
                if v1.position.z < 0.0 {
                    n_outside += 1;
                }
                if v2.position.z < 0.0 {
                    n_outside += 1;
                }

                // n_outside = 0;

                match n_outside {
                    // If all vertices are in front of the near plane, don't do anything
                    0 => {
                        new_triangles.push(v0);
                        new_triangles.push(v1);
                        new_triangles.push(v2);
                    }

                    // If one vertex is behind the near plane, clip it, we should get 2 triangles back
                    1 => {
                        // Order vertices so that C is always the one behind the near clipping plane
                        let a;
                        let b;
                        let c;
                        if v0.position.z < 0.0 {
                            a = v1;
                            b = v2;
                            c = v0;
                        } else if v1.position.z < 0.0 {
                            a = v2;
                            b = v0;
                            c = v1;
                        } else {
                            // if v2.position.z < 0.0
                            a = v0;
                            b = v1;
                            c = v2;
                        }

                        // Calculate mid_AC and mid_BC
                        let t_mid_ac = (0.0 - a.position.z) / (c.position.z - a.position.z);
                        let t_mid_bc = (0.0 - b.position.z) / (c.position.z - b.position.z);
                        let mid_ac = a.lerp(c, t_mid_ac);
                        let mid_bc = b.lerp(c, t_mid_bc);

                        // Triangle 1
                        new_triangles.push(a);
                        new_triangles.push(b);
                        new_triangles.push(mid_ac);

                        // Triangle 2
                        new_triangles.push(mid_ac);
                        new_triangles.push(b);
                        new_triangles.push(mid_bc);
                    }

                    // If two vertices are behind the near clipping plane, we should get one triangle back
                    2 => {
                        // Order vertices so that A is always the one in front of the near clipping plane
                        let a;
                        let b;
                        let c;
                        if v0.position.z > 0.0 {
                            a = v0;
                            b = v1;
                            c = v2;
                        } else if v1.position.z > 0.0 {
                            a = v1;
                            b = v2;
                            c = v0;
                        } else {
                            // if v2.position.z > 0.0
                            a = v2;
                            b = v0;
                            c = v1;
                        }

                        // Calculate mid_AC and mid_BC
                        let t_mid_ab = (0.0 - a.position.z) / (b.position.z - a.position.z);
                        let t_mid_ac = (0.0 - a.position.z) / (c.position.z - a.position.z);
                        let mid_ab = a.lerp(b, t_mid_ab);
                        let mid_ac = a.lerp(c, t_mid_ac);

                        // Return triangle
                        new_triangles.push(a);
                        new_triangles.push(mid_ab);
                        new_triangles.push(mid_ac);
                    }

                    // Otherwise, don't render
                    _ => {}
                }
            }

            // Perform perspective divide
            for item in &mut new_triangles {
                item.position.x /= item.position.w;
                item.position.y /= item.position.w;
                item.position.z /= item.position.w;
            }

            // Draw vertices
            for i in (0..new_triangles.len()).step_by(3) {
                Self::draw_triangle_filled(
                    new_triangles[i],
                    new_triangles[i + 1],
                    new_triangles[i + 2],
                    colour_buffer,
                    depth_buffer,
                    width,
                    height,
                    material,
                );
            }
        }
    }

    pub fn draw_model(
        &self,
        model: &Model,
        model_matrix: &Transform,
        colour_buffer: &mut [u32],
        depth_buffer: &mut [f32],
        width: usize,
        height: usize,
    ) {
        for (tex_id, mesh) in &model.meshes {
            self.draw_mesh(
                mesh,
                model_matrix,
                colour_buffer,
                depth_buffer,
                width,
                height,
                match tex_id.as_str() {
                    "None" => None,
                    _ => Some(&self.materials[tex_id]),
                },
            );
        }
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
    }
    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
    }
}
