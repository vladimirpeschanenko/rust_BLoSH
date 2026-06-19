use std::str::FromStr;

// Import our custom parser from the main crate
use assignment::assignment_two_data::parse_fixed_price;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;

fn bench_parser(c: &mut Criterion) {
    let price_str = "0.00245678";

    let mut group = c.benchmark_group("Price Parsing");

    group.bench_function("HFT Fixed-Point Parser (Custom)", |b| {
        b.iter(|| {
            let parsed = parse_fixed_price(black_box(price_str));
            black_box(parsed);
        })
    });

    group.bench_function("rust_decimal::Decimal::from_str", |b| {
        b.iter(|| {
            let parsed = Decimal::from_str(black_box(price_str)).unwrap();
            black_box(parsed);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
