use std::{
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
