use crate::noise::perlin::{Perlin, PerlinMap, PerlinVol};
use crate::perlin::Octave2D;
use std::time::Instant;
use std::hint::black_box;

pub fn profile_perlin_2d_call(octaves: u32, scale: f32, lacunarity: f32, persistence: f32) {
    const NUM_LOOPS: usize = 10000000;
    const SAMPLE_SIZE: usize = 1024;

    let mut perlin = Perlin::new(0);

    let mut array = PerlinMap::new_uninit();
    let start = Instant::now();
    for i in 0..NUM_LOOPS {
        perlin.uniform_grid_2d(&mut array, (i as i32, i as i32).into(), octaves, scale, 1.0, lacunarity, persistence, 1, 0.0);
        black_box(&array);
    }
    let elapsed = start.elapsed();
    let ms_elapsed = elapsed.as_millis();

    let total = NUM_LOOPS * SAMPLE_SIZE;
    let elapsed_per_loop = elapsed.as_nanos() as u64 / NUM_LOOPS as u64;
    let samples_per_second = (total as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "-+- Completed 2D Perlin Noise Profile -+-\n\
        -> {total} samples in {ms_elapsed} ms!\n\
        -> {elapsed_per_loop} ns per {SAMPLE_SIZE} samples!\n\
        -> {samples_per_second} samples per second!"
    );
}

pub fn profile_perlin_2d_batched_call(octaves: u32, scale: f32, lacunarity: f32, persistence: f32) {
    const NUM_LOOPS: usize = 1000000;
    const SAMPLE_SIZE: usize = 1024;

    let mut perlin = Perlin::new(0);

    let mut x_array = PerlinMap::new_uninit();
    let mut y_array = PerlinMap::new_uninit();

    const ROW_SIZE: usize = 32;
    for x in 0..ROW_SIZE {
        for y in 0..ROW_SIZE {
            x_array[x * ROW_SIZE + y] = x as f32;
            y_array[x * ROW_SIZE + y] = y as f32;
        }
    }

    let mut array = PerlinMap::new_uninit();
    let octave = Octave2D::splat(1.0/scale, 1.0);

    let start = Instant::now();
    for _ in 0..NUM_LOOPS {
        perlin.batched_2d(&mut array, &x_array, &y_array, &octave, 1.0, 1, 0.0);
        black_box(&array);
    }
    let elapsed = start.elapsed();
    let ms_elapsed = elapsed.as_millis();

    let total = NUM_LOOPS * SAMPLE_SIZE;
    let elapsed_per_loop = elapsed.as_nanos() as u64 / NUM_LOOPS as u64;
    let samples_per_second = (total as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "-+- Completed 2D Perlin Noise Profile -+-\n\
        -> {total} samples in {ms_elapsed} ms!\n\
        -> {elapsed_per_loop} ns per {SAMPLE_SIZE} samples!\n\
        -> {samples_per_second} samples per second!"
    );
}

pub fn profile_perlin_3d_call(octaves: u32, scale: f32, lacunarity: f32, persistence: f32) {
    const NUM_LOOPS: usize = 1000000;
    const SAMPLE_SIZE: usize = 32768;

    let mut perlin = Perlin::new(0);

    let mut array = PerlinVol::new_uninit();
    let start = Instant::now();
    for i in 0..NUM_LOOPS {
        perlin.uniform_grid_3d(&mut array, (i as i32, i as i32, i as i32).into(), octaves, scale, 1.0, lacunarity, persistence, 1, 0.0);
        black_box(&array);
    }
    let elapsed = start.elapsed();
    let ms_elapsed = elapsed.as_millis();

    let total = NUM_LOOPS * SAMPLE_SIZE;
    let elapsed_per_loop = elapsed.as_nanos() as u64 / NUM_LOOPS as u64;
    let samples_per_second = (total as f64 / elapsed.as_secs_f64()) as u64;

    println!(
        "-+- Completed 3D Perlin Noise Profile -+-\n\
        -> {total} samples in {ms_elapsed} ms!\n\
        -> {elapsed_per_loop} ns per {SAMPLE_SIZE} samples!\n\
        -> {samples_per_second} samples per second!"
    );
}

pub fn bench_perlin_2d() {
    println!("-+-+- Perlin 2D Noise Bench -+-+-");

    profile_perlin_2d_call_internal(1, 64.0, 100000);
    profile_perlin_2d_call_internal(1, 48.0, 100000);
    profile_perlin_2d_call_internal(1, 32.0, 100000);
    profile_perlin_2d_call_internal(1, 24.0, 100000);
    profile_perlin_2d_call_internal(1, 16.0, 100000);
    profile_perlin_2d_call_internal(1, 12.0, 75000);
    profile_perlin_2d_call_internal(1, 8.0, 50000);
    profile_perlin_2d_call_internal(1, 6.0, 50000);
    profile_perlin_2d_call_internal(1, 4.0, 25000);
    profile_perlin_2d_call_internal(1, 3.0, 25000);
    profile_perlin_2d_call_internal(1, 2.0, 10000);

    println!("-+-+- Completed Perlin 2D Bench -+-+-");
}

pub fn bench_perlin_3d() {
    println!("-+-+- Perlin 3D Noise Bench -+-+-");

    profile_perlin_3d_call_internal(1, 64.0, 100000);
    profile_perlin_3d_call_internal(1, 48.0, 100000);
    profile_perlin_3d_call_internal(1, 32.0, 100000);
    profile_perlin_3d_call_internal(1, 24.0, 100000);
    profile_perlin_3d_call_internal(1, 16.0, 100000);
    profile_perlin_3d_call_internal(1, 12.0, 75000);
    profile_perlin_3d_call_internal(1, 8.0, 50000);
    profile_perlin_3d_call_internal(1, 6.0, 50000);
    profile_perlin_3d_call_internal(1, 4.0, 25000);
    profile_perlin_3d_call_internal(1, 3.0, 25000);
    profile_perlin_3d_call_internal(1, 2.0, 10000);

    println!("-+-+- Completed Perlin 2D Bench -+-+-");
}

fn profile_perlin_2d_call_internal(octaves: u32, scale: f32, num_loops: usize) {
    const SAMPLE_SIZE: usize = 1024;
    
    let mut perlin = Perlin::new(0);

    let mut array = PerlinMap::new_uninit();
    let start = Instant::now();
    for i in 0..num_loops {
        perlin.uniform_grid_2d(&mut array, (i as i32, i as i32).into(), octaves, scale, 1.0, 2.0, 0.5, 1, 0.0);
        black_box(&array);
    }
    let elapsed = start.elapsed();

    let elapsed_per_loop = elapsed.as_nanos() as u64 / num_loops as u64;
    println!("Scale {scale} -> {elapsed_per_loop} ns per {SAMPLE_SIZE} samples!");
}

fn profile_perlin_3d_call_internal(octaves: u32, scale: f32, num_loops: usize) {
    const SAMPLE_SIZE: usize = 32768;
    
    let mut perlin = Perlin::new(0);

    let mut array = PerlinVol::new_uninit();
    let start = Instant::now();
    for i in 0..num_loops {
        perlin.uniform_grid_3d(&mut array, (i as i32, i as i32, i as i32).into(), octaves, scale, 1.0, 2.0, 0.5, 1, 0.0);
        black_box(&array);
    }
    let elapsed = start.elapsed();

    let elapsed_per_loop = elapsed.as_nanos() as u64 / num_loops as u64;
    println!("Scale {scale} -> {elapsed_per_loop} ns per {SAMPLE_SIZE} samples!");
}