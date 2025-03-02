# Fluent Api for Burn Contract and Test Assertions

This crate provides a fluent api for contract and test assertions for the Burn framework.

```rust
let tensor: Tensor<B, 4> = Tensor::new(&[10, 3, 32, 17]);
burn_contracts::assert_tensor(&tensor)
    .has_named_dims([('N', 10), ('C', 3), ('H', 32), ('W', 32)]);
// Panics:
// "Expected tensor to have dimensions [('N', 10), ('C', 3), ('H', 32), ('W', 32)] but got [(10, 3, 32, 17)]"
```
