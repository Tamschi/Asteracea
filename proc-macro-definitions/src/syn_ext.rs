use easy_ext::ext;
use merging_iterator::MergeIter;
use std::borrow::Cow;
use syn::{
	punctuated::Punctuated, Error, Expr, ExprPath, FieldPat, GenericArgument, GenericParam,
	Generics, Ident, Member, Pat, PatIdent, PatPath, Path, Result, Token, Type, TypeParam,
	TypePath, WhereClause,
};

pub trait AddOptionExt<'a, Other = Self, Output = Cow<'a, Self>> {
	fn add(&'a self, other: &'a Other) -> Output;
}

impl<'a, T: Clone + AddExt> AddOptionExt<'a, Self, Option<Cow<'a, T>>> for Option<T> {
	fn add(&'a self, other: &'a Self) -> Option<Cow<'a, T>> {
		match (self, other) {
			(None, None) => None,
			(None, one @ Some(_)) | (one @ Some(_), None) => one.as_ref().map(Cow::Borrowed),
			(Some(left), Some(right)) => Some(left.add(right)),
		}
	}
}

pub trait AddExt<Other = Self, Output: Clone = Self> {
	fn add<'a>(&'a self, other: &'a Other) -> Cow<'a, Output>;
}

impl AddExt for Generics {
	/// # Debug Panics
	///
	/// On malformed [`Generics`] (missing required lt or gt tokens).
	fn add<'a>(&'a self, other: &'a Self) -> Cow<'a, Self> {
		if other.lt_token.is_none() && other.where_clause.is_none() {
			debug_assert!(other.params.is_empty());
			debug_assert!(other.gt_token.is_none());
			Cow::Borrowed(self)
		} else if self.lt_token.is_none() && self.where_clause.is_none() {
			debug_assert!(self.params.is_empty());
			debug_assert!(self.gt_token.is_none());
			Cow::Borrowed(other)
		} else {
			if !self.params.is_empty() {
				debug_assert!(self.lt_token.is_some())
			}
			if self.lt_token.is_some() {
				debug_assert!(self.gt_token.is_some())
			};
			if self.gt_token.is_some() {
				debug_assert!(self.lt_token.is_some())
			}
			if !other.params.is_empty() {
				debug_assert!(other.lt_token.is_some())
			}
			if other.lt_token.is_some() {
				debug_assert!(other.gt_token.is_some())
			};
			if other.gt_token.is_some() {
				debug_assert!(other.lt_token.is_some())
			}
			Cow::Owned(Generics {
				lt_token: self
					.lt_token
					.as_ref()
					.or_else(|| other.lt_token.as_ref())
					.cloned(),
				params: merging_iterator::MergeIter::with_custom_ordering(
					self.params.iter(),
					other.params.iter(),
					|left, right| match (left, right) {
						(GenericParam::Lifetime(_), _) => true,
						(_, GenericParam::Lifetime(_)) => false,
						(GenericParam::Type(_), _) => true,
						(_, GenericParam::Type(_)) => false,
						(GenericParam::Const(_), GenericParam::Const(_)) => true,
					},
				)
				.cloned()
				.collect(),
				gt_token: self
					.gt_token
					.as_ref()
					.or_else(|| other.gt_token.as_ref())
					.cloned(),
				where_clause: self
					.where_clause
					.add(&other.where_clause)
					.map(Cow::into_owned),
			})
		}
	}
}

impl AddExt for WhereClause {
	fn add<'a>(&'a self, other: &'a Self) -> Cow<'a, Self> {
		if other.predicates.is_empty() {
			Cow::Borrowed(self)
		} else if self.predicates.is_empty() {
			Cow::Borrowed(other)
		} else {
			Cow::Owned(Self {
				where_token: self.where_token,
				predicates: self
					.predicates
					.iter()
					.chain(other.predicates.iter())
					.cloned()
					.collect(),
			})
		}
	}
}

impl<P: Default + Clone> AddExt for Punctuated<GenericArgument, P> {
	fn add<'a>(&'a self, other: &'a Self) -> Cow<'a, Self> {
		if other.is_empty() {
			Cow::Borrowed(self)
		} else if self.is_empty() {
			Cow::Borrowed(other)
		} else {
			Cow::Owned(
				MergeIter::with_custom_ordering(self.iter(), other.iter(), |left, right| {
					match (left, right) {
						(GenericArgument::Lifetime(_), _) => true,
						(_, GenericArgument::Lifetime(_)) => false,
						(GenericArgument::Type(_), _) => true,
						(_, GenericArgument::Type(_)) => false,
						(GenericArgument::Binding(_), _) => true,
						(_, GenericArgument::Binding(_)) => false,
						(GenericArgument::Constraint(_), _) => true,
						(_, GenericArgument::Constraint(_)) => false,
						(GenericArgument::Const(_), GenericArgument::Const(_)) => true,
					}
				})
				.cloned()
				.collect(),
			)
		}
	}
}

