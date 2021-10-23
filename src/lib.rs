use std::{
    convert::TryInto,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use glib::translate::*;

#[derive(Debug)]
pub struct Array<T> {
    garray: glib::Array,
    _marker: PhantomData<[T]>,
}

impl<T> Array<T> {
    pub unsafe fn new(garray: glib::Array) -> Array<T> {
        assert_eq!(
            std::mem::size_of::<T>(),
            garray.element_size(),
            "Array elements are not the correct size",
        );
        Self {
            garray,
            _marker: PhantomData,
        }
    }

    pub fn from_slice_none<'a, U>(slice: &'a [U]) -> (Self, Vec<Stash<'a, T, U>>)
    where
        T: Copy + Ptr,
        U: 'a + ToGlibPtr<'a, T>,
    {
        let len = slice
            .len()
            .try_into()
            .expect("Vec is too long to fit into a GArray");
        // Create a new GArray of the correct size
        let ptr = unsafe {
            glib::ffi::g_array_sized_new(
                false.into(),
                false.into(),
                std::mem::size_of::<T>()
                    .try_into()
                    .expect("Pointer is too large to fit into a GArray"),
                len,
            )
        };
        // Convert Rust types to glib types
        // This Vec is the backing storage for the glib types and the pointers to them
        let stashes: Vec<_> = slice.iter().map(|i| i.to_glib_none()).collect();
        // Add all the elements
        for elem in &stashes {
            unsafe {
                glib::ffi::g_array_append_vals(ptr, elem.0.to(), 1);
            }
        }
        // Convert to the Rust type
        assert!(!ptr.is_null(), "Out of memory");
        let array: glib::Array = unsafe { from_glib_full(ptr) };
        (unsafe { Self::new(array) }, stashes)
    }

    pub unsafe fn as_slice(&self) -> &[T] {
        std::slice::from_raw_parts(self.garray.data() as _, self.garray.len())
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        std::slice::from_raw_parts_mut(self.garray.data() as _, self.garray.len())
    }

    pub unsafe fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        Vec::from(self.as_slice())
    }

    pub unsafe fn as_vec<U>(&self) -> Vec<U>
    where
        T: Copy,
        U: FromGlib<T>,
    {
        self.as_slice()
            .iter()
            .copied()
            .map(|i| unsafe { from_glib(i) })
            .collect()
    }

    pub unsafe fn as_vec_full<U>(&self) -> Vec<U>
    where
        T: Clone + Ptr,
        U: FromGlibPtrFull<T>,
    {
        self.as_slice()
            .iter()
            .cloned()
            .map(|i| unsafe { from_glib_full(i) })
            .collect()
    }

    pub unsafe fn as_vec_none<U>(&self) -> Vec<U>
    where
        T: Clone + Ptr,
        U: FromGlibPtrNone<T>,
    {
        self.as_slice()
            .iter()
            .cloned()
            .map(|i| unsafe { from_glib_none(i) })
            .collect()
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { self.as_slice() }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.as_slice_mut() }
    }
}

// Note: T must be Copy because glib will just blindly copy the bytes
impl<T: Copy> From<Vec<T>> for Array<T> {
    fn from(vec: Vec<T>) -> Self {
        let len = vec
            .len()
            .try_into()
            .expect("Vec is too long to fit into a GArray");
        // Create a new GArray of the correct size
        let ptr = unsafe {
            glib::ffi::g_array_sized_new(
                false.into(),
                false.into(),
                std::mem::size_of::<T>()
                    .try_into()
                    .expect("Type is too large to fit into a GArray"),
                len,
            )
        };
        // Add all the elements
        if len != 0 {
            unsafe {
                glib::ffi::g_array_append_vals(ptr, vec.as_ptr().cast(), len);
            }
        }
        // Convert to the Rust type
        assert!(!ptr.is_null(), "Out of memory");
        let array: glib::Array = unsafe { from_glib_full(ptr) };
        unsafe { Self::new(array) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_from_vec() {
        let vec = vec![1, 2, 3];
        let array = Array::from(vec);
        let slice = unsafe { array.as_slice() };
        assert_eq!(slice.len(), 3);
        assert_eq!(slice, &[1, 2, 3]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
    }
}
