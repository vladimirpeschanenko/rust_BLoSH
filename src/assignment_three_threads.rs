use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam::queue::ArrayQueue;

pub trait SharedData: Send + Sync {
    fn write(&self, new_value: f64);
    fn read(&self) -> Option<f64>;
    fn clear(&self);
}

#[repr(align(64))]
pub struct LatestSharedData {
    cell: AtomicU64,
}

// We use u64::MAX to represent `None`, since f64 bit patterns for NaN have many
// variations and it's extremely unlikely a valid price/quantity exactly equals
// u64::MAX bitwise.
const NONE_VALUE: u64 = u64::MAX;

impl Default for LatestSharedData {
    fn default() -> Self {
        Self::new()
    }
}

impl LatestSharedData {
    pub fn new() -> Self {
        Self {
            cell: AtomicU64::new(NONE_VALUE),
        }
    }
}

impl SharedData for LatestSharedData {
    #[inline(always)]
    fn write(&self, new_value: f64) {
        self.cell.store(new_value.to_bits(), Ordering::Release);
    }

    #[inline(always)]
    fn read(&self) -> Option<f64> {
        let val = self.cell.swap(NONE_VALUE, Ordering::AcqRel);
        if val == NONE_VALUE {
            None
        } else {
            Some(f64::from_bits(val))
        }
    }

    #[inline(always)]
    fn clear(&self) {
        self.cell.store(NONE_VALUE, Ordering::Release);
    }
}

#[repr(align(64))]
pub struct OrderedSharedData {
    // ArrayQueue is a bounded, heap-allocation-free ring buffer (Lock-Free).
    // The size is set to 100,000 to accommodate `bench_tenthousand_writes` cleanly
    // without dynamic allocations, preventing hot-path latency spikes.
    queue: ArrayQueue<f64>,
}

impl Default for OrderedSharedData {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderedSharedData {
    pub fn new() -> Self {
        Self {
            queue: ArrayQueue::new(100_000),
        }
    }
}

impl SharedData for OrderedSharedData {
    #[inline(always)]
    fn write(&self, new_value: f64) {
        // We drop the error if the queue is full since the trait doesn't return Result
        let _ = self.queue.push(new_value);
    }

    #[inline(always)]
    fn read(&self) -> Option<f64> {
        self.queue.pop()
    }

    #[inline(always)]
    fn clear(&self) {
        while self.queue.pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use std::thread;

    use rand::{rngs::StdRng, Rng, SeedableRng};

    use self::test::{black_box, Bencher};
    use crate::assignment_three_threads::{LatestSharedData, OrderedSharedData, SharedData};

    #[inline(always)]
    fn seeded_f64_samples(seed: u64, count: usize) -> Vec<f64> {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..count).map(|_| f64::from_bits(rng.gen::<u64>())).collect()
    }

