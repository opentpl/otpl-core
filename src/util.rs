
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

pub trait Queue<T> {
    fn offer(&mut self, t: T);
    fn take(&mut self) -> Option<T>;
}

impl<T> Queue<T> for Vec<T> {
    fn offer(&mut self, t: T) {
        self.insert(0, t);
    }

    fn take(&mut self) -> Option<T> {
        self.pop()
    }
}