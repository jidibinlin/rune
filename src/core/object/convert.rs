//! This module holds implementation of conversion functions. Much of
//! this code could be replaced with macros or specialized generics if
//! those are ever stabalized.

use super::{
    super::error::{ArgError, Type, TypeError},
    nil, qtrue, LispHashTable, LispString, LispVec,
};
use super::{Gc, Object};
use super::{GcObj, LispFloat};
use crate::core::env::Symbol;
use anyhow::Context;

impl<'ob> TryFrom<GcObj<'ob>> for &'ob str {
    type Error = anyhow::Error;
    fn try_from(obj: GcObj<'ob>) -> Result<Self, Self::Error> {
        match obj.untag() {
            Object::String(x) => x.try_into(),
            x => Err(TypeError::new(Type::String, x).into()),
        }
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for Option<&'ob str> {
    type Error = anyhow::Error;
    fn try_from(obj: GcObj<'ob>) -> Result<Self, Self::Error> {
        match obj.untag() {
            Object::NIL => Ok(None),
            Object::String(x) => Ok(Some(x.try_into()?)),
            x => Err(TypeError::new(Type::String, x).into()),
        }
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for usize {
    type Error = anyhow::Error;
    fn try_from(obj: GcObj<'ob>) -> Result<Self, Self::Error> {
        match obj.untag() {
            Object::Int(x) => x
                .try_into()
                .with_context(|| format!("Integer must be positive, but was {x}")),
            x => Err(TypeError::new(Type::Int, x).into()),
        }
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for u64 {
    type Error = anyhow::Error;
    fn try_from(obj: GcObj<'ob>) -> Result<Self, Self::Error> {
        match obj.untag() {
            Object::Int(x) => x
                .try_into()
                .with_context(|| format!("Integer must be positive, but was {x}")),
            x => Err(TypeError::new(Type::Int, x).into()),
        }
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for Option<usize> {
    type Error = anyhow::Error;
    fn try_from(obj: GcObj<'ob>) -> Result<Self, Self::Error> {
        match obj.untag() {
            Object::Int(x) => match x.try_into() {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(e).with_context(|| format!("Integer must be positive, but was {x}")),
            },
            Object::NIL => Ok(None),
            _ => Err(TypeError::new(Type::Int, obj).into()),
        }
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for bool {
    type Error = ArgError;
    fn try_from(obj: GcObj) -> Result<Self, Self::Error> {
        Ok(obj.nil())
    }
}

impl<'ob> TryFrom<GcObj<'ob>> for Option<()> {
    type Error = ArgError;
    fn try_from(obj: GcObj) -> Result<Self, Self::Error> {
        Ok(obj.nil().then_some(()))
    }
}

/// This function is required because we have no specialization yet.
/// Essentially this let's us convert one type to another "in place"
/// without the need to allocate a new slice. We ensure that the two
/// types have the exact same representation, so that no writes
/// actually need to be performed.
pub(crate) fn try_from_slice<'brw, 'ob, T, E>(slice: &'brw [GcObj<'ob>]) -> Result<&'brw [Gc<T>], E>
where
    Gc<T>: TryFrom<GcObj<'ob>, Error = E> + 'ob,
{
    for x in slice.iter() {
        let _new = Gc::<T>::try_from(*x)?;
    }
    let ptr = slice.as_ptr().cast::<Gc<T>>();
    let len = slice.len();
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

impl<'ob> From<bool> for GcObj<'ob> {
    fn from(b: bool) -> Self {
        if b {
            qtrue()
        } else {
            nil()
        }
    }
}

define_unbox!(Int, i64);
define_unbox!(Float, &'ob LispFloat);
define_unbox!(HashTable, &'ob LispHashTable);
define_unbox!(String, &'ob LispString);
define_unbox!(Vec, &'ob LispVec);
define_unbox!(Symbol, Symbol<'ob>);

impl<'ob, T> From<Option<T>> for GcObj<'ob>
where
    T: Into<GcObj<'ob>>,
{
    fn from(t: Option<T>) -> Self {
        match t {
            Some(x) => x.into(),
            None => nil(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::core::cons::Cons;

    use super::super::super::gc::{Context, RootSet};

    use super::*;

    fn wrapper(args: &[GcObj]) -> Result<i64, TypeError> {
        Ok(inner(
            std::convert::TryFrom::try_from(args[0])?,
            std::convert::TryFrom::try_from(args[1])?,
        ))
    }

    fn inner(arg0: Option<i64>, arg1: &Cons) -> i64 {
        let x: i64 = arg1.car().try_into().unwrap();
        arg0.unwrap() + x
    }

    #[test]
    fn test() {
        let roots = &RootSet::default();
        let cx = &Context::new(roots);
        let obj0 = cx.add(5);
        // SAFETY: We don't call garbage collect so references are valid
        let obj1 = unsafe { cx.add(Cons::new(1.into(), 2.into())) };
        let vec = vec![obj0, obj1];
        let res = wrapper(vec.as_slice());
        assert_eq!(6, res.unwrap());
    }
}
