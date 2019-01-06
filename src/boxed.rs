/*! `Box<[u1]>`
!*/

#![cfg(feature = "alloc")]

use crate::{
	BigEndian,
	BitPtr,
	BitSlice,
	BitVec,
	Bits,
	Cursor,
};
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
	borrow::{
		Borrow,
		BorrowMut,
		ToOwned,
	},
	boxed::Box,
	vec::Vec,
};
use conv::Conv;
use core::{
	clone::Clone,
	cmp::{
		Eq,
		PartialEq,
		PartialOrd,
		Ord,
		Ordering,
	},
	convert::{
		AsMut,
		AsRef,
		From,
		Into,
	},
	default::Default,
	fmt::{
		self,
		Debug,
		Display,
		Formatter,
	},
	hash::{
		Hash,
		Hasher,
	},
	iter::{
		DoubleEndedIterator,
		ExactSizeIterator,
		FusedIterator,
		Iterator,
		IntoIterator,
	},
	marker::PhantomData,
	mem,
	ops::{
		Add,
		AddAssign,
		BitAnd,
		BitAndAssign,
		BitOr,
		BitOrAssign,
		BitXor,
		BitXorAssign,
		Deref,
		DerefMut,
		Drop,
		Index,
		IndexMut,
		Range,
		RangeFrom,
		RangeFull,
		RangeInclusive,
		RangeTo,
		RangeToInclusive,
		Neg,
		Not,
		Shl,
		ShlAssign,
		Shr,
		ShrAssign,
	},
};
#[cfg(feature = "std")]
use std::{
	borrow::{
		Borrow,
		BorrowMut,
		ToOwned,
	},
	boxed::Box,
	vec::Vec,
};
use tap::Tap;

#[repr(C)]
pub struct BitBox<C = BigEndian, T = u8>
where C: Cursor, T: Bits {
	_cursor: PhantomData<C>,
	/// Marks that the `BitBox<T>` *owns* memory of type `T`.
	_memory: PhantomData<T>,
	pointer: BitPtr<T>,
}

