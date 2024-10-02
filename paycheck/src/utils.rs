#[macro_export]
macro_rules! paycheck_seeds {
    ($whirlpool:expr, $creator:expr, $a_to_b:expr) => {
        &[
            b"paycheck",
            &$whirlpool.to_bytes(),
            &$creator.to_bytes(),
            &[$a_to_b as u8],
        ]
    };
}

// with bump
#[macro_export]
macro_rules! paycheck_seeds_with_bump {
    ($whirlpool:expr, $creator:expr, $a_to_b:expr, $bump:expr) => {
        &[
            b"paycheck",
            &$whirlpool.to_bytes(),
            &$creator.to_bytes(),
            &[$a_to_b as u8],
            &[$bump],
        ]
    };
}
