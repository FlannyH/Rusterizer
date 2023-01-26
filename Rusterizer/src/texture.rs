use crate::helpers::*;
use std::path::Path;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u32>,
}

impl Texture {
    pub fn load(path: &Path) -> Self {
        //Load image
        let loaded_image = stb_image::image::load(path);

        //Map the image data to argb8 format
        if let stb_image::image::LoadResult::ImageU8(image) = loaded_image {
            if image.depth == 4 {
                let data = (0..image.data.len() / 4)
                    .map(|id| {
                        colour_rgba(
                            image.data[id * 4 + 3],
                            image.data[id * 4],
                            image.data[id * 4 + 1],
                            image.data[id * 4 + 2],
                        )
                    })
                    .collect();
                Self {
                    width: image.width,
                    height: image.height,
                    depth: image.depth,
                    data,
                }
            } else if image.depth == 3 {
                let data = (0..image.data.len() / 3)
                    .map(|id| {
                        colour_rgba(
                            255,
                            image.data[id * 3],
                            image.data[id * 3 + 1],
                            image.data[id * 3 + 2],
                        )
                    })
                    .collect();
                Self {
                    width: image.width,
                    height: image.height,
                    depth: image.depth,
                    data,
                }
            } else {
                panic!("Unsupported texture type");
            }
        } else {
            panic!("Unsupported texture type");
        }
    }

    //Get ARGB value from a UV coordinate
    pub fn argb_at_uv(&self, u: f32, v: f32) -> u32 {
        let (u, v) = (u * self.width as f32, v * self.height as f32);
        let id = coords_to_index(u as u32, v as u32, self.width as u32);

        //If the data is in bounds, show that pixel. Otherwise, show debug pink
        if id < self.data.len() {
            self.data[id]
        } else {
            colour_rgba(255, 255, 0, 255)
        }
    }
}