impl<C, T> BitBox<C, T>
where C: Cursor, T: Bits {
	pub fn uninhabited(data: *const T) -> Self {
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::uninhabited(data),
		}
	}

	pub fn new(src: &BitSlice<C, T>) -> Self {
		let store: Box<[T]> = src.as_ref().to_owned().into_boxed_slice();
		let data = store.as_ptr();
		let (_, elts, head, tail) = src.bitptr().raw_parts();
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::new(data, elts, head, tail),
		}.tap(|_| mem::forget(store))
	}

	pub unsafe fn from_raw(pointer: BitPtr<T>) -> Self {
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer,
		}
	}

	/// Consumes the `BitBox`, returning the wrapped `BitPtr` directly.
	///
	/// After calling this function, the caller is responsible for the memory
	/// previously managed by the `BitBox`. In particular, the caller must
	/// properly release the memory region to which the `BitPtr` refers.
	/// The proper way to do so is to convert the `BitPtr` back into a `BitBox`
	/// with the [`BitBox::from_raw`] function.
	///
	/// Note: this is an associated function, which means that you must call it
	/// as `BitBox::into_raw(b)` instead of `b.into_raw()`. This is to match the
	/// API of [`Box`]; there is no method conflict with [`BitSlice`].
	///
	/// [`BitBox::from_raw`]: #method.from_raw
	/// [`BitSlice`]: ../struct.BitSlice.html
	#[cfg_attr(not(feature = "std"), doc = "[`Box`]: https://doc.rust-lang.org/stable/alloc/boxed/struct.Box.html")]
	#[cfg_attr(feature = "std", doc = "[`Box`]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html")]
	pub unsafe fn into_raw(b: BitBox<C, T>) -> BitPtr<T> {
		b.bitptr().tap(|_| mem::forget(b))
	}

	/// Consumes and leaks the `BitBox`, returning a mutable reference,
	/// `&'a mut BitSlice<C, T>`. Note that the memory region `[T]` must outlive
	/// the chosen lifetime `'a`.
	///
	/// This function is mainly useful for bit regions that live for the
	/// remainder of the program’s life. Dropping the returned reference will
	/// cause a memory leak. If this is not acceptable, the reference should
	/// first be wrapped with the [`Box::from_raw`] function, producing a
	/// `BitBox`. This `BitBox` can then be dropped which will properly
	/// deallocate the memory.
	///
	/// Note: this is an associated function, which means that you must call it
	/// as `BitBox::leak(b)` instead of `b.leak()`. This is to match the API of
	/// [`Box`]; there is no method conflict with [`BitSlice`].
	///
	/// [`BitBox::from_raw`]: #method.from_raw
	/// [`BitSlice`]: ../struct.BitSlice.html
	#[cfg_attr(not(feature = "std"), doc = "[`Box`]: https://doc.rust-lang.org/stable/alloc/boxed/struct.Box.html")]
	#[cfg_attr(feature = "std", doc = "[`Box`]: https://doc.rust-lang.org/stable/std/boxed/struct.Box.html")]
	pub fn leak<'a>(b: BitBox<C, T>) -> &'a mut BitSlice<C, T> {
		b.bitptr().tap(|_| mem::forget(b)).into()
	}

	pub fn as_bitslice(&self) -> &BitSlice<C, T> {
		self.pointer.into()
	}

	pub fn as_mut_bitslice(&mut self) -> &mut BitSlice<C, T> {
		self.pointer.into()
	}

	/// Gives read access to the `BitPtr<T>` structure powering the box.
	///
	/// # Parameters
	///
	/// - `&self`
	///
	/// # Returns
	///
	/// A copy of the interior `BitPtr<T>`.
	pub(crate) fn bitptr(&self) -> BitPtr<T> {
		self.pointer
	}

	fn do_with_box<F, R>(&self, func: F) -> R
	where F: FnOnce(&Box<[T]>) -> R {
		let (data, elts, _, _) = self.bitptr().raw_parts();
		let b: Box<[T]> = unsafe {
			Vec::from_raw_parts(data as *mut T, elts, elts)
		}.into_boxed_slice();
		func(&b).tap(|_| mem::forget(b))
	}
}

impl<C, T> Borrow<BitSlice<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn borrow(&self) -> &BitSlice<C, T> {
		&*self
	}
}

impl<C, T> BorrowMut<BitSlice<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn borrow_mut(&mut self) -> &mut BitSlice<C, T> {
		&mut *self
	}
}

impl<C, T> Clone for BitBox<C, T>
where C: Cursor, T: Bits {
	fn clone(&self) -> Self {
		let (_, e, h, t) = self.bitptr().raw_parts();
		let new_box = self.do_with_box(Clone::clone);
		let ptr = new_box.as_ptr();
		mem::forget(new_box);
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::new(ptr, e, h, t),
		}
	}
}

impl<C, T> Eq for BitBox<C, T>
where C: Cursor, T: Bits {}

impl<C, T> Ord for BitBox<C, T>
where C: Cursor, T: Bits {
	fn cmp(&self, rhs: &Self) -> Ordering {
		(&**self).cmp(&**rhs)
	}
}

impl<A, B, C, D> PartialEq<BitBox<C, D>> for BitBox<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn eq(&self, rhs: &BitBox<C, D>) -> bool {
		(&**self).eq(&**rhs)
	}
}

impl<A, B, C, D> PartialEq<BitSlice<C, D>> for BitBox<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn eq(&self, rhs: &BitSlice<C, D>) -> bool {
		(&**self).eq(rhs)
	}
}

impl<A, B, C, D> PartialEq<BitBox<C, D>> for BitSlice<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn eq(&self, rhs: &BitBox<C, D>) -> bool {
		self.eq(&**rhs)
	}
}

