macro_rules! ensure_in_range {
    ($value:expr, $min:expr => $max:expr) => {
        #[allow(clippy::manual_range_contains)]
        if $value > $max || $value < $min {
            return Err(crate::Error::OutOfRange);
        }
    };

    ($value:ident, $max:expr) => {
        if $value > $max {
            return Err(crate::Error::OutOfRange);
        }
    };
}

pub(crate) use ensure_in_range;

/// Computes the quotient and remainder using truncating division.
///
/// Equivalent to `(lhs / rhs, lhs % rhs)`.
macro_rules! divrem {
    ($lhs:expr, $rhs:expr) => {{
        let lhs = $lhs;
        let rhs = $rhs;
        (lhs / rhs, lhs % rhs)
    }};
}

/// Returns the quotient and remainder using Euclidean division.
///
/// This is similar to Python's `divmod` function.
macro_rules! divmod {
    ($lhs:expr, $rhs:expr) => {{
        let (lhs, rhs) = ($lhs, $rhs);
        (lhs.div_euclid(rhs), lhs.rem_euclid(rhs))
    }};
}

pub(crate) use divmod;
pub(crate) use divrem;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divrem() {
        assert_eq!(divrem!(-23, 12), (-1, -11));
        assert_eq!(divrem!(27, 12), (2, 3));
        assert_eq!(divrem!(23, -12), (-1, 11));
    }
}
