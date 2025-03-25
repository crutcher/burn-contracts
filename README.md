# Fluent Api for Burn Contract and Test Assertions

This crate provides a fluent api for contract and test assertions for the Burn framework.

```rust
use burn_contracts::assert_tensor;

let tensor: Tensor<B, 4 > = Tensor::new(& [10, 3, 32, 17]);
assert_tensor( & tensor)
.has_named_dims([('N', 10), ('C', 3), ('H', 32), ('W', 32)]);
// Panics:
// "Expected tensor to have dimensions [('N', 10), ('C', 3), ('H', 32), ('W', 32)] but got [(10, 3, 32, 17)]"
```

## Assertions

The following assertions are available:

### '.has_dims()'

```rust
fn example<B: Backend>() {
    let tensor = Tensor::<B, 4>::zeros([10, 3, 32, 17], &device);

    assert_tensor(&tensor).has_dims([10, 3, 32, 17]).unwrap();
}
```

### '.has_named_dims()'

```rust
fn example<B: Backend>() {
    let tensor = Tensor::<B, 4>::zeros([10, 3, 32, 17], &device);

    assert_tensor(&tensor).has_named_dims(
        [("N", 10), ("C", 3), ("H", 32), ("W", 17)],
    ).unwrap();
}
```

### '.unpacks_shape()'

```rust
fn example<B: Backend>() {
    let tensor = Tensor::<B, 6>::zeros([2, 2, 2, 5 * 4, 4 * 4, 3], &device);

    let [b, h, w] = assert_tensor(&tensor).unpacks_shape(
        ["b", "h", "w"],
        "b ... (h p) (w p) c",
        &[("p", 4), ("c", 3)],
    ).unwrap();

    assert_eq!(b, 2);
    assert_eq!(h, 5);
    assert_eq!(w, 4);
}
```

## Testing API

The "testing" feature enables the testing api; which provides expensive methods for testing
tensor contents.

To ensure that the testing api is not used in production, the testing api is only available
when the "testing" feature is enabled.

To enable the "testing" feature only for testing, add the following to your `Cargo.toml`:

```toml
[dependencies]
burn-contracts = $VERSION

[dev-dependencies]
burn-contracts = { version = $VERSION, features = ["testing"] }
```

## Change Log

### 0.2.0

Added `assert_tensor(&tensor).unpacks_shape(...)` assertion.

### 0.1.0

Initial release.