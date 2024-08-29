use ropey::{Rope, RopeSlice};
use std::{
	cmp, fmt,
	ops::{Add, Sub},
};

#[derive(Debug)]
pub struct Position(pub u32, pub u32);
#[derive(Debug)]
pub struct Range(pub Position, pub Position);

#[derive(Debug, Clone)]
pub struct Source(pub Rope);

impl Source {
	#[must_use]
	pub fn new(source: &str) -> Self {
		Self(Rope::from_str(source))
	}

	#[must_use]
	#[track_caller]
	pub fn str_from(&self, span: Span) -> RopeSlice<'_> {
		self.0
			.get_slice(span.low.to_usize()..span.high.to_usize())
			.unwrap()
	}

	#[must_use]
	pub fn to_line_col(&self, offset: BytePos) -> Option<Position> {
		let line = self.0.try_byte_to_line(offset.to_usize()).ok()?;
		let first_char_of_line = self.0.try_line_to_char(line).ok()?;
		let column = offset.to_usize() - first_char_of_line;
		Some(Position(line as u32, column as u32))
	}

	#[must_use]
	pub fn span_to_range(&self, span: Span) -> Option<Range> {
		let one = self.to_line_col(span.low)?;
		let two = self.to_line_col(span.high)?;

		Some(Range(one, two))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
	pub low: BytePos,
	pub high: BytePos,
}

impl Default for Span {
	fn default() -> Self {
		Self::DUMMY
	}
}

impl Span {
	pub const DUMMY: Self = Self {
		low: BytePos(u32::MAX),
		high: BytePos(u32::MAX),
	};

	#[must_use]
	pub const fn new(low: BytePos, high: BytePos) -> Self {
		Self { low, high }
	}

	#[must_use]
	pub fn relative(self, low: BytePos, high: BytePos) -> Self {
		Self {
			low: self.low + low,
			high: self.low + high,
		}
	}

	#[must_use]
	pub const fn low(&self) -> BytePos {
		self.low
	}

	#[must_use]
	pub const fn high(&self) -> BytePos {
		self.high
	}

	#[must_use]
	pub fn to(&self, span: Self) -> Self {
		Self {
			low: cmp::min(self.low, span.low),
			high: cmp::max(self.high, span.high),
		}
	}

	#[must_use]
	pub fn len(&self) -> BytePos {
		self.high - self.low
	}
}

/// Implements binary operators "&T op U", "T op &U", "&T op &U"
/// based on "T op U" where T and U are expected to be `Copy`able
macro_rules! forward_ref_bin_op {
	(impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
		impl<'a> $imp<$u> for &'a $t {
			type Output = <$t as $imp<$u>>::Output;

			#[inline]
			fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(*self, other)
			}
		}

		impl<'a> $imp<&'a $u> for $t {
			type Output = <$t as $imp<$u>>::Output;

			#[inline]
			fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(self, *other)
			}
		}

		impl<'a, 'b> $imp<&'a $u> for &'b $t {
			type Output = <$t as $imp<$u>>::Output;

			#[inline]
			fn $method(self, other: &'a $u) -> <$t as $imp<$u>>::Output {
				$imp::$method(*self, *other)
			}
		}
	};
}

macro_rules! impl_pos {
		(
			$(
				$(#[$attr:meta])*
				$vis:vis struct $ident:ident($inner_vis:vis $inner_ty:ty);
			)*
		) => {
			$(
				$(#[$attr])*
				$vis struct $ident($inner_vis $inner_ty);

				impl $ident {
					#[must_use]
					#[inline(always)]
					pub const fn from_usize(n: usize) -> $ident {
						$ident(n as $inner_ty)
					}

					#[must_use]
					#[inline(always)]
					pub const fn to_usize(self) -> usize {
						self.0 as usize
					}

					#[must_use]
					#[inline(always)]
					pub const fn from_u32(n: u32) -> $ident {
						$ident(n as $inner_ty)
					}

					#[must_use]
					#[inline(always)]
					pub const fn to_u32(self) -> u32 {
						self.0 as u32
					}
				}

				impl fmt::Display for $ident {
					fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
						self.0.fmt(f)
					}
				}

				impl core::ops::Add for $ident {
					type Output = $ident;

					#[inline(always)]
					fn add(self, rhs: $ident) -> $ident {
						$ident(self.0 + rhs.0)
					}
				}

				forward_ref_bin_op! { impl Add, add for $ident, $ident }

				impl core::ops::Sub for $ident {
					type Output = $ident;

					#[inline(always)]
					fn sub(self, rhs: $ident) -> $ident {
						$ident(self.0 - rhs.0)
					}
				}

				forward_ref_bin_op! { impl Sub, sub for $ident, $ident }
			)*
		};
	}

impl_pos! {
	/// A byte offset.
	///
	/// This is used in the
	/// This is kept small because an AST contains a lot of them.
	/// They also the limit the amount of sources that can be imported (≈ 4GiB).
	/// Find more information on [`SourceMap::allocate_space`](crate::SourceMap)
	#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
	pub struct BytePos(pub u32);

	/// A character offset.
	///
	/// Because of multibyte UTF-8 characters, a byte offset
	/// is not equivalent to a character offset. The [`SourceMap`](crate::SourceMap) will convert [`BytePos`]
	/// values to `CharPos` values as necessary.
	///
	/// It's a `usize` because it's easier to use with string slices
	#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
	pub struct CharPos(pub usize);
}