impl<A, B, C, D> PartialOrd<BitBox<C, D>> for BitBox<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn partial_cmp(&self, rhs: &BitBox<C, D>) -> Option<Ordering> {
		(&**self).partial_cmp(&**rhs)
	}
}

impl<A, B, C, D> PartialOrd<BitSlice<C, D>> for BitBox<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn partial_cmp(&self, rhs: &BitSlice<C, D>) -> Option<Ordering> {
		(&**self).partial_cmp(rhs)
	}
}

impl<A, B, C, D> PartialOrd<BitBox<C, D>> for BitSlice<A, B>
where A: Cursor, B: Bits, C: Cursor, D: Bits {
	fn partial_cmp(&self, rhs: &BitBox<C, D>) -> Option<Ordering> {
		self.partial_cmp(&**rhs)
	}
}

impl<C, T> AsMut<BitSlice<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn as_mut(&mut self) -> &mut BitSlice<C, T> {
		self.as_mut_bitslice()
	}
}

impl<C, T> AsMut<[T]> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn as_mut(&mut self) -> &mut [T] {
		(&mut **self).as_mut()
	}
}

impl<C, T> AsRef<BitSlice<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn as_ref(&self) -> &BitSlice<C, T> {
		self.as_bitslice()
	}
}

impl<C, T> AsRef<[T]> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn as_ref(&self) -> &[T] {
		(&**self).as_ref()
	}
}

impl<C, T> From<&BitSlice<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn from(src: &BitSlice<C, T>) -> Self {
		let (_, elts, head, tail) = src.bitptr().raw_parts();
		let b: Box<[T]> = src.as_ref().to_owned().into_boxed_slice();
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::new(b.as_ptr(), elts, head, tail),
		}.tap(|_| mem::forget(b))
	}
}

/// Builds a `BitBox` out of a borrowed slice of elements.
///
/// This copies the memory as-is from the source buffer into the new `BitBox`.
/// The source buffer will be unchanged by this operation, so you don't need to
/// worry about using the correct cursor type for the read.
///
/// This operation does a copy from the source buffer into a new allocation, as
/// it can only borrow the source and not take ownership.
impl<C, T> From<&[T]> for BitBox<C, T>
where C: Cursor, T: Bits {
	/// Builds a `BitBox<C: Cursor, T: Bits>` from a borrowed `&[T]`.
	///
	/// # Parameters
	///
	/// - `src`: The elements to use as the values for the new vector.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::*;
	///
	/// let src: &[u8] = &[5, 10];
	/// let bv: BitBox = src.into();
	/// assert!(bv[5]);
	/// assert!(bv[7]);
	/// assert!(bv[12]);
	/// assert!(bv[14]);
	/// ```
	fn from(src: &[T]) -> Self {
		assert!(src.len() < BitPtr::<T>::MAX_ELTS, "Box overflow");
		src.conv::<&BitSlice<C, T>>().into()
	}
}

impl<C, T> From<BitVec<C, T>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn from(mut src: BitVec<C, T>) -> Self {
		src.shrink_to_fit();
		let pointer = src.bitptr();
		mem::forget(src);
		unsafe { Self::from_raw(pointer) }
	}
}

/// Builds a `BitBox` out of an owned slice of elements.
///
/// This moves the memory as-is from the source buffer into the new `BitBox`.
/// The source buffer will be unchanged by this operation, so you don't need to
/// worry about using the correct cursor type.
impl<C, T> From<Box<[T]>> for BitBox<C, T>
where C: Cursor, T: Bits {
	/// Consumes a `Box<[T: Bits]>` and creates a `BitBox<C: Cursor, T>` from
	/// it.
	///
	/// # Parameters
	///
	/// - `src`: The source box whose memory will be used.
	///
	/// # Returns
	///
	/// A new `BitBox` using the `src` `Box`’s memory.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::*;
	///
	/// let src: Box<[u8]> = Box::new([3, 6, 9, 12, 15]);
	/// let bv: BitBox = src.into();
	/// ```
	fn from(src: Box<[T]>) -> Self {
		assert!(src.len() < BitPtr::<T>::MAX_ELTS, "Box overflow");
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::new(src.as_ptr(), src.len(), 0, T::SIZE)
		}
	}
}

