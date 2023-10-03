# Octree
- Tree-based structure where each node has 8 children (octants)
- Used in computer graphics!

## Structure
```rust
struct Octree<T> {
    data: T,
    children: [Octree; 8],
}
```

![](Octree.png)
