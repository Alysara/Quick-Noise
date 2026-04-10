use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fastnoise2::Node;
use quick_noise::perlin::{Octave2D, Octave3D, Perlin, PerlinMap, PerlinVol};
use quick_noise::simplex::Simplex;
use quick_noise::value::Value;
use quick_noise::worley::Worley;
const SCALES: [f32; 11] = [64.0, 48.0, 32.0, 24.0, 16.0, 12.0, 8.0, 6.0, 4.0, 3.0, 2.0];

fn perlin_2d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_2d");
    for scale in SCALES {
        group.throughput(Throughput::Elements(1024)); 
        group.bench_function(format!("scale: {scale}"), |b| {
            let mut perlin = Perlin::new(0);
            let mut array = PerlinMap::new_uninit();
            let mut i = 0;
            b.iter(|| {
                i = i + 1 & 0xFFFFFF;
                perlin.uniform_grid_2d(&mut array, (i, i).into(), 1, scale, 1.0, 2.0, 0.5, 1, 0.0)
            });
        });
    }
}

fn perlin_2d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_2d_batch");
    let scale = 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut perlin = Perlin::new(0);
        let mut x_array = PerlinMap::new_uninit();
        let mut y_array = PerlinMap::new_uninit();
        let mut output = PerlinMap::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                x_array[x * 32 + y] = x as f32;
                y_array[x * 32 + y] = y as f32;
            }
        }

        let octave = Octave2D::splat(scale, 1.0);

        b.iter(|| {
            perlin.batched_2d(&mut output, &x_array, &y_array, &octave, 1.0, 1, 0.0)
        });
    });
}

fn simplex_2d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplex_noise_2d_batch");
    let scale = 1.0 / 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut simplex = Simplex::new(0);
        let mut x_array = PerlinMap::new_uninit();
        let mut y_array = PerlinMap::new_uninit();
        let mut output = PerlinMap::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                x_array[x * 32 + y] = x as f32;
                y_array[x * 32 + y] = y as f32;
            }
        }

        b.iter(|| {
            simplex.batched_2d(&mut output, &x_array, &y_array, scale, 1.0, 1, 0.0)
        });
    });
}

fn value_2d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_noise_2d_batch");
    let scale = 1.0 / 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut value = Value::new(0);
        let mut x_array = PerlinMap::new_uninit();
        let mut y_array = PerlinMap::new_uninit();
        let mut output = PerlinMap::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                x_array[x * 32 + y] = x as f32;
                y_array[x * 32 + y] = y as f32;
            }
        }

        b.iter(|| {
            value.batched_2d(&mut output, &x_array, &y_array, scale, 1.0, 1, 0.0)
        });
    });
}


fn worley_2d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("worley_noise_2d_batch");
    let scale = 1.0 / 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut worley = Worley::new(0);
        let mut x_array = PerlinMap::new_uninit();
        let mut y_array = PerlinMap::new_uninit();
        let mut output = PerlinMap::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                x_array[x * 32 + y] = x as f32;
                y_array[x * 32 + y] = y as f32;
            }
        }

        b.iter(|| {
            worley.batched_2d(&mut output, &x_array, &y_array, scale, 1.0, 1, 0.0)
        });
    });
}


fn perlin_3d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_3d_batch");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut perlin = Perlin::new(0);
        let mut x_array = PerlinVol::new_uninit();
        let mut y_array = PerlinVol::new_uninit();
        let mut z_array = PerlinVol::new_uninit();
        let mut output = PerlinVol::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    x_array[x * 1024 + y * 32 + z] = x as f32;
                    y_array[x * 1024 + y * 32 + z] = y as f32;
                    z_array[x * 1024 + y * 32 + z] = z as f32;
                }
            }
        }

        let octave = Octave3D::splat(scale, 1.0);

        b.iter(|| {
            perlin.batched_3d(&mut output, &x_array, &y_array, &z_array, &octave, 1.0, 1, 0.0)
        });
    });
}

fn value_3d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_noise_3d_batch");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut value = Value::new(0);
        let mut x_array = PerlinVol::new_uninit();
        let mut y_array = PerlinVol::new_uninit();
        let mut z_array = PerlinVol::new_uninit();
        let mut output = PerlinVol::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    x_array[x * 1024 + y * 32 + z] = x as f32;
                    y_array[x * 1024 + y * 32 + z] = y as f32;
                    z_array[x * 1024 + y * 32 + z] = z as f32;
                }
            }
        }

        b.iter(|| {
            value.batched_3d(&mut output, &x_array, &y_array, &z_array, scale, 1.0, 1, 0.0)
        });
    });
}

fn cellular_3d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellular_noise_3d_batch");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut cellular = Worley::new(0);
        let mut x_array = PerlinVol::new_uninit();
        let mut y_array = PerlinVol::new_uninit();
        let mut z_array = PerlinVol::new_uninit();
        let mut output = PerlinVol::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    x_array[x * 1024 + y * 32 + z] = x as f32;
                    y_array[x * 1024 + y * 32 + z] = y as f32;
                    z_array[x * 1024 + y * 32 + z] = z as f32;
                }
            }
        }

        b.iter(|| {
            cellular.batched_3d(&mut output, &x_array, &y_array, &z_array, scale, 1.0, 1, 0.0)
        });
    });
}