impl<C, T> Into<Box<[T]>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn into(self) -> Box<[T]> {
		let (ptr, len, _, _) = self.bitptr().raw_parts();
		unsafe { Vec::from_raw_parts(ptr as *mut T, len, len) }
			.into_boxed_slice()
			.tap(|_| mem::forget(self))
	}
}

impl<C, T> Default for BitBox<C, T>
where C: Cursor, T: Bits {
	fn default() -> Self {
		Self {
			_cursor: PhantomData,
			_memory: PhantomData,
			pointer: BitPtr::default(),
		}
	}
}

impl<C, T> Debug for BitBox<C, T>
where C: Cursor, T: Bits {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		f.write_str("BitBox<")?;
		f.write_str(C::TYPENAME)?;
		f.write_str(", ")?;
		f.write_str(T::TYPENAME)?;
		f.write_str("> ")?;
		Display::fmt(&**self, f)
	}
}

impl<C, T> Display for BitBox<C, T>
where C: Cursor, T: Bits {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		Display::fmt(&**self, f)
	}
}

impl<C, T> Hash for BitBox<C, T>
where C: Cursor, T: Bits {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		(&**self).hash(hasher)
	}
}

impl<C, T> IntoIterator for BitBox<C, T>
where C: Cursor, T: Bits {
	type Item = bool;
	type IntoIter = IntoIter<C, T>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			iterator: self.bitptr(),
			_original: self,
		}
	}
}

impl<'a, C, T> IntoIterator for &'a BitBox<C, T>
where C: Cursor, T: 'a + Bits {
	type Item = bool;
	type IntoIter = <&'a BitSlice<C, T> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&**self).into_iter()
	}
}

impl<C, T> Add<Self> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = Self;

	fn add(mut self, addend: Self) -> Self::Output {
		self += addend;
		self
	}
}

impl<C, T> AddAssign for BitBox<C, T>
where C: Cursor, T: Bits {
	fn add_assign(&mut self, addend: Self) {
		**self += &*addend
	}
}

impl<C, T, I> BitAnd<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	type Output = Self;

	fn bitand(mut self, rhs: I) -> Self::Output {
		self &= rhs;
		self
	}
}

impl<C, T, I> BitAndAssign<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	fn bitand_assign(&mut self, rhs: I) {
		**self &= rhs;
	}
}

impl<C, T, I> BitOr<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	type Output = Self;

	fn bitor(mut self, rhs: I) -> Self::Output {
		self |= rhs;
		self
	}
}

impl<C, T, I> BitOrAssign<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	fn bitor_assign(&mut self, rhs: I) {
		**self |= rhs;
	}
}

impl<C, T, I> BitXor<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	type Output = Self;

	fn bitxor(mut self, rhs: I) -> Self::Output {
		self ^= rhs;
		self
	}
}

impl<C, T, I> BitXorAssign<I> for BitBox<C, T>
where C: Cursor, T: Bits, I: IntoIterator<Item=bool> {
	fn bitxor_assign(&mut self, rhs: I) {
		**self ^= rhs;
	}
}

impl<C, T> Deref for BitBox<C, T>
where C: Cursor, T: Bits {
	type Target = BitSlice<C, T>;

	fn deref(&self) -> &Self::Target {
		self.pointer.into()
	}
}

impl<C, T> DerefMut for BitBox<C, T>
where C: Cursor, T: Bits {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.pointer.into()
	}
}

impl<C, T> Drop for BitBox<C, T>
where C: Cursor, T: Bits {
	fn drop(&mut self) {
		let ptr = self.as_mut_bitslice().as_mut_ptr();
		let len = self.as_bitslice().len();
		//  Run the `Box<[T]>` destructor.
		drop(unsafe { Vec::from_raw_parts(ptr, len, len).into_boxed_slice() });
	}
}

