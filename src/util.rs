pub trait VecSliceCompare<T: PartialEq> {
    fn compare(&self, s: &[T]) -> bool;
}

impl<T: PartialEq> VecSliceCompare<T> for Vec<T> {
    fn compare(&self, s: &[T]) -> bool {
        if self.len() != s.len() {
            return false;
        }
        for i in 0..self.len() {
            if self[i] != s[i] {
                return false;
            }
        }
        return true;
    }
}

pub trait Stack<T> {
    fn offer(&mut self, t: T);
    fn take(&mut self) -> Option<T>;
}

impl<T> Stack<T> for Vec<T> {
    fn offer(&mut self, t: T) {
        self.insert(0, t);
    }

    fn take(&mut self) -> Option<T> {
        Some(self.remove(0))
    }
}

pub trait BinarySearch<T: Sized> {
    fn binary_search(&self, accept: Box<Fn(&T) -> isize>) -> Option<usize>;
}

impl<T: Sized> BinarySearch<T> for Vec<T> {
    fn binary_search(&self, accept: Box<Fn(&T) -> isize>) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let mut low = 0usize;
        let mut high = self.len() - 1;
        while low <= high {
            let mid: usize = (high + low) / 2;
            let r = accept(&self[mid]);
            if r == 0 {
                return Some(mid);
            } else if r > 0 {// && mid != 0
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }
        return None;
    }
}


#[test]
fn test_binary_search() {
    let arr = vec![1, 3, 5, 9, 10];
    let rst = arr.binary_search(Box::new(move |i| -> isize{
        let offset = 10;
        if offset < *i {
            return 1;
        } else if offset > *i {
            return -1;
        }
        return 0;
    }));
    assert_eq!(rst, Some(4));
}

