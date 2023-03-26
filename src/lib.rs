use crossbeam::channel::bounded;
use rayon::prelude::*;
use std::cmp;

pub fn single_thread_map<R, T, F>(values: Vec<T>, f: F) -> Vec<R>
where
    F: FnMut(T) -> R,
{
    values.into_iter().map(f).collect()
}

pub fn multithread_map_rayon<T, R, F>(input: Vec<T>, f: F, threshold: usize) -> Vec<R>
where
    T: Send + Sync + Copy,
    R: Send + Sync,
    F: Fn(T) -> R + Send + Sync + Copy,
{
    let input_len = input.len();
    if input_len < threshold {
        return single_thread_map(input, f);
    }
    input.into_par_iter().map(f).collect()
}

pub fn multithread_map_crossbeam_channel<T, R, F>(
    input: Vec<T>,
    f: F,
    threshold: usize,
) -> anyhow::Result<Vec<R>>
where
    T: Send + Sync + Clone,
    R: Send + Sync + Default + Clone,
    F: Fn(T) -> R + Send + Sync + Copy,
{
    let size = input.len();

    if size == 0 {
        return Ok(vec![]);
    }

    let num_threads = num_cpus::get() - 1; // one thread to collect results

    // this may be an incorrect treshold if second condition works
    // but this is the most convinient way to handle it
    if size <= threshold || size < num_threads {
        Ok(single_thread_map(input, f))
    } else {
        let mut results = vec![Default::default(); size];
        let chunk_size = size / num_threads;
        let (snd, rcv) = bounded(size);
        let res = crossbeam::thread::scope(|s| {
            for (i, chunk) in input.chunks(chunk_size).enumerate() {
                let snd_cloned = snd.clone();
                s.spawn(move |_| {
                    for (j, item) in chunk.iter().enumerate() {
                        snd_cloned
                            .send((i * chunk_size + j, f(item.clone())))
                            .unwrap();
                    }
                });
            }
            s.spawn(|_| {
                let mut counter = size;
                while counter > 0 {
                    let (index, value) = rcv.recv().unwrap();
                    results[index] = value;
                    counter -= 1;
                }
            });
        });
        res.map(|_| results)
            .map_err(|_| anyhow::anyhow!("One of the threads failed"))
    }
}

// run one more thread to process, collect results after join
pub fn multithread_map_crossbeam_fast<T, R, F>(
    input: Vec<T>,
    f: F,
    threshold: usize,
) -> anyhow::Result<Vec<R>>
where
    T: Send + Sync + Clone,
    R: Send + Sync + Default + Clone,
    F: Fn(T) -> R + Send + Sync + Copy,
{
    let size = input.len();

    if size == 0 {
        return Ok(vec![]);
    }

    let num_threads = num_cpus::get();

    // this may be an incorrect treshold if second condition works
    // but this is the most convinient way to handle it
    if size <= threshold || size < num_threads {
        Ok(single_thread_map(input, f))
    } else {
        let mut results = vec![Default::default(); size];
        let chunk_size = size / num_threads;
        let res = crossbeam::thread::scope(|s| {
            let (mut work_slice, mut rest) = results.split_at_mut(chunk_size);
            for chunk in input.chunks(chunk_size) {
                s.spawn(move |_| {
                    for (j, item) in chunk.iter().enumerate() {
                        work_slice[j] = f(item.clone());
                    }
                });
                if rest.is_empty() {
                    break;
                }
                (work_slice, rest) = rest.split_at_mut(cmp::min(chunk_size, rest.len()));
            }
        });
        res.map(|_| results)
            .map_err(|_| anyhow::anyhow!("One of the threads failed"))
    }
}

pub fn multithread_map_chat_gpt<'a, T, R, F>(vec: Vec<T>, f: F, threshold: usize) -> Vec<R>
where
    T: 'a + Send + Sync + Clone + 'static,
    R: Send + 'static,
    F: 'a + Send + Sync + Clone + Fn(T) -> R + 'static,
{
    let size = vec.len();
    if size <= threshold {
        vec.into_iter().map(f).collect()
    } else {
        let num_threads = num_cpus::get();
        let chunk_size = (size + num_threads - 1) / num_threads;
        let chunks = vec.chunks(chunk_size);

        let handles: Vec<_> = chunks
            .map(|chunk| {
                let f = f.clone();
                let chunk = chunk.to_vec();
                std::thread::spawn(move || chunk.into_iter().map(f).collect::<Vec<_>>())
            })
            .collect();

        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        multithread_map_chat_gpt, multithread_map_crossbeam_channel,
        multithread_map_crossbeam_fast, multithread_map_rayon, single_thread_map,
    };

    fn duplicate(x: i32) -> i32 {
        x * 2
    }

    #[test]
    fn test_small() {
        let input = vec![1, 2, 8];
        let expected = vec![2, 4, 16];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), expected);
        assert_eq!(
            multithread_map_crossbeam_channel(input.clone(), duplicate, 2).unwrap(),
            expected
        );
        assert_eq!(
            multithread_map_crossbeam_fast(input.clone(), duplicate, 2).unwrap(),
            expected
        );
        assert_eq!(
            multithread_map_chat_gpt(input.clone(), duplicate, 2),
            expected
        );
        assert_eq!(single_thread_map(input.clone(), duplicate), expected);
    }

    #[test]
    fn test_big() {
        let input = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let expected = vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), expected);
        assert_eq!(
            multithread_map_crossbeam_channel(input.clone(), duplicate, 2).unwrap(),
            expected
        );
        assert_eq!(
            multithread_map_crossbeam_fast(input.clone(), duplicate, 2).unwrap(),
            expected
        );
        assert_eq!(
            multithread_map_chat_gpt(input.clone(), duplicate, 2),
            expected
        );
        assert_eq!(single_thread_map(input.clone(), duplicate), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = vec![];
        assert_eq!(multithread_map_rayon(input.clone(), duplicate, 2), input);
        assert_eq!(
            multithread_map_crossbeam_channel(input.clone(), duplicate, 2).unwrap(),
            input
        );
        assert_eq!(
            multithread_map_crossbeam_fast(input.clone(), duplicate, 2).unwrap(),
            input
        );
        assert_eq!(multithread_map_chat_gpt(input.clone(), duplicate, 2), input);
        assert_eq!(single_thread_map(input.clone(), duplicate), input);
    }
}