impl<C, T> Index<usize> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = bool;

	fn index(&self, index: usize) -> &Self::Output {
		&(**self)[index]
	}
}

impl<C, T> Index<Range<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: Range<usize>) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<Range<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Index<RangeFrom<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: RangeFrom<usize>) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<RangeFrom<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: RangeFrom<usize>) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Index<RangeFull> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: RangeFull) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<RangeFull> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: RangeFull) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Index<RangeInclusive<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<RangeInclusive<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: RangeInclusive<usize>) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Index<RangeTo<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: RangeTo<usize>) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<RangeTo<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: RangeTo<usize>) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Index<RangeToInclusive<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = BitSlice<C, T>;

	fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {
		&(**self)[range]
	}
}

impl<C, T> IndexMut<RangeToInclusive<usize>> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn index_mut(&mut self, range: RangeToInclusive<usize>) -> &mut Self::Output {
		&mut (**self)[range]
	}
}

impl<C, T> Neg for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = Self;

	fn neg(mut self) -> Self::Output {
		let _ = -(&mut *self);
		self
	}
}

impl<C, T> Not for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = Self;

	fn not(mut self) -> Self::Output {
		let _ = !(&mut *self);
		self
	}
}

impl<C, T> Shl<usize> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = Self;

	fn shl(mut self, shamt: usize) -> Self::Output {
		self <<= shamt;
		self
	}
}

impl<C, T> ShlAssign<usize> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn shl_assign(&mut self, shamt: usize) {
		**self <<= shamt;
	}
}

impl<C, T> Shr<usize> for BitBox<C, T>
where C: Cursor, T: Bits {
	type Output = Self;

	fn shr(mut self, shamt: usize) -> Self::Output {
		self >>= shamt;
		self
	}
}

impl<C, T> ShrAssign<usize> for BitBox<C, T>
where C: Cursor, T: Bits {
	fn shr_assign(&mut self, shamt: usize) {
		**self >>= shamt;
	}
}

#[repr(C)]
pub struct IntoIter<C, T>
where C: Cursor, T: Bits {
	/// Owning pointer to the full slab
	_original: BitBox<C, T>,
	/// Slice descriptor for the region undergoing iteration.
	iterator: BitPtr<T>,
}

impl<C, T> IntoIter<C, T>
where C: Cursor, T: Bits {
	fn iterator(&self) -> <&BitSlice<C, T> as IntoIterator>::IntoIter {
		self.iterator.conv::<&BitSlice<C, T>>().into_iter()
	}
}

impl<C, T> DoubleEndedIterator for IntoIter<C, T>
where C: Cursor, T: Bits {
	fn next_back(&mut self) -> Option<Self::Item> {
		let mut slice_iter = self.iterator();
		let out = slice_iter.next_back();
		self.iterator = slice_iter.bitptr();
		out
	}
}

impl<C, T> ExactSizeIterator for IntoIter<C, T>
where C: Cursor, T: Bits {}

impl<C, T> FusedIterator for IntoIter<C, T>
where C: Cursor, T: Bits {}

impl<C, T> Iterator for IntoIter<C, T>
where C: Cursor, T: Bits {
	type Item = bool;

	fn next(&mut self) -> Option<Self::Item> {
		let mut slice_iter = self.iterator();
		let out = slice_iter.next();
		self.iterator = slice_iter.bitptr();
		out
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.iterator().size_hint()
	}

	fn count(self) -> usize {
		self.len()
	}

	fn nth(&mut self, n: usize) -> Option<Self::Item> {
		let mut slice_iter = self.iterator();
		let out = slice_iter.nth(n);
		self.iterator = slice_iter.bitptr();
		out
	}

	fn last(mut self) -> Option<Self::Item> {
		self.next_back()
	}
}
