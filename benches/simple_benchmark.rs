use bondi::Bondi;
use criterion::{criterion_group, criterion_main, Criterion};

fn asd() {
    let num_elements = 10_000;
    let bondi = Bondi::<usize>::new(1000);
    let writer = bondi.get_tx().unwrap();
    let readers = (0..10).into_iter().map(|_| bondi.get_rx().unwrap());
    std::thread::spawn(move || {
        for i in 0..num_elements {
            writer.write(i);
        }
    });

    let mut v = vec![];
    for reader in readers {
        v.push(std::thread::spawn(move || {
            for _ in 0..num_elements {
                reader.read();
            }
        }));
    }

    for t in v {
        t.join().unwrap();
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("ten_consumers_10k_usizes", |b| b.iter(|| asd()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
