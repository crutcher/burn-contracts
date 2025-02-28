/// Testing utilities for Burn contracts.
///
/// These utilities are useful for writing unittests; but generally bad-practice
/// for use in production code.
use crate::TensorWrapper;
use burn::prelude::{Backend, Tensor};
use burn::tensor::{BasicOps, Numeric};

impl<B, const D: usize, K> TensorWrapper<'_, B, D, K>
where
    B: Backend,
    K: BasicOps<B>,
{
    pub fn equals(
        &self,
        expected: &Tensor<B, D, K>,
    ) -> &Self {
        self.has_dims(expected.dims());

        assert_eq!(
            self.inner.to_data(),
            expected.to_data(),
            "Expected tensor to have data {:?}, but got {:?}",
            expected.to_data(),
            self.inner.to_data()
        );

        self
    }
}

const DEFAULT_ATOL: f64 = 1e-8;
const DEFAULT_RTOL: f64 = 1e-5;

impl<B, const D: usize, K> TensorWrapper<'_, B, D, K>
where
    B: Backend,
    K: BasicOps<B> + Numeric<B>,
{
    /// Assert that the wrapped tensor is close to the expected tensor.
    ///
    /// The `atol` and `rtol` parameters are optional and default to `DEFAULT_ATOL` and `DEFAULT_RTOL` respectively.
    pub fn is_close(
        &self,
        expected: &Tensor<B, D, K>,
        atol: Option<f64>,
        rtol: Option<f64>,
    ) -> &Self {
        self.has_dims(expected.dims());

        // reference implementation:
        // - Tensor::is_close()
        // - burn::tensor::check_closeness()
        // TODO(crutcher): see close_to(); maybe promote defaults to public.
        let atol = atol.unwrap_or(DEFAULT_ATOL);
        let rtol = rtol.unwrap_or(DEFAULT_RTOL);

        let close = self
            .inner
            .clone()
            .is_close(expected.clone(), Some(atol), Some(rtol));

        let data = close.clone().into_data();
        let num_elements = data.num_elements();

        // Count the number of elements that are close (true)
        let count = data.iter::<bool>().filter(|x| *x).count();

        if count != num_elements {
            let percentage = (count as f64 / num_elements as f64) * 100.0;

            panic!(
                "Expected tensor to be within (atol={:?}, rtol={:?}) of target\n\
                 - {}/{} ({:.2}%) elements passed",
                atol, rtol, count, num_elements, percentage,
            );
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_tensor;
    use burn::backend::NdArray;
    use burn::prelude::Backend;
    use burn::tensor::Tensor;

    #[test]
    fn test_is_close_passing() {
        impl_is_close_passing::<NdArray>();
    }

    fn impl_is_close_passing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2., 3.], [4., 5.]], &device);

        let dims = tensor.dims();
        assert_eq!(dims, [2, 2]);

        assert_tensor(&tensor).is_close(&tensor, None, None);
    }

    #[test]
    #[should_panic(
        expected = "Expected tensor to be within (atol=1e-8, rtol=1e-5) of target\n\
                    - 3/4 (75.00%) elements passed"
    )]
    fn test_is_close_failing() {
        impl_is_close_failing::<NdArray>();
    }

    fn impl_is_close_failing<B: Backend>() {
        let device = Default::default();
        let tensor = Tensor::<B, 2>::from_data([[2., 3.], [4., 5.]], &device);
        let other = Tensor::<B, 2>::from_data([[2., 3.], [4., 6.]], &device);

        let dims = tensor.dims();
        assert_eq!(dims, [2, 2]);

        assert_tensor(&tensor).is_close(&other, None, None);
    }
}
