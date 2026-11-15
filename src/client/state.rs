pub trait State<T> {
    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R;
    fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}