    #[inline(always)]
    fn assert_same_f64_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "bit pattern mismatch: actual={actual:?}, expected={expected:?}"
        );
    }

    #[test]
    fn test_latest_read_and_write() {
        let shared = LatestSharedData::new();

        assert_eq!(shared.read(), None);
        shared.write(99.0);
        assert_eq!(shared.read(), Some(99.0));
        assert_eq!(shared.read(), None);
    }

    #[test]
    fn test_ordered_read_and_write() {
        let shared = OrderedSharedData::new();

        assert_eq!(shared.read(), None);
        shared.write(99.0);
        assert_eq!(shared.read(), Some(99.0));
    }

    #[test]
    fn test_latest_reads_in_write_order() {
        let shared = LatestSharedData::new();

        shared.write(7.0);
        shared.write(8.0);
        shared.write(9.0);

        assert_eq!(shared.read(), Some(9.0));
        assert_eq!(shared.read(), None);
    }

    #[test]
    fn test_ordered_reads_in_write_order() {
        let shared = OrderedSharedData::new();

        shared.write(7.0);
        shared.write(8.0);
        shared.write(9.0);

        assert_eq!(shared.read(), Some(7.0));
        assert_eq!(shared.read(), Some(8.0));
        assert_eq!(shared.read(), Some(9.0));
        assert_eq!(shared.read(), None);
    }

    #[test]
    fn test_latest_can_write_on_one_thread_and_read_on_another() {
        // Arrange
        let shared = LatestSharedData::new();

        // Act: write on worker thread.
        thread::scope(|scope| {
            let writer = scope.spawn(|| {
                shared.write(7.0);
                shared.write(8.0);
                shared.write(9.0);
            });
            writer.join().unwrap();
        });

        // Act: read on a different worker thread.
        let (first, second) = thread::scope(|scope| {
            let reader = scope.spawn(|| (shared.read(), shared.read()));
            reader.join().unwrap()
        });

        // Assert
        assert_eq!(first, Some(9.0));
        assert_eq!(second, None);
    }

    #[test]
    fn test_ordered_can_write_on_one_thread_and_read_on_another() {
        // Arrange
        let shared = OrderedSharedData::new();

        // Act: write on worker thread.
        thread::scope(|scope| {
            let writer = scope.spawn(|| {
                shared.write(7.0);
                shared.write(8.0);
                shared.write(9.0);
            });
            writer.join().unwrap();
        });

        // Act: read on a different worker thread.
        let (first, second, third, fourth) = thread::scope(|scope| {
            let reader = scope.spawn(|| (shared.read(), shared.read(), shared.read(), shared.read()));
            reader.join().unwrap()
        });

        // Assert
        assert_eq!(first, Some(7.0));
        assert_eq!(second, Some(8.0));
        assert_eq!(third, Some(9.0));
        assert_eq!(fourth, None);
    }

    #[test]
    fn test_latest_clear_wipes_data() {
        let shared = LatestSharedData::new();
        shared.write(42.0);
        assert_eq!(shared.read(), Some(42.0));

        shared.write(99.0);
        shared.clear();

        assert_eq!(shared.read(), None);
    }

    #[test]
    fn test_ordered_clear_wipes_data() {
        let shared = OrderedSharedData::new();
        shared.write(1.0);
        shared.write(2.0);
        shared.write(3.0);

        shared.clear();

        assert_eq!(shared.read(), None);

        shared.write(7.0);
        assert_eq!(shared.read(), Some(7.0));
        assert_eq!(shared.read(), None);
    }

    #[test]
    fn test_handles_special_f64_values_latest() {
        let shared = LatestSharedData::new();
        let special_values = [
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0001), // quiet NaN with payload
            f64::from_bits(0xfff8_0000_0000_0001), // negative quiet NaN payload
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::from_bits(1),          // smallest subnormal
            f64::from_bits(1u64 << 63), // negative zero
            0.0,
            -1.0,
            1.0,
            42.5,
        ];

        for value in special_values {
            shared.write(value);
            let actual = shared.read().expect("value should be present");
            assert_same_f64_bits(actual, value);
            assert_eq!(shared.read(), None);
        }
    }

    #[test]
    fn test_handles_special_f64_values_ordered() {
        let shared = OrderedSharedData::new();
        let special_values = [
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0001), // quiet NaN with payload
            f64::from_bits(0xfff8_0000_0000_0001), // negative quiet NaN payload
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::from_bits(1),          // smallest subnormal
            f64::from_bits(1u64 << 63), // negative zero
            0.0,
            -1.0,
            1.0,
            42.5,
        ];

        for value in special_values {
            shared.write(value);
        }

        for expected in special_values {
            let actual = shared.read().expect("value should be present");
            assert_same_f64_bits(actual, expected);
        }

        assert_eq!(shared.read(), None);
    }

    #[bench]
    fn bench_no_writes_ten_reads_latest(b: &mut Bencher) {
        b.iter(|| {
            let shared = LatestSharedData::new();
            for _ in 0..10 {
                black_box(shared.read());
            }
        });
    }

    #[bench]
    fn bench_no_writes_ten_reads_ordered(b: &mut Bencher) {
        b.iter(|| {
            let shared = OrderedSharedData::new();
            for _ in 0..10 {
                black_box(shared.read());
            }
        });
    }

    #[bench]
    fn bench_single_write_single_read_latest(b: &mut Bencher) {
        let input = black_box(1273811.0_f64);
        b.iter(|| {
            let shared = LatestSharedData::new();
            shared.write(input);
            black_box(shared.read());
        });
    }

    #[bench]
    fn bench_single_write_single_read_ordered(b: &mut Bencher) {
        let input = black_box(1273811.0_f64);
        b.iter(|| {
            let shared = OrderedSharedData::new();
            shared.write(input);
            black_box(shared.read());
        });
    }

    #[bench]
    fn bench_fifty_writes_single_read_latest(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(1222111357, 50));
        b.iter(|| {
            let shared = LatestSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }
            black_box(shared.read());
        });
    }

    #[bench]
    fn bench_fifty_writes_single_read_ordered(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(1222111357, 50));
        b.iter(|| {
            let shared = OrderedSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }
            black_box(shared.read());
        });
    }

    #[bench]
    fn bench_fifty_writes_read_until_empty_latest(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(45353413, 50));
        b.iter(|| {
            let shared = LatestSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }

            let mut reads = 0usize;
            while let Some(value) = shared.read() {
                black_box(value);
                reads += 1;
            }
            assert_eq!(reads, 1);
            black_box(reads);
        });
    }

    #[bench]
    fn bench_fifty_writes_read_until_empty_ordered(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(45353413, 50));
        b.iter(|| {
            let shared = OrderedSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }

            let mut reads = 0usize;
            while let Some(value) = shared.read() {
                black_box(value);
                reads += 1;
            }
            assert_eq!(reads, 50);
            black_box(reads);
        });
    }

    #[bench]
    fn bench_tenthousand_writes_read_until_empty_latest(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(99123123, 10_000));
        b.iter(|| {
            let shared = LatestSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }

            let mut reads = 0usize;
            while let Some(value) = shared.read() {
                black_box(value);
                reads += 1;
            }
            assert_eq!(reads, 1);
            black_box(reads);
        });
    }

    #[bench]
    fn bench_tenthousand_writes_read_until_empty_ordered(b: &mut Bencher) {
        let inputs = black_box(seeded_f64_samples(99123123, 10_000));
        b.iter(|| {
            let shared = OrderedSharedData::new();
            for &value in inputs.iter() {
                shared.write(value);
            }

            let mut reads = 0usize;
            while let Some(value) = shared.read() {
                black_box(value);
                reads += 1;
            }
            assert_eq!(reads, 10_000);

            black_box(reads);
        });
    }
}
