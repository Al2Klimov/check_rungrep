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
