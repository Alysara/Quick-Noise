use std::{fs, path::Path};

use crate::worley::Worley;
use crate::{noise::perlin::*, simd::simd_array::SimdArray};
use crate::noise::simplex::Simplex;
use crate::noise::value::Value;

pub fn write_perlin_height_map(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut perlin = Perlin::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinMap::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            perlin.uniform_grid_2d(
                &mut noise,
                (x as i32, y as i32).into(),
                octaves,
                scale,
                1.0,
                lacunarity,
                persistence,
                1,
                0.0,
            );

            noise = (noise + PerlinMap::new(1.0)) * PerlinMap::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_perlin_octaves_height_map(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: impl IntoIterator<Item = impl Into<Octave2D>>,
    channel: i32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut perlin = Perlin::new(0);

    let octaves_vec: Vec<Octave2D> = octaves.into_iter().map(Into::into).collect();
    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut noise: PerlinMap = perlin.uniform_grid_2d_octaves(
                (x as i32, y as i32).into(),
                &octaves_vec,
                1.0,
                channel,
                0.0,
            );

            noise = (noise + PerlinMap::new(1.0)) * PerlinMap::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_perlin_height_map_3d(
    path: &str,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    let mut perlin = Perlin::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut array = PerlinVol::new_uninit();

    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            perlin.uniform_grid_3d(
                &mut array,
                (0, x as i32, y as i32).into(),
                octaves,
                scale,
                1.0,
                lacunarity,
                persistence,
                1,
                0.0,
            );

            array = (array + PerlinVol::new(1.0)) * PerlinVol::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = array[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {path}!");
}


pub fn write_perlin_height_map_3d_octaves(
    path: &str,
    dimension: usize,
    octaves: impl IntoIterator<Item = impl Into<Octave3D>>,
    channel: i32,
) {
    let mut perlin = Perlin::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let octaves_vec: Vec<Octave3D> = octaves.into_iter().map(Into::into).collect();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut array = perlin.uniform_grid_3d_octaves(
                (0, x as i32, y as i32).into(),
                &octaves_vec,
                1.,
                channel,
                0.
            );

            array = (array + PerlinVol::new(1.0)) * PerlinVol::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = array[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {path}!");
}

pub fn write_perlin_height_map_batched(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut perlin = Perlin::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinMap::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinMap::new_uninit();
            let mut y_array = PerlinMap::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    x_array[dx * ROW_SIZE + dy] = dx as f32 + x_start;
                    y_array[dx * ROW_SIZE + dy] = dy as f32 + y_start;
                }
            }

            let octave = Octave2D::splat(scale, 1.0);

            perlin.batched_2d(
                &mut noise,
                &x_array,
                &y_array,
                &octave,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinMap::new(1.0)) * PerlinMap::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}


pub fn write_simplex_height_map_batched(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut simplex = Simplex::new(2);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinMap::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinMap::new_uninit();
            let mut y_array = PerlinMap::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    x_array[dx * ROW_SIZE + dy] = dx as f32 + x_start;
                    y_array[dx * ROW_SIZE + dy] = dy as f32 + y_start;
                }
            }

            simplex.batched_2d(
                &mut noise,
                &x_array,
                &y_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinMap::new(1.0)) * PerlinMap::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}


