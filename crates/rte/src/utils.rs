use std::{
    ffi::CString,
    ops::{Deref, DerefMut},
};

pub trait AsCString {
    fn as_c_str(&self) -> CString;
}

impl<T> AsCString for T
where
    T: AsRef<str>,
{
    fn as_c_str(&self) -> CString {
        let mut v = self.as_ref().as_bytes().to_owned();
        v.push(0);
        unsafe { CString::from_vec_unchecked(v) }
    }
}

pub trait Raw<T>: Deref<Target = T> + DerefMut + AsRaw<Raw = T> + FromRaw + From<*mut T> {}

pub trait AsRaw {
    type Raw;

    fn as_raw(&self) -> *mut Self::Raw;
}

pub trait FromRaw: AsRaw
where
    Self: Sized,
{
    fn from_raw(raw: *mut Self::Raw) -> Option<Self>;
}

macro_rules! raw {
    (
        $(#[$outer:meta])*
        pub $wrapper:ident ( $raw_ty:ty )
    ) => {
        #[repr(transparent)]
        #[derive(Debug)]
        pub struct $wrapper(::std::ptr::NonNull<$raw_ty>);

        impl utils::Raw<$raw_ty> for $wrapper {}

        impl ::std::ops::Deref for $wrapper {
            type Target = $raw_ty;

            fn deref(&self) -> &Self::Target {
                unsafe { self.0.as_ref() }
            }
        }

        impl ::std::ops::DerefMut for $wrapper {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { self.0.as_mut() }
            }
        }

        impl utils::AsRaw for $wrapper {
            type Raw = $raw_ty;

            #[inline]
            fn as_raw(&self) -> *mut Self::Raw {
                self.0.as_ptr()
            }
        }

        impl utils::FromRaw for $wrapper {
            fn from_raw(raw: *mut Self::Raw) -> Option<Self> {
                ::std::ptr::NonNull::new(raw).map($wrapper)
            }
        }

        impl From<*mut $raw_ty> for $wrapper {
            fn from(p: *mut $raw_ty) -> Self {
                use super::utils::FromRaw;

                Self::from_raw(p).unwrap()
            }
        }
    };
}
