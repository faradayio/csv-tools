//! Unpacking vectors.

use failure::format_err;

use crate::Result;

/// Given a vector `input`, an expected output vector length `output_len` and a
/// function `idx_fn` that maps input values to output indices, assemble an
/// output vector with each element at the right index, or `None` if no element
/// is found for an index.
pub fn unpack_vec<T, F>(
    input: Vec<T>,
    output_len: usize,
    idx_fn: F,
) -> Result<Vec<Option<T>>>
where
    F: Fn(&T) -> usize,
    T: Clone,
{
    let mut output = vec![None; output_len];
    for value in input {
        let idx = idx_fn(&value);
        if idx >= output_len {
            return Err(format_err!(
                "output has length {}, but found index {}",
                output_len,
                idx,
            ));
        } else if output[idx].is_some() {
            return Err(format_err!("index {} appears twice", idx));
        }
        output[idx] = Some(value);
    }
    Ok(output)
}

#[test]
fn unpack_to_correct_index() {
    assert_eq!(
        unpack_vec(vec![2, 4, 5], 7, |v| *v).unwrap(),
        vec![None, None, Some(2), None, Some(4), Some(5), None],
    );
}

#[test]
fn error_if_output_too_short() {
    assert!(unpack_vec(vec![1], 1, |v| *v).is_err());
}

#[test]
fn error_on_duplicate_indices() {
    assert!(unpack_vec(vec![0, 0], 2, |v| *v).is_err());
}