fn simplex_3d_benchmark_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplex_noise_3d_batch");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let mut simplex = Simplex::new(0);
        let mut x_array = PerlinVol::new_uninit();
        let mut y_array = PerlinVol::new_uninit();
        let mut z_array = PerlinVol::new_uninit();
        let mut output = PerlinVol::new_uninit();
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    x_array[x * 1024 + y * 32 + z] = x as f32;
                    y_array[x * 1024 + y * 32 + z] = y as f32;
                    z_array[x * 1024 + y * 32 + z] = z as f32;
                }
            }
        }

        b.iter(|| {
            simplex.batched_3d(&mut output, &x_array, &y_array, &z_array, scale, 1.0, 1, 0.0)
        });
    });
}

fn perlin_2d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_2d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Perlin").unwrap();
        let mut array = [0f32; 1024];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_2d_unchecked(
                    &mut array,
                    offset,
                    offset,
                    32,
                    32,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn simplex_2d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplex_noise_2d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Simplex").unwrap();
        let mut array = [0f32; 1024];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_2d_unchecked(
                    &mut array,
                    offset,
                    offset,
                    32,
                    32,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn value_2d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_noise_2d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Value").unwrap();
        let mut array = [0f32; 1024];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_2d_unchecked(
                    &mut array,
                    offset,
                    offset,
                    32,
                    32,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn cellular_2d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellular_noise_2d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(1024)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("cellulardistance").unwrap();
        let mut array = [0f32; 1024];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_2d_unchecked(
                    &mut array,
                    offset,
                    offset,
                    32,
                    32,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}


fn perlin_3d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_3d");
    for scale in SCALES {
        group.throughput(Throughput::Elements(32768)); 
        group.bench_function(format!("scale: {scale}"), |b| {
            let mut perlin = Perlin::new(0);
            let mut array = PerlinVol::new_uninit();
            let mut i = 0;
            b.iter(|| {
                i = i + 1 & 0xFFFFFF;
                perlin.uniform_grid_3d(
                    &mut array,
                    (i, i, i).into(),
                    1,
                    scale,
                    1.0,
                    2.0,
                    0.5,
                    1,
                    0.0,
                )
            });
        });
    }
}

fn perlin_3d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_3d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Perlin").unwrap();
        let mut array = [0f32; 32768];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_3d_unchecked(
                    &mut array, 
                    offset,
                    offset,
                    offset,
                    32,
                    32,
                    32,
                    scale,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn value_3d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_noise_3d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Value").unwrap();
        let mut array = [0f32; 32768];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_3d_unchecked(
                    &mut array, 
                    offset,
                    offset,
                    offset,
                    32,
                    32,
                    32,
                    scale,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn cellular_3d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("cellular_noise_3d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("cellulardistance").unwrap();
        let mut array = [0f32; 32768];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_3d_unchecked(
                    &mut array, 
                    offset,
                    offset,
                    offset,
                    32,
                    32,
                    32,
                    scale,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

fn simplex_3d_benchmark_fn2(c: &mut Criterion) {
    let mut group = c.benchmark_group("simplex_noise_3d_fn2");
    let scale = 32.0;
    group.throughput(Throughput::Elements(32768)); 
    group.bench_function(format!("scale: {scale}"), |b| {
        let node = Node::from_name("Simplex").unwrap();
        let mut array = [0f32; 32768];
        let mut i = 0;
        let scale = 1f32 / scale as f32;
        b.iter(|| {
            i = i + 1 & 0xFFFFFF;
            let offset = (i * 32) as f32;
            unsafe {
                node.gen_uniform_grid_3d_unchecked(
                    &mut array, 
                    offset,
                    offset,
                    offset,
                    32,
                    32,
                    32,
                    scale,
                    scale,
                    scale,
                    0
                );
            }
        });
    });
}

criterion_group!(benches, perlin_2d_benchmark, perlin_3d_benchmark, perlin_2d_benchmark_fn2, perlin_3d_benchmark_fn2);
// criterion_group!(benches, perlin_2d_benchmark_batch, perlin_2d_benchmark_fn2, perlin_3d_benchmark_batch, perlin_3d_benchmark_fn2);
// criterion_group!(benches, simplex_2d_benchmark_batch, simplex_2d_benchmark_fn2);
// criterion_group!(benches, value_2d_benchmark_batch, perlin_2d_benchmark_batch, simplex_2d_benchmark_batch);
// criterion_group!(benches, worley_2d_benchmark_batch, cellular_2d_benchmark_fn2);
// criterion_group!(benches, cellular_3d_benchmark_batch, cellular_3d_benchmark_fn2);
// criterion_group!(benches, value_3d_benchmark_batch, value_3d_benchmark_fn2);
// criterion_group!(benches, simplex_3d_benchmark_batch, simplex_3d_benchmark_fn2);
// criterion_group!(benches, perlin_3d_benchmark);
criterion_main!(benches);
