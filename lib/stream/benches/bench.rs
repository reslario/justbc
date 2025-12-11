use {criterion::*, stream::AudioStream};

criterion_group!(benches, stream);
criterion_main!(benches);

fn stream(c: &mut Criterion) {
    use std::io::Read;

    const READ: usize = stream::BUF_SIZE / 4;

    c.bench_function("stream", |b| {
        b.iter(|| {
            let mut stream =
                AudioStream::new(std::io::repeat(0x00F).take(stream::BUF_SIZE as u64 * 100))
                    .unwrap();

            let mut buf = [0; READ];
            while stream.read(&mut buf).unwrap() != 0 {}
        })
    });
}
