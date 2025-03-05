#[cfg(any(test, feature = "testing"))]
pub mod testing;

use burn::prelude::{Backend, Float};
use burn::tensor::{BasicOps, Tensor};

pub struct TensorWrapper<'a, B, const D: usize, K = Float>
where
    B: Backend,
    K: BasicOps<B>,
{
    inner: &'a Tensor<B, D, K>,
}

/// Wrap a Tensor for test assertions.
pub fn assert_tensor<B, const D: usize, K>(tensor: &Tensor<B, D, K>) -> TensorWrapper<B, D, K>
where
    B: Backend,
    K: BasicOps<B>,
{
    TensorWrapper { inner: tensor }
}

impl<B, const D: usize, K> TensorWrapper<'_, B, D, K>
where
    B: Backend,
    K: BasicOps<B>,
{
    /// Assert that the wrapped tensor has the expected dimensions.
    ///
    /// ## Parameters
    ///
    /// - `dims`: The expected dimensions of the tensor.
    ///
    /// ## Panics
    ///
    /// Panics if the tensor does not have the expected dimensions.
    ///
    /// ## Example:
    /// ```
    /// use burn::backend::NdArray;
    /// use burn::tensor::Tensor;
    /// use burn_contracts::assert_tensor;
    ///
    /// let device = Default::default();
    /// let tensor = Tensor::<NdArray, 2>::from_data([[2., 3.], [4., 5.]], &device);
    ///
    /// assert_tensor(&tensor).has_dims([2, 2]);
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn has_dims(
        &self,
        dims: [usize; D],
    ) -> &Self {
        // Example assertion
        assert_eq!(
            self.inner.dims(),
            dims,
            "Expected tensor to have dimensions {:?}, but got {:?}",
            dims,
            self.inner.dims()
        );
        self
    }

    /// Assert that the wrapped tensor has the expected named dimensions.
    ///
    /// ## Parameters
    ///
    /// - `dims`: The expected named dimensions of the tensor.
    ///
    /// ## Panics
    ///
    /// Panics if the tensor does not have the expected named dimensions.
    ///
    /// ## Example:
    /// ```
    /// use burn::backend::NdArray;
    /// use burn::tensor::Tensor;
    /// use burn_contracts::assert_tensor;
    ///
    /// let device = Default::default();
    /// let tensor = Tensor::<NdArray, 2>::from_data([[2., 3.], [4., 5.]], &device);
    ///
    /// assert_tensor(&tensor).has_named_dims([("rows", 2), ("cols", 2)]);
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn has_named_dims(
        &self,
        dims: [(&str, usize); D],
    ) -> &Self {
        if self
            .inner
            .dims()
            .iter()
            .zip(dims.iter())
            .all(|(&a, &(_, b))| a == b)
        {
            return self;
        }

        let actual = self
            .inner
            .dims()
            .iter()
            .zip(dims.iter())
            .map(|(&d, &(n, _))| format!("{n}={d}"))
            .collect::<Vec<String>>()
            .join(", ");

        let expected = dims
            .iter()
            .map(|&(n, d)| format!("{n}={d}"))
            .collect::<Vec<String>>()
            .join(", ");

        panic!("Expected dims [{expected}], found [{actual}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::backend::NdArray;
    use burn::prelude::Backend;
    use burn::tensor::Tensor;

    #[test]
    fn test_has_dims_passing() {
        impl_has_dims_passing::<NdArray>();
    }

    fn impl_has_dims_passing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2.], [3.]], &device);

        assert_tensor(&tensor).has_dims([2, 1]);
    }

    #[test]
    #[should_panic(expected = "Expected tensor to have dimensions [1, 2], but got [2, 1]")]
    fn test_has_dims_failing() {
        impl_has_dims_failing::<NdArray>();
    }

    fn impl_has_dims_failing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2.], [3.]], &device);

        assert_tensor(&tensor).has_dims([1, 2]);
    }

    #[test]
    fn test_has_named_dims_passing() {
        impl_has_named_dims_passing::<NdArray>();
    }

    fn impl_has_named_dims_passing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2.], [3.]], &device);

        assert_tensor(&tensor).has_named_dims([("rows", 2), ("cols", 1)]);
    }

    #[test]
    #[should_panic(expected = "Expected dims [rows=1, cols=2], found [rows=2, cols=1]")]
    fn test_has_named_dims_failing() {
        impl_has_named_dims_failing::<NdArray>();
    }

    fn impl_has_named_dims_failing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2.], [3.]], &device);

        assert_tensor(&tensor).has_named_dims([("rows", 1), ("cols", 2)]);
    }
}
