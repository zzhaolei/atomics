use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;

struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> Guard<'_, T> {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }

        Guard { lock: self }
    }
}

struct Guard<'lock, T> {
    lock: &'lock SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

fn main() {
    let spin: SpinLock<Vec<i32>> = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| {
            spin.lock().push(1);
        });
        s.spawn(|| {
            let mut g = spin.lock();
            g.push(2);
            g.push(3);
        });
    });

    let spin = spin.lock();
    assert_eq!(spin.as_slice(), [1, 2, 3]);
}
