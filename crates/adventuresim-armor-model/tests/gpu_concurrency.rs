use adventuresim_armor_model::ArmorGpu;

fn points(count: usize, seed: u32) -> Vec<[f32; 3]> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    (0..count).map(|_| [next(), next(), next()]).collect()
}

/// Every thread submits at the same moment, as a burst of fitted pieces does
/// when the shared device first opens.
#[test]
fn concurrent_nearest_points_agree_with_the_host() {
    let gpu = ArmorGpu::open().unwrap();
    let targets = points(18_000, 1);
    let threads = 32u32;
    let barrier = std::sync::Barrier::new(threads as usize);
    let failures = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for thread in 0..threads {
            let (gpu, targets, barrier, failures) = (&gpu, &targets, &barrier, &failures);
            scope.spawn(move || {
                for round in 0..6u32 {
                    let queries = points(1_000 + 37 * thread as usize, 7 + thread * 31 + round);
                    barrier.wait();
                    let found = gpu.nearest_points(&queries, targets).unwrap();
                    for (q, f) in queries.iter().zip(&found).step_by(97) {
                        let d =
                            |i: usize| (0..3).map(|a| (q[a] - targets[i][a]).powi(2)).sum::<f32>();
                        let best = (0..targets.len()).map(d).fold(f32::INFINITY, f32::min);
                        if d(*f as usize) > best * (1.0 + 1e-5) {
                            failures.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            });
        }
    });
    assert_eq!(failures.into_inner(), 0, "wrong nearest points");
}
