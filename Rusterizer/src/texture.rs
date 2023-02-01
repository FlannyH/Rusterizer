use crate::helpers::*;
use std::path::Path;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u32>,
    pub mipmap_offsets: Vec<usize>,
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
                    mipmap_offsets: vec![0; 1],
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
                    mipmap_offsets: vec![0; 1],
                }
            } else {
                panic!("Unsupported texture type");
            }
        } else {
            panic!("Unsupported texture type");
        }
    }

    //Get ARGB value from a UV coordinate
    pub fn argb_at_uv(&self, u: f32, v: f32, mip_level: usize) -> u32 {
        let (u, v) = (
            (u % 1.0) * (self.width >> mip_level) as f32,
            (v % 1.0) * (self.height >> mip_level) as f32,
        );
        let id = coords_to_index(u as u32, v as u32, (self.width >> mip_level) as u32);

        //If the data is in bounds, show that pixel. Otherwise, show debug pink
        if id < self.data.len() {
            self.data[id + self.mipmap_offsets[mip_level]]
        } else {
            colour_rgba(255, 255, 0, 255)
        }
    }

    pub fn load_texture_from_gltf_image(image: &gltf::image::Data) -> Texture {
        Texture {
            width: image.width as usize,
            height: image.height as usize,
            depth: 4,
            data: {
                let mut data = Vec::<u32>::new();
                for i in (0..image.pixels.len()).step_by(4) {
                    data.push(
                        (image.pixels[i] as u32)
                            + ((image.pixels[i + 1] as u32) << 8)
                            + ((image.pixels[i + 2] as u32) << 16)
                            + ((image.pixels[i + 3] as u32) << 24),
                    );
                }
                data
            },
            mipmap_offsets: vec![0; 1],
        }
    }

    pub fn generate_mipmaps(&mut self) {
        // Set up first target
        let mut src_offset = 0;

        // Iterate until dst_width is 0
        let mut i = 0;
        loop {
            // Calculate resolutions
            let src_width = self.width >> i;
            let src_height = self.height >> i;
            let dst_width = self.width >> (i + 1);
            let dst_height = self.height >> (i + 1);

            if dst_width == 0 || dst_height == 0 {
                break;
            }

            // Create new texture vector for simplicity sake
            let mut new_mipmap = Vec::<u32>::new();
            for y in 0..dst_height {
                for x in 0..dst_width {
                    // Sample 4 pixels from the source and combine them into one
                    let pixel_sample1 =
                        self.data[src_offset + ((x * 2) + 0) + ((y * 2) + 0) * src_width];
                    let pixel_sample2 =
                        self.data[src_offset + ((x * 2) + 1) + ((y * 2) + 0) * src_width];
                    let pixel_sample3 =
                        self.data[src_offset + ((x * 2) + 0) + ((y * 2) + 1) * src_width];
                    let pixel_sample4 =
                        self.data[src_offset + ((x * 2) + 1) + ((y * 2) + 1) * src_width];
                    let r = (((pixel_sample1 >> 0) & 0xFF)
                        + ((pixel_sample2 >> 0) & 0xFF)
                        + ((pixel_sample3 >> 0) & 0xFF)
                        + ((pixel_sample4 >> 0) & 0xFF))
                        / 4;
                    let g = (((pixel_sample1 >> 8) & 0xFF)
                        + ((pixel_sample2 >> 8) & 0xFF)
                        + ((pixel_sample3 >> 8) & 0xFF)
                        + ((pixel_sample4 >> 8) & 0xFF))
                        / 4;
                    let b = (((pixel_sample1 >> 16) & 0xFF)
                        + ((pixel_sample2 >> 16) & 0xFF)
                        + ((pixel_sample3 >> 16) & 0xFF)
                        + ((pixel_sample4 >> 16) & 0xFF))
                        / 4;
                    let a = (((pixel_sample1 >> 24) & 0xFF)
                        + ((pixel_sample2 >> 24) & 0xFF)
                        + ((pixel_sample3 >> 24) & 0xFF)
                        + ((pixel_sample4 >> 24) & 0xFF))
                        / 4;
                    let pixel_output = (r) | (g << 8) | (b << 16) | (a << 24);

                    // Write it to the output buffer
                    new_mipmap.push(pixel_output);
                }
            }

            // Store the mipmap
            let new_mipmap_offset = self.data.len();
            self.mipmap_offsets.push(new_mipmap_offset);
            self.data.append(&mut new_mipmap);

            // Move to the mipmap we just created
            src_offset = new_mipmap_offset;
            i += 1;
        }
    }
}
