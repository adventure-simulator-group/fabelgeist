//! Observational CPU generation timings and mesh counts; no GPU cost is measured.
use adventuresim_plant_generator::{PlantSpecies, Tessellation};
use std::{hint::black_box, time::Instant};

const TIMING_SAMPLES: usize = 25;
const SPECIMEN_SEED: u64 = 42;

fn main() {
    println!("species,detail,vertices,triangles,first_us,median_us,p95_us");
    for species in PlantSpecies::ALL {
        for detail in [Tessellation::Close, Tessellation::Field] {
            let first = Instant::now();
            let mesh = species.generate(SPECIMEN_SEED, detail).unwrap();
            let first_us = first.elapsed().as_micros();
            let mut samples = Vec::with_capacity(TIMING_SAMPLES);
            for _ in 0..TIMING_SAMPLES {
                let start = Instant::now();
                let generated =
                    black_box(species.generate(black_box(SPECIMEN_SEED), detail).unwrap());
                samples.push(start.elapsed().as_micros());
                drop(generated);
            }
            samples.sort_unstable();
            println!(
                "{species:?},{detail:?},{},{},{first_us},{},{}",
                mesh.positions.len(),
                mesh.indices.len() / 3,
                samples[TIMING_SAMPLES / 2],
                samples[(TIMING_SAMPLES * 95).div_ceil(100) - 1],
            );
        }
    }
}