#[ext]
impl<V, P: Default> Punctuated<V, P> {
	pub fn into_with_trailing(mut self) -> Self {
		if !self.is_empty() && !self.trailing_punct() {
			self.push_punct(P::default())
		}
		self
	}
}

#[ext]
impl GenericParam {
	pub fn to_argument(&self) -> GenericArgument {
		match self {
			GenericParam::Type(ty_param) => GenericArgument::Type(ty_param.ident.to_type()),
			GenericParam::Lifetime(l_def) => GenericArgument::Lifetime(l_def.lifetime.clone()),
			GenericParam::Const(c_param) => GenericArgument::Const(c_param.ident.to_expr()),
		}
	}
}

#[ext]
impl TypeParam {
	pub fn to_argument(&self) -> GenericArgument {
		GenericArgument::Type(self.ident.to_type())
	}
}

#[ext]
impl Ident {
	pub fn to_member(&self) -> Member {
		Member::Named(self.clone())
	}

	pub fn to_pat(&self) -> Pat {
		Pat::Path(self.to_pat_path())
	}

	pub fn to_pat_path(&self) -> PatPath {
		PatPath {
			attrs: vec![],
			qself: None,
			path: self.to_path(),
		}
	}

	pub fn to_expr(&self) -> Expr {
		self.to_expr_path().into_expr()
	}

	pub fn to_expr_path(&self) -> ExprPath {
		self.to_path().into_expr_path()
	}

	pub fn to_path(&self) -> Path {
		self.clone().into()
	}

	pub fn to_type(&self) -> Type {
		self.to_type_path().into_type()
	}

	pub fn to_type_path(&self) -> TypePath {
		self.to_path().into_type_path()
	}
}

#[ext]
impl Path {
	pub fn into_expr_path(self) -> ExprPath {
		ExprPath {
			attrs: vec![],
			qself: None,
			path: self,
		}
	}

	pub fn into_type_path(self) -> TypePath {
		TypePath {
			qself: None,
			path: self,
		}
	}
}

#[ext]
impl ExprPath {
	pub fn into_expr(self) -> Expr {
		Expr::Path(self)
	}
}

#[ext]
impl TypePath {
	pub fn into_type(self) -> Type {
		Type::Path(self)
	}
}

#[ext]
impl<T> Option<T> {
	fn reduce(self, other: Self, reducer: impl FnOnce(T, T) -> T) -> Self {
		match (self, other) {
			(None, None) => None,
			(None, one @ Some(_)) | (one @ Some(_), None) => one,
			(Some(this), Some(other)) => Some(reducer(this, other)),
		}
	}
}

#[ext]
impl PatIdent {
	/// # Errors
	///
	/// Iff `self.subpat` is [`None`].
	pub fn try_into_field_pat(self) -> Result<FieldPat> {
		Ok(match self.subpat {
			Some((at, subpat)) => {
				let by_ref_err = self.by_ref.map(|by_ref| {
					Error::new_spanned(
						by_ref,
						"`ref` is not yet supported in this position. Please nest the pattern.",
					)
				});
				let mut_err = self.mutability.map(|mutability| {
					Error::new_spanned(
						mutability,
						"`ref` is not yet supported in this position. Please nest the pattern.",
					)
				});
				if let Some(err) = by_ref_err.reduce(mut_err, |mut a, b| {
					a.combine(b);
					a
				}) {
					return Err(err);
				}
				FieldPat {
					attrs: self.attrs,
					member: self.ident.to_member(),
					colon_token: Some(Token![:](at.span)),
					pat: subpat,
				}
			}
			None => FieldPat {
				attrs: self.attrs,
				member: self.ident.clone().to_member(),
				colon_token: None,
				pat: self.ident.to_pat().into(),
			},
		})
	}
}

#[ext]
impl Option<Generics> {
	pub fn realise(&self) -> Cow<Generics> {
		match self {
			Some(real) => Cow::Borrowed(real),
			None => Cow::Owned(Generics {
				lt_token: None,
				params: Punctuated::<_, _>::default(),
				gt_token: None,
				where_clause: None,
			}),
		}
	}
}
