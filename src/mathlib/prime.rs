pub struct Prime;

impl Prime {
    /// エラトステネスの篩: 0..=limit の素数フラグを返す
    pub fn sieve(limit: u32) -> Vec<bool> {
        let n = limit as usize;
        let mut is_prime = vec![true; n + 1];

        is_prime[0] = false;
        if n >= 1 {
            is_prime[1] = false;
        }

        let mut p = 2_usize;
        while p * p <= n {
            if is_prime[p] {
                let mut k = p * p;
                while k <= n {
                    is_prime[k] = false;
                    k += p;
                }
            }
            p += 1;
        }
        is_prime
    }
}

pub struct PrimeTable {
    is_prime: Vec<bool>,
    limit: u32,
}

impl PrimeTable {
    pub fn new(limit: u32) -> Self {
        Self {
            is_prime: Prime::sieve(limit),
            limit,
        }
    }

    /// 偶数 n のゴールドバッハ分割数 g(n)
    /// p + q = n（p, q は素数、p<=q）を数える（順序は数えない）
    pub fn goldbach_pairs_count(&self, n: u32) -> u32 {
        assert!(
            n.is_multiple_of(2),
            "goldbach_pairs_count expects an even n, got {}",
            n
        );
        assert!(
            n <= self.limit,
            "goldbach_pairs_count expects n <= limit ({}), got {}",
            self.limit,
            n
        );
        let half = n / 2; // p<=q を満たすため n/2 まで探索
        let mut count = 0u32;
        for p in 2..=half {
            if self.is_prime[p as usize] {
                let q = n - p; // q>=p が保証される
                if self.is_prime[q as usize] {
                    count += 1;
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::{Prime, PrimeTable};

    #[test]
    fn prime_sieve_marks_primes() {
        let is_prime = Prime::sieve(100);
        let primes = [
            2_u32, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79,
            83, 89, 97,
        ];
        let mut expected = [false; 101];
        for &p in &primes {
            expected[p as usize] = true;
        }
        for n in 0..=100 {
            assert_eq!(
                is_prime[n], expected[n],
                "mismatch at {} (expected {} got {})",
                n, expected[n], is_prime[n]
            );
        }
    }

    #[test]
    fn prime_table_goldbach_pairs_count() {
        let table = PrimeTable::new(20);
        assert_eq!(table.goldbach_pairs_count(4), 1); // 2+2
        assert_eq!(table.goldbach_pairs_count(6), 1); // 3+3
        assert_eq!(table.goldbach_pairs_count(8), 1); // 3+5
        assert_eq!(table.goldbach_pairs_count(10), 2); // 3+7, 5+5
        assert_eq!(table.goldbach_pairs_count(12), 1); // 5+7
    }

    #[test]
    #[should_panic]
    fn prime_table_panics_on_odd() {
        let table = PrimeTable::new(20);
        table.goldbach_pairs_count(9);
    }

    #[test]
    #[should_panic]
    fn prime_table_panics_on_out_of_range() {
        let table = PrimeTable::new(20);
        table.goldbach_pairs_count(22);
    }
}
