pub(crate) struct CounterIterator<T, I>
where
    I: Iterator<Item = T>,
{
    inner: I,
    taken: usize,
}

impl<T, I> CounterIterator<T, I>
where
    I: Iterator<Item = T>,
{
    pub(crate) fn new(it: I) -> Self {
        Self {
            inner: it,
            taken: 0,
        }
    }

    pub(crate) fn taken(&self) -> usize {
        self.taken
    }
}

impl<T, I> Iterator for CounterIterator<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.inner.next();
        if ret.is_some() {
            self.taken += 1;
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taken_starts_at_zero() {
        let ci = CounterIterator::new(vec![1, 2, 3].into_iter());
        assert_eq!(ci.taken(), 0);
    }

    #[test]
    fn test_empty_iterator() {
        let mut ci = CounterIterator::new(Vec::<i32>::new().into_iter());
        assert_eq!(ci.next(), None);
        assert_eq!(ci.taken(), 0);
    }

    #[test]
    fn test_full_consumption() {
        let mut ci = CounterIterator::new(vec![10, 20, 30].into_iter());
        assert_eq!(ci.next(), Some(10));
        assert_eq!(ci.next(), Some(20));
        assert_eq!(ci.next(), Some(30));
        assert_eq!(ci.next(), None);
        assert_eq!(ci.taken(), 3);
    }

    #[test]
    fn test_partial_consumption() {
        let mut ci = CounterIterator::new(vec![10, 20, 30].into_iter());
        assert_eq!(ci.next(), Some(10));
        assert_eq!(ci.taken(), 1);
        assert_eq!(ci.next(), Some(20));
        assert_eq!(ci.taken(), 2);
    }
}
