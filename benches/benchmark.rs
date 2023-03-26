use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;

use parallelizator::{
    multithread_map_chat_gpt, multithread_map_crossbeam_channel, multithread_map_crossbeam_fast,
    multithread_map_rayon, single_thread_map,
};

// Define a function to generate a Vec of random integers
fn gen_data(n: usize) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    (0..n).map(|_| rng.gen_range(0..1000)).collect()
}

fn dup(x: u32) -> u32 {
    x * 2
}

fn fib(n: u32) -> u32 {
    let mut a = 0;
    let mut b = 1;

    match n {
        0 => b,
        _ => {
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

// Define the benchmark function
fn bench_parallel_map(c: &mut Criterion) {
    // let input = gen_data(10_000);
    let threshold = 100;

    // Benchmark the parallel_map function with different input sizes
    let mut group = c.benchmark_group("parallel_map");
    for n in [150, 1000, 5_000, 20_000].iter() {
        group.bench_with_input(format!("rayon_dup_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_rayon(gen_data(n), dup, threshold))
        });
        group.bench_with_input(format!("crossbeam_chan_dup_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_crossbeam_channel(gen_data(n), dup, threshold))
        });
        group.bench_with_input(format!("crossbeam_n_fast_dup_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_crossbeam_fast(gen_data(n), dup, threshold))
        });
        group.bench_with_input(format!("CHAT_GPT_dup_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_chat_gpt(gen_data(n), dup, threshold))
        });
        group.bench_with_input(format!("simple_dup_{}", n), n, |b, &n| {
            b.iter(|| single_thread_map(gen_data(n), dup))
        });

        group.bench_with_input(format!("rayon_fib_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_rayon(gen_data(n), fib, threshold))
        });
        group.bench_with_input(format!("crossbeam_chan_fib_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_crossbeam_channel(gen_data(n), fib, threshold))
        });
        group.bench_with_input(format!("crossbeam_fast_fib_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_crossbeam_fast(gen_data(n), fib, threshold))
        });
        group.bench_with_input(format!("CHAT_GPT_fib_{}", n), n, |b, &n| {
            b.iter(|| multithread_map_chat_gpt(gen_data(n), fib, threshold))
        });
        group.bench_with_input(format!("simple_fib_{}", n), n, |b, &n| {
            b.iter(|| single_thread_map(gen_data(n), fib))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parallel_map);
criterion_main!(benches);
