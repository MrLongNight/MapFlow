## 2024-05-22 - Graph Lookup Optimization
**Learning:** The borrow checker in Rust is sensitive to disjoint fields when using 'self' in methods. To allow calling a method while mutating a field, the method should be static (associated function) or take only necessary references instead of '&self'.
**Action:** When encountering borrow conflicts in struct methods, extract logic into static functions that take explicit references to fields.
