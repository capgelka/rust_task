use rayon::prelude::*;

pub fn single_thread_map<R, T, F>(values: Vec<T>, f: F) -> Vec<R>
where
    F: Fn(T) -> R,
{
    values.into_iter().map(f).collect()
}

pub fn multithread_map_rayon<T, R, F>(input: Vec<T>, f: F, threshold: usize) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    let input_len = input.len();
    if input_len < threshold {
        return single_thread_map(input, f);
    }
    input.into_par_iter().map(f).collect()
}

#[cfg(test)]
mod tests {
    use crate::{multithread_map_rayon, single_thread_map};

    fn duplicate(x: i32) -> i32 {
        x * 2
    }

    #[test]
    fn test_small() {
        let input = vec![1, 2, 8];
        let expected = vec![2, 4, 16];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), expected);
        assert_eq!(single_thread_map(input.clone(), duplicate), expected);
    }

    #[test]
    fn test_big() {
        let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), expected);
        assert_eq!(single_thread_map(input.clone(), duplicate), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), input);
        assert_eq!(single_thread_map(input.clone(), duplicate), input);
    }
}
