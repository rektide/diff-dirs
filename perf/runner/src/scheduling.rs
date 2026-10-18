use crate::Result;

/// Specified SplitMix64 shuffle, each permutation followed by its reverse.
pub fn schedule(count: usize, blocks: usize, mut seed: u64) -> Result<Vec<Vec<usize>>> {
    if count < 2 || blocks < 2 || !blocks.is_multiple_of(2) {
        return Err("need >=2 variants and positive even blocks".into());
    }
    let mut result = Vec::new();
    for _ in 0..blocks / 2 {
        let mut order: Vec<_> = (0..count).collect();
        for i in (1..count).rev() {
            seed = seed.wrapping_add(0x9e3779b97f4a7c15);
            let mut x = seed;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
            x ^= x >> 31;
            order.swap(i, (x % (i as u64 + 1)) as usize);
        }
        result.push(order.clone());
        order.reverse();
        result.push(order);
    }
    Ok(result)
}

#[test]
fn mirrored_seed_replays_and_balances_every_pair() {
    let order = schedule(4, 8, 1701).unwrap();
    assert_eq!(order, schedule(4, 8, 1701).unwrap());
    assert_ne!(order, schedule(4, 8, 1702).unwrap());
    for pair in order.as_chunks::<2>().0 {
        assert_eq!(pair[0].iter().rev().copied().collect::<Vec<_>>(), pair[1]);
    }
    assert!(schedule(1, 8, 0).is_err());
    assert!(schedule(2, 7, 0).is_err());
}
