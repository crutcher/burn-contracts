use std::collections::HashMap;
use std::iter;
use std::slice;

/// A trait for types that can be used as a read-only source of shape binding references.
///
///
/// ## Example
///
/// The following works with:
///  - `&[(T, usize)] where T: AsRef<str>`
///  - `&[(T, usize); N] where T: AsRef<str>, const N: usize`
///  - `&Vec<(T, usize)> where T : AsRef<str>`
///  - `&HashMap<String, usize>`
///
/// ```rust
/// use burn_contracts::shapes::bindings::{ShapeBindingSource, collect_binding_map};
///
/// fn example<T: ShapeBindingSource>(bindings: T) {
///    let map = collect_binding_map(bindings);
///   // ...
/// }
/// ```
pub trait ShapeBindingSource {
    type Iter<'a>: Iterator<Item = (&'a str, usize)>
    where
        Self: 'a;

    /// Returns an iterator over the shape bindings.
    ///
    /// The iterator yields a tuple of the shape binding name and its index.
    fn for_each_shape_binding(&self) -> Self::Iter<'_>;

    /// Looks up the value of a shape binding by name.
    ///
    /// Returns `None` if the shape binding is not found.
    fn lookup_shape_binding(
        &self,
        name: &str,
    ) -> Option<usize> {
        self.for_each_shape_binding()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v)
    }
}

impl<T> ShapeBindingSource for &[(T, usize)]
where
    T: AsRef<str>,
{
    type Iter<'a>
        = iter::Map<slice::Iter<'a, (T, usize)>, fn(&'a (T, usize)) -> (&'a str, usize)>
    where
        Self: 'a;

    fn for_each_shape_binding(&self) -> Self::Iter<'_> {
        self.iter().map(|(k, v)| (k.as_ref(), *v))
    }
}

impl<const N: usize, T> ShapeBindingSource for &[(T, usize); N]
where
    T: AsRef<str>,
{
    type Iter<'a>
        = iter::Map<slice::Iter<'a, (T, usize)>, fn(&'a (T, usize)) -> (&'a str, usize)>
    where
        Self: 'a;

    fn for_each_shape_binding(&self) -> Self::Iter<'_> {
        self.iter().map(|(k, v)| (k.as_ref(), *v))
    }
}

impl<T> ShapeBindingSource for &Vec<(T, usize)>
where
    T: AsRef<str>,
{
    type Iter<'a>
        = iter::Map<slice::Iter<'a, (T, usize)>, fn(&'a (T, usize)) -> (&'a str, usize)>
    where
        Self: 'a;

    fn for_each_shape_binding(&self) -> Self::Iter<'_> {
        self.iter().map(|(k, v)| (k.as_ref(), *v))
    }
}

impl<S: ::std::hash::BuildHasher> ShapeBindingSource for &HashMap<String, usize, S> {
    type Iter<'a>
        = iter::Map<
        std::collections::hash_map::Iter<'a, String, usize>,
        fn((&'a String, &'a usize)) -> (&'a str, usize),
    >
    where
        Self: 'a;

    fn for_each_shape_binding(&self) -> Self::Iter<'_> {
        self.iter().map(|(k, v)| (k.as_ref(), *v))
    }

    fn lookup_shape_binding(
        &self,
        name: &str,
    ) -> Option<usize> {
        self.get(name).copied()
    }
}

/// Collects the shape bindings into a `HashMap<String, usize>`.
pub fn collect_binding_map<T: ShapeBindingSource>(bindings: T) -> HashMap<String, usize> {
    bindings
        .for_each_shape_binding()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

/// Collects the shape bindings into a sorted list of `(name, index)` pairs.
pub fn collect_sorted_binding_list<T: ShapeBindingSource>(bindings: T) -> Vec<(String, usize)> {
    let mut items: Vec<(&str, usize)> = bindings.for_each_shape_binding().collect();
    items.sort_unstable();
    items.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// Looks up the value of a shape binding by name.
pub fn lookup_binding<T: ShapeBindingSource>(
    bindings: T,
    name: &str,
) -> Option<usize> {
    bindings.lookup_shape_binding(name)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_collect_binding_map() {
        let source: [(&str, usize); 2] = [("a", 1), ("b", 2)];

        let map = collect_binding_map(&source);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.get("x"), None);
    }

    #[test]
    fn test_from_str_array() {
        let source: [(&str, usize); 2] = [("a", 1), ("b", 2)];

        let pairs = collect_sorted_binding_list(&source);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);

        assert_eq!(lookup_binding(&source, "a"), Some(1));
        assert_eq!(lookup_binding(&source, "x"), None);

        let pairs = collect_sorted_binding_list(&source[..]);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);

        assert_eq!(lookup_binding(&source[..], "a"), Some(1));
        assert_eq!(lookup_binding(&source[..], "x"), None);
    }

    #[test]
    fn test_from_string_array() {
        let source: [(String, usize); 2] = [("a".to_string(), 1), ("b".to_string(), 2)];

        // As array reference.
        let pairs = collect_sorted_binding_list(&source);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);

        assert_eq!(lookup_binding(&source, "a"), Some(1));
        assert_eq!(lookup_binding(&source, "x"), None);

        // As slice reference.
        let pairs = collect_sorted_binding_list(&source[..]);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);

        assert_eq!(lookup_binding(&source[..], "a"), Some(1));
        assert_eq!(lookup_binding(&source[..], "x"), None);
    }

    #[test]
    fn test_from_string_vec() {
        let source: Vec<(String, usize)> = vec![("a".to_string(), 1), ("b".to_string(), 2)];

        let pairs = collect_sorted_binding_list(&source);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);
        assert_eq!(lookup_binding(&source, "a"), Some(1));

        assert_eq!(lookup_binding(&source, "x"), None);
    }

    #[test]
    fn test_from_hashmap() {
        let mut source: HashMap<String, usize> = Default::default();
        source.insert("a".to_string(), 1);
        source.insert("b".to_string(), 2);

        let pairs = collect_sorted_binding_list(&source);

        assert_eq!(pairs, vec![("a".to_string(), 1), ("b".to_string(), 2)]);

        assert_eq!(lookup_binding(&source, "a"), Some(1));
        assert_eq!(lookup_binding(&source, "x"), None);
    }
}