pub fn write_value_height_map_batched(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut value = Value::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinMap::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinMap::new_uninit();
            let mut y_array = PerlinMap::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    x_array[dx * ROW_SIZE + dy] = dx as f32 + x_start;
                    y_array[dx * ROW_SIZE + dy] = dy as f32 + y_start;
                }
            }

            value.batched_2d(
                &mut noise,
                &x_array,
                &y_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinMap::new(1.0)) * PerlinMap::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_worley_height_map_batched(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut worley = Worley::new(0);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinMap::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinMap::new_uninit();
            let mut y_array = PerlinMap::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    x_array[dx * ROW_SIZE + dy] = dx as f32 + x_start;
                    y_array[dx * ROW_SIZE + dy] = dy as f32 + y_start;
                }
            }

            worley.batched_2d(
                &mut noise,
                &x_array,
                &y_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = noise * PerlinMap::new(256.0);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * ROW_SIZE + dy] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_perlin_height_map_batched_3d(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut perlin = Perlin::new(1);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinVol::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinVol::new_uninit();
            let mut y_array = PerlinVol::new_uninit();
            let mut z_array = PerlinVol::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    for dz in 0..ROW_SIZE {
                        x_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dx as f32 + x_start;
                        y_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dy as f32 + y_start;
                        z_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dz as f32;
                    }
                }
            }

            let octave = Octave3D::splat(scale, 1.0);

            perlin.batched_3d(
                &mut noise,
                &x_array,
                &y_array,
                &z_array,
                &octave,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinVol::new(1.0)) * PerlinVol::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * MAP_SIZE + dy * ROW_SIZE] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_value_height_map_batched_3d(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut value = Value::new(1);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinVol::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinVol::new_uninit();
            let mut y_array = PerlinVol::new_uninit();
            let mut z_array = PerlinVol::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    for dz in 0..ROW_SIZE {
                        x_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dx as f32 + x_start;
                        y_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dy as f32 + y_start;
                        z_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dz as f32;
                    }
                }
            }

            value.batched_3d(
                &mut noise,
                &x_array,
                &y_array,
                &z_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinVol::new(1.0)) * PerlinVol::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * MAP_SIZE + dy * ROW_SIZE] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}


pub fn write_simplex_height_map_batched_3d(
    path: impl AsRef<Path>,
    dimension: usize,
    octaves: u32,
    scale: f32,
    lacunarity: f32,
    persistence: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut simplex = Simplex::new(1);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinVol::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinVol::new_uninit();
            let mut y_array = PerlinVol::new_uninit();
            let mut z_array = PerlinVol::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    for dz in 0..ROW_SIZE {
                        x_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dx as f32 + x_start;
                        y_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dy as f32 + y_start;
                        z_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dz as f32;
                    }
                }
            }

            simplex.batched_3d(
                &mut noise,
                &x_array,
                &y_array,
                &z_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = (noise + PerlinVol::new(1.0)) * PerlinVol::new(127.5);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * MAP_SIZE + dy * ROW_SIZE] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

pub fn write_cellular_height_map_batched_3d(
    path: impl AsRef<Path>,
    dimension: usize,
    scale: f32,
) {
    if let Some(parent) = path.as_ref().parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).expect("Failed to create parent");
    }

    let mut cellular = Worley::new(1);

    let mut pixels = Vec::<u8>::new();
    pixels.resize(dimension * dimension * MAP_SIZE, 0);

    let mut noise = PerlinVol::new_uninit();
    for x in 0..dimension {
        let x_offset = x * dimension * MAP_SIZE;
        for y in 0..dimension {
            let y_offset = y * ROW_SIZE;

            let mut x_array = PerlinVol::new_uninit();
            let mut y_array = PerlinVol::new_uninit();
            let mut z_array = PerlinVol::new_uninit();

            let x_start = (x * ROW_SIZE) as f32;
            let y_start = (y * ROW_SIZE) as f32;

            for dx in 0..ROW_SIZE {
                for dy in 0..ROW_SIZE {
                    for dz in 0..ROW_SIZE {
                        x_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dx as f32 + x_start;
                        y_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dy as f32 + y_start;
                        z_array[dx * MAP_SIZE + dy * ROW_SIZE + dz] = dz as f32;
                    }
                }
            }

            cellular.batched_3d(
                &mut noise,
                &x_array,
                &y_array,
                &z_array,
                scale,
                1.0,
                1,
                0.0,
            );

            noise = noise * PerlinVol::new(256.0);

            for dx in 0..ROW_SIZE {
                let offset = x_offset + y_offset + dx * ROW_SIZE * dimension;
                for dy in 0..ROW_SIZE {
                    pixels[offset + dy] = noise[dx * MAP_SIZE + dy * ROW_SIZE] as u8;
                }
            }
        }
    }

    let pixel_dimension = (dimension * ROW_SIZE) as u32;
    image::save_buffer(
        &path,
        &pixels,
        pixel_dimension,
        pixel_dimension,
        image::ColorType::L8,
    )
    .expect("Failed to write height map!");

    println!("Wrote height map to {}!", path.as_ref().display());
}

