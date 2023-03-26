# Comments

I followed the assumption that we need not just to calculate results, but also preserve the order from the original vector.

`rayon` implementation is something to easy and available available out of the box, to be an expected solutions. So I made some more implementations and benchmarked them.
The main one to be considered is `multithread_map_crossbeam_fast`, only the `rayon` one beats it.
