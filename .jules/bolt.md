## 2024-05-24 - [Avoid Intermediate HashSets for Small Lookups]
**Learning:** In Rust loops, mapping an array of values into a `HashSet` to perform an `O(1)` `.contains()` check is an anti-pattern when the data set is small and the loop runs frequently (like in media orchestration ticks). The allocation overhead and hashing cost far outweigh a simple `O(N)` linear `.find()`.
**Action:** Avoid temporary `HashSet` allocations for small cross-referencing tasks in hot loops; rely on direct iterator methods instead. Also ensure ownership transfer is favored over `.clone()` when the source variable is no longer needed.

## 2025-03-25 - [Redundant String Allocation in Loops]
**Learning:** Found a common pattern in UI filtering where `String::to_lowercase()` was called inside a `.filter()` iterator closure. This causes O(N) heap allocations during search operations, degrading UI performance with large media libraries.
**Action:** Always hoist string transformations like `to_lowercase()` outside of iterator closures when filtering static query data.
