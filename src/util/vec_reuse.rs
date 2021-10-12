

// Re-claim old spaces, there is probably a library for this but I don't know what it is called so whatever.
//
// Keeps position of old instances while being able to mark indices as unused so that future inserts can use them.
#[derive(Debug, Clone)]
pub struct ReuseVec<T> {
    current: Vec<T>,
    reclaim: Vec<usize>,
}

impl<T> ReuseVec<T> {
    pub fn new() -> ReuseVec<T> {
        ReuseVec {
            current: Vec::new(),
            reclaim: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        if self.reclaim.len() >= 1 {
            let index = self.reclaim.swap_remove(0);
            self.current[index] = item;
        } else {
            self.current.push(item);
        }
    }

    pub fn mark_reclaim(&mut self, index: usize) {
        self.reclaim.push(index);
    }

    pub fn iter<'a>(&'a self) -> ReuseVecIter<'a, T> {
        ReuseVecIter {
            index: 0,
            reuse_vec: self,
        }
    }
}

pub struct ReuseVecIter<'a, T> {
    index: usize,
    reuse_vec: &'a ReuseVec<T>,
}

impl<'a, T> Iterator for ReuseVecIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.reuse_vec.current.get(self.index);
        self.index += 1;
        current
    }
}

impl<T> IntoIterator for ReuseVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.current.into_iter()
    }
}

#[cfg(test)]
mod test {
    use crate::util::ReuseVec;

    #[test]
    fn sanity() {

        let mut rvec = ReuseVec::<u32>::new();

        rvec.push(1);
        rvec.push(2);
        rvec.push(3);
        assert_eq!(rvec.current.as_slice(), &[1, 2, 3]);
        assert_eq!(rvec.reclaim.len(), 0);

        rvec.mark_reclaim(1);
        assert_eq!(rvec.current.as_slice(), &[1, 2, 3]);
        assert_eq!(rvec.reclaim.as_slice(), &[1]);

        rvec.push(4);
        assert_eq!(rvec.current.as_slice(), &[1, 4, 3]);
        assert_eq!(rvec.reclaim.len(), 0);

        rvec.push(5);
        assert_eq!(rvec.current.as_slice(), &[1, 4, 3, 5]);
        assert_eq!(rvec.reclaim.len(), 0);
    }
}