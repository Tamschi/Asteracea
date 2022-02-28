use asteracea::services::Invalidator;
use bumpalo::Bump;
use debugless_unwrap::DebuglessUnwrap;
use ergo_pin::ergo_pin;
use rhizome::sync::{Inject, Node};
use std::{any::TypeId, task::Context};
use this_is_fine::FineExt;
struct ContainerNewArgs<'NEW, 'a: 'NEW> {
    __Asteracea__phantom: ::std::marker::PhantomData<(&'NEW (), &'a ())>,
}
impl<'NEW, 'a: 'NEW> ContainerNewArgs<'NEW, 'a> {
    /**
                    Create a builder for building `ContainerNewArgs`.
                    On the builder, call  to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ContainerNewArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ContainerNewArgsBuilder<'NEW, 'a, ()> {
        ContainerNewArgsBuilder {
            fields: (),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ContainerNewArgs`] instances.

See [`ContainerNewArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ContainerNewArgsBuilder<'NEW, 'a: 'NEW, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'NEW ()>,
        ::core::marker::PhantomData<&'a ()>,
    ),
}
impl<'NEW, 'a: 'NEW, TypedBuilderFields> Clone
for ContainerNewArgsBuilder<'NEW, 'a, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ContainerNewArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ContainerNewArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ContainerNewArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'NEW, 'a: 'NEW> ContainerNewArgsBuilder<'NEW, 'a, ()> {
    ///Finalise the builder and create its [`ContainerNewArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ContainerNewArgs<'NEW, 'a> {
        let () = self.fields;
        let __Asteracea__phantom = ::core::default::Default::default();
        ContainerNewArgs {
            __Asteracea__phantom,
        }
    }
}
struct ContainerRenderArgs<'RENDER, 'a, 'bump: 'RENDER> {
    __Asteracea__anonymous_content: (
        ::asteracea::__::AnonymousContentParentParameters,
        ::std::boxed::Box<
            dyn 'RENDER + ::core::ops::FnOnce(
                &'bump ::asteracea::bumpalo::Bump,
            ) -> ::std::result::Result<
                    ::asteracea::lignin::Guard<'bump, ::asteracea::lignin::ThreadSafe>,
                    ::asteracea::error::Escalation,
                >,
        >,
    ),
    __Asteracea__phantom: ::std::marker::PhantomData<(&'RENDER (), &'a (), &'bump ())>,
}
impl<'RENDER, 'a, 'bump: 'RENDER> ContainerRenderArgs<'RENDER, 'a, 'bump> {
    /**
                    Create a builder for building `ContainerRenderArgs`.
                    On the builder, call `.__Asteracea__anonymous_content(...)` to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ContainerRenderArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ContainerRenderArgsBuilder<'RENDER, 'a, 'bump, ((),)> {
        ContainerRenderArgsBuilder {
            fields: ((),),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ContainerRenderArgs`] instances.

See [`ContainerRenderArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ContainerRenderArgsBuilder<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'RENDER ()>,
        ::core::marker::PhantomData<&'a ()>,
        ::core::marker::PhantomData<&'bump ()>,
    ),
}
impl<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> Clone
for ContainerRenderArgsBuilder<'RENDER, 'a, 'bump, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ContainerRenderArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ContainerRenderArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ContainerRenderArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'RENDER, 'a, 'bump: 'RENDER> ContainerRenderArgsBuilder<'RENDER, 'a, 'bump, ((),)> {
    pub fn __Asteracea__anonymous_content(
        self,
        __Asteracea__anonymous_content: (
            ::asteracea::__::AnonymousContentParentParameters,
            ::std::boxed::Box<
                dyn 'RENDER + ::core::ops::FnOnce(
                    &'bump ::asteracea::bumpalo::Bump,
                ) -> ::std::result::Result<
                        ::asteracea::lignin::Guard<
                            'bump,
                            ::asteracea::lignin::ThreadSafe,
                        >,
                        ::asteracea::error::Escalation,
                    >,
            >,
        ),
    ) -> ContainerRenderArgsBuilder<
            'RENDER,
            'a,
            'bump,
            (
                (
                    (
                        ::asteracea::__::AnonymousContentParentParameters,
                        ::std::boxed::Box<
                            dyn 'RENDER + ::core::ops::FnOnce(
                                &'bump ::asteracea::bumpalo::Bump,
                            ) -> ::std::result::Result<
                                    ::asteracea::lignin::Guard<
                                        'bump,
                                        ::asteracea::lignin::ThreadSafe,
                                    >,
                                    ::asteracea::error::Escalation,
                                >,
                        >,
                    ),
                ),
            ),
        > {
        let __Asteracea__anonymous_content = (__Asteracea__anonymous_content,);
        let (_,) = self.fields;
        ContainerRenderArgsBuilder {
            fields: (__Asteracea__anonymous_content,),
            phantom: self.phantom,
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub enum ContainerRenderArgsBuilder_Error_Repeated_field___Asteracea__anonymous_content {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<
    'RENDER,
    'a,
    'bump: 'RENDER,
> ContainerRenderArgsBuilder<
    'RENDER,
    'a,
    'bump,
    (
        (
            (
                ::asteracea::__::AnonymousContentParentParameters,
                ::std::boxed::Box<
                    dyn 'RENDER + ::core::ops::FnOnce(
                        &'bump ::asteracea::bumpalo::Bump,
                    ) -> ::std::result::Result<
                            ::asteracea::lignin::Guard<
                                'bump,
                                ::asteracea::lignin::ThreadSafe,
                            >,
                            ::asteracea::error::Escalation,
                        >,
                >,
            ),
        ),
    ),
> {
    #[deprecated(note = "Repeated field __Asteracea__anonymous_content")]
    pub fn __Asteracea__anonymous_content(
        self,
        _: ContainerRenderArgsBuilder_Error_Repeated_field___Asteracea__anonymous_content,
    ) -> ContainerRenderArgsBuilder<
            'RENDER,
            'a,
            'bump,
            (
                (
                    (
                        ::asteracea::__::AnonymousContentParentParameters,
                        ::std::boxed::Box<
                            dyn 'RENDER + ::core::ops::FnOnce(
                                &'bump ::asteracea::bumpalo::Bump,
                            ) -> ::std::result::Result<
                                    ::asteracea::lignin::Guard<
                                        'bump,
                                        ::asteracea::lignin::ThreadSafe,
                                    >,
                                    ::asteracea::error::Escalation,
                                >,
                        >,
                    ),
                ),
            ),
        > {
        self
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub enum ContainerRenderArgsBuilder_Error_Missing_required_field___Asteracea__anonymous_content {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::panic)]
impl<'RENDER, 'a, 'bump: 'RENDER> ContainerRenderArgsBuilder<'RENDER, 'a, 'bump, ((),)> {
    #[deprecated(note = "Missing required field __Asteracea__anonymous_content")]
    pub fn build(
        self,
        _: ContainerRenderArgsBuilder_Error_Missing_required_field___Asteracea__anonymous_content,
    ) -> ContainerRenderArgs<'RENDER, 'a, 'bump> {
        { panic!("explicit panic") };
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<
    'RENDER,
    'a,
    'bump: 'RENDER,
> ContainerRenderArgsBuilder<
    'RENDER,
    'a,
    'bump,
    (
        (
            (
                ::asteracea::__::AnonymousContentParentParameters,
                ::std::boxed::Box<
                    dyn 'RENDER + ::core::ops::FnOnce(
                        &'bump ::asteracea::bumpalo::Bump,
                    ) -> ::std::result::Result<
                            ::asteracea::lignin::Guard<
                                'bump,
                                ::asteracea::lignin::ThreadSafe,
                            >,
                            ::asteracea::error::Escalation,
                        >,
                >,
            ),
        ),
    ),
> {
    ///Finalise the builder and create its [`ContainerRenderArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ContainerRenderArgs<'RENDER, 'a, 'bump> {
        let (__Asteracea__anonymous_content,) = self.fields;
        let __Asteracea__anonymous_content = __Asteracea__anonymous_content.0;
        let __Asteracea__phantom = ::core::default::Default::default();
        ContainerRenderArgs {
            __Asteracea__anonymous_content,
            __Asteracea__phantom,
        }
    }
}
struct Container {}
impl Container {}
impl Container {
    /// <!-- (suppress `missing_docs`) -->
    pub fn new<'parent_resource_node_borrow, 'a>(
        parent_node: ::core::pin::Pin<
            &'parent_resource_node_borrow ::asteracea::include::dependency_injection::ResourceNode,
        >,
        args: ContainerNewArgs<'_, 'a>,
    ) -> ::std::result::Result<
            (
                Self,
                ::asteracea::include::dependency_injection::SparseResourceNodeHandle<
                    'parent_resource_node_borrow,
                >,
            ),
            ::asteracea::error::Escalation,
        >
    where
        Self: 'a + 'static,
    {
        let (ContainerNewArgs { __Asteracea__phantom: _ }, ()) = (args, ());
        let mut resource_node = ::asteracea::include::dependency_injection::ResourceBob::new_for::<
            Self,
        >(parent_node);
        let local_resource_node = unsafe {
            ::core::pin::Pin::new_unchecked(&mut resource_node)
        };
        {}
        <dyn Invalidator>::inject(
                local_resource_node.borrow(),
                |_: Option<&mut Context<'_>>| panic!("not implemented"),
            )
            .not_fine()
            .debugless_unwrap();
        {}
        let resource_node = resource_node.into_sparse_handle();
        ::std::result::Result::Ok((
            {
                let resource_node = resource_node.as_ref();
                Container {}
            },
            resource_node,
        ))
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn new_args_builder<'NEW, 'a: 'NEW>() -> ContainerNewArgsBuilder<'NEW, 'a, ()> {
        ContainerNewArgs::builder()
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render<'a, 'bump>(
        self: ::std::pin::Pin<&'a Self>,
        bump: &'bump asteracea::bumpalo::Bump,
        args: ContainerRenderArgs<'_, 'a, 'bump>,
    ) -> ::std::result::Result<
            impl Container__Asteracea__AutoSafe<
                Bound = ::asteracea::lignin::Guard<
                    'bump,
                    ::asteracea::lignin::ThreadBound,
                >,
            >,
            ::asteracea::error::Escalation,
        > {
        let ContainerRenderArgs {
            __Asteracea__anonymous_content,
            __Asteracea__phantom: _,
        } = args;
        let mut on_vdom_drop: ::core::option::Option<
            ::asteracea::lignin::guard::ConsumedCallback,
        > = None;
        let this = self;
        ::std::result::Result::Ok(
            ::asteracea::lignin::Guard::new(
                {
                    let children = {
                        let guard = (__Asteracea__anonymous_content.1)(bump)?;
                        unsafe {
                            guard
                                .peel(
                                    &mut on_vdom_drop,
                                    || bump.alloc_with(|| ::core::mem::MaybeUninit::uninit()),
                                )
                        }
                    }
                        .prefer_thread_safe();
                    ::asteracea::lignin::Node::HtmlElement::<'bump, _> {
                        element: bump
                            .alloc_with(|| asteracea::lignin::Element {
                                name: "custom-container",
                                creation_options: ::asteracea::lignin::ElementCreationOptions::new(),
                                attributes: &*bump.alloc_with(|| []),
                                content: children,
                                event_bindings: &*bump.alloc_with(|| []),
                            }),
                        dom_binding: None,
                    }
                }
                    .prefer_thread_safe(),
                on_vdom_drop,
            ),
        )
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render_args_builder<'RENDER, 'a, 'bump: 'RENDER>() -> ContainerRenderArgsBuilder<
            'RENDER,
            'a,
            'bump,
            ((),),
        > {
        ContainerRenderArgs::builder()
    }
    #[doc(hidden)]
    pub fn __Asteracea__ref_render_args_builder<'RENDER, 'a, 'bump: 'RENDER>(
        &self,
    ) -> ContainerRenderArgsBuilder<'RENDER, 'a, 'bump, ((),)> {
        let _ = self;
        ContainerRenderArgs::builder()
    }
}
/// An alias for [`$crate::auto_safety::AutoSafe`] with custom visibility.
pub(self) trait Container__Asteracea__AutoSafe: ::lignin::guard::auto_safety::AutoSafe<
        Bound = <Self as Container__Asteracea__AutoSafe>::Bound,
    > {
    type Bound: ::lignin::guard::auto_safety::Bound;
}
impl<T> Container__Asteracea__AutoSafe for T
where
    T: ::lignin::guard::auto_safety::AutoSafe,
{
    type Bound = <T as ::lignin::guard::auto_safety::AutoSafe>::Bound;
}
/// Asteracea components do not currently support custom [`Drop`](`::std::ops::Drop`) implementations.
impl ::std::ops::Drop for Container {
    fn drop(&mut self) {
        unsafe {}
    }
}
struct ContentNewArgs<'NEW, 'a: 'NEW> {
    __Asteracea__phantom: ::std::marker::PhantomData<(&'NEW (), &'a ())>,
}
impl<'NEW, 'a: 'NEW> ContentNewArgs<'NEW, 'a> {
    /**
                    Create a builder for building `ContentNewArgs`.
                    On the builder, call  to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ContentNewArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ContentNewArgsBuilder<'NEW, 'a, ()> {
        ContentNewArgsBuilder {
            fields: (),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ContentNewArgs`] instances.

See [`ContentNewArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ContentNewArgsBuilder<'NEW, 'a: 'NEW, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'NEW ()>,
        ::core::marker::PhantomData<&'a ()>,
    ),
}
impl<'NEW, 'a: 'NEW, TypedBuilderFields> Clone
for ContentNewArgsBuilder<'NEW, 'a, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ContentNewArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ContentNewArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ContentNewArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'NEW, 'a: 'NEW> ContentNewArgsBuilder<'NEW, 'a, ()> {
    ///Finalise the builder and create its [`ContentNewArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ContentNewArgs<'NEW, 'a> {
        let () = self.fields;
        let __Asteracea__phantom = ::core::default::Default::default();
        ContentNewArgs {
            __Asteracea__phantom,
        }
    }
}
struct ContentRenderArgs<'RENDER, 'a, 'bump: 'RENDER> {
    __Asteracea__phantom: ::std::marker::PhantomData<(&'RENDER (), &'a (), &'bump ())>,
}
impl<'RENDER, 'a, 'bump: 'RENDER> ContentRenderArgs<'RENDER, 'a, 'bump> {
    /**
                    Create a builder for building `ContentRenderArgs`.
                    On the builder, call  to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ContentRenderArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ContentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
        ContentRenderArgsBuilder {
            fields: (),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ContentRenderArgs`] instances.

See [`ContentRenderArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ContentRenderArgsBuilder<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'RENDER ()>,
        ::core::marker::PhantomData<&'a ()>,
        ::core::marker::PhantomData<&'bump ()>,
    ),
}
impl<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> Clone
for ContentRenderArgsBuilder<'RENDER, 'a, 'bump, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ContentRenderArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ContentRenderArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ContentRenderArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'RENDER, 'a, 'bump: 'RENDER> ContentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
    ///Finalise the builder and create its [`ContentRenderArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ContentRenderArgs<'RENDER, 'a, 'bump> {
        let () = self.fields;
        let __Asteracea__phantom = ::core::default::Default::default();
        ContentRenderArgs {
            __Asteracea__phantom,
        }
    }
}
struct Content {}
impl Content {}
impl Content {
    /// <!-- (suppress `missing_docs`) -->
    pub fn new<'parent_resource_node_borrow, 'a>(
        parent_node: ::core::pin::Pin<
            &'parent_resource_node_borrow ::asteracea::include::dependency_injection::ResourceNode,
        >,
        args: ContentNewArgs<'_, 'a>,
    ) -> ::std::result::Result<
            (
                Self,
                ::asteracea::include::dependency_injection::SparseResourceNodeHandle<
                    'parent_resource_node_borrow,
                >,
            ),
            ::asteracea::error::Escalation,
        >
    where
        Self: 'a + 'static,
    {
        let (ContentNewArgs { __Asteracea__phantom: _ }, (_invalidator,)) = (
            args,
            (
                <dyn Invalidator as ::asteracea::__::rhizome::sync::Extract>::extract(
                        parent_node,
                    )
                    .map_err(
                        ::asteracea::error::IncompatibleRuntimeDependency::<
                            dyn Invalidator,
                        >::new_and_log,
                    )
                    .map_err(::asteracea::error::Escalate::escalate)?
                    .ok_or_else(|| ::asteracea::error::RuntimeDependencyMissing::<
                        dyn Invalidator,
                    >::new_and_log())
                    .map_err(::asteracea::error::Escalate::escalate)?,
            ),
        );
        let mut resource_node = ::asteracea::include::dependency_injection::ResourceBob::new_for::<
            Self,
        >(parent_node);
        let local_resource_node = unsafe {
            ::core::pin::Pin::new_unchecked(&mut resource_node)
        };
        {}
        {}
        let resource_node = resource_node.into_sparse_handle();
        ::std::result::Result::Ok((
            {
                let resource_node = resource_node.as_ref();
                Content {}
            },
            resource_node,
        ))
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn new_args_builder<'NEW, 'a: 'NEW>() -> ContentNewArgsBuilder<'NEW, 'a, ()> {
        ContentNewArgs::builder()
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render<'a, 'bump>(
        self: ::std::pin::Pin<&'a Self>,
        bump: &'bump asteracea::bumpalo::Bump,
        args: ContentRenderArgs<'_, 'a, 'bump>,
    ) -> ::std::result::Result<
            impl Content__Asteracea__AutoSafe<
                Bound = ::asteracea::lignin::Guard<
                    'bump,
                    ::asteracea::lignin::ThreadBound,
                >,
            >,
            ::asteracea::error::Escalation,
        > {
        let ContentRenderArgs { __Asteracea__phantom: _ } = args;
        let mut on_vdom_drop: ::core::option::Option<
            ::asteracea::lignin::guard::ConsumedCallback,
        > = None;
        let this = self;
        ::std::result::Result::Ok(
            ::asteracea::lignin::Guard::new(
                {
                    let children = ::asteracea::lignin::Node::Multi::<
                        'bump,
                        _,
                    >(
                        &*bump
                            .alloc_try_with(|| -> ::std::result::Result<
                                _,
                                ::asteracea::error::Escalation,
                            > { ::std::result::Result::Ok([]) })?,
                    );
                    ::asteracea::lignin::Node::HtmlElement::<'bump, _> {
                        element: bump
                            .alloc_with(|| asteracea::lignin::Element {
                                name: "custom-content",
                                creation_options: ::asteracea::lignin::ElementCreationOptions::new(),
                                attributes: &*bump.alloc_with(|| []),
                                content: children,
                                event_bindings: &*bump.alloc_with(|| []),
                            }),
                        dom_binding: None,
                    }
                }
                    .prefer_thread_safe(),
                on_vdom_drop,
            ),
        )
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render_args_builder<'RENDER, 'a, 'bump: 'RENDER>() -> ContentRenderArgsBuilder<
            'RENDER,
            'a,
            'bump,
            (),
        > {
        ContentRenderArgs::builder()
    }
    #[doc(hidden)]
    pub fn __Asteracea__ref_render_args_builder<'RENDER, 'a, 'bump: 'RENDER>(
        &self,
    ) -> ContentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
        let _ = self;
        ContentRenderArgs::builder()
    }
}
/// An alias for [`$crate::auto_safety::AutoSafe`] with custom visibility.
pub(self) trait Content__Asteracea__AutoSafe: ::lignin::guard::auto_safety::AutoSafe<
        Bound = <Self as Content__Asteracea__AutoSafe>::Bound,
    > {
    type Bound: ::lignin::guard::auto_safety::Bound;
}
impl<T> Content__Asteracea__AutoSafe for T
where
    T: ::lignin::guard::auto_safety::AutoSafe,
{
    type Bound = <T as ::lignin::guard::auto_safety::AutoSafe>::Bound;
}
/// Asteracea components do not currently support custom [`Drop`](`::std::ops::Drop`) implementations.
impl ::std::ops::Drop for Content {
    fn drop(&mut self) {
        unsafe {}
    }
}
struct ParentNewArgs<'NEW, 'a: 'NEW> {
    __Asteracea__phantom: ::std::marker::PhantomData<(&'NEW (), &'a ())>,
}
impl<'NEW, 'a: 'NEW> ParentNewArgs<'NEW, 'a> {
    /**
                    Create a builder for building `ParentNewArgs`.
                    On the builder, call  to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ParentNewArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ParentNewArgsBuilder<'NEW, 'a, ()> {
        ParentNewArgsBuilder {
            fields: (),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ParentNewArgs`] instances.

See [`ParentNewArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ParentNewArgsBuilder<'NEW, 'a: 'NEW, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'NEW ()>,
        ::core::marker::PhantomData<&'a ()>,
    ),
}
impl<'NEW, 'a: 'NEW, TypedBuilderFields> Clone
for ParentNewArgsBuilder<'NEW, 'a, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ParentNewArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ParentNewArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ParentNewArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'NEW, 'a: 'NEW> ParentNewArgsBuilder<'NEW, 'a, ()> {
    ///Finalise the builder and create its [`ParentNewArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ParentNewArgs<'NEW, 'a> {
        let () = self.fields;
        let __Asteracea__phantom = ::core::default::Default::default();
        ParentNewArgs {
            __Asteracea__phantom,
        }
    }
}
struct ParentRenderArgs<'RENDER, 'a, 'bump: 'RENDER> {
    __Asteracea__phantom: ::std::marker::PhantomData<(&'RENDER (), &'a (), &'bump ())>,
}
impl<'RENDER, 'a, 'bump: 'RENDER> ParentRenderArgs<'RENDER, 'a, 'bump> {
    /**
                    Create a builder for building `ParentRenderArgs`.
                    On the builder, call  to set the values of the fields.
                    Finally, call `.build()` to create the instance of `ParentRenderArgs`.
                    */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> ParentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
        ParentRenderArgsBuilder {
            fields: (),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
/**Builder for [`ParentRenderArgs`] instances.

See [`ParentRenderArgs::builder()`] for more info.*/
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct ParentRenderArgsBuilder<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> {
    fields: TypedBuilderFields,
    phantom: (
        ::core::marker::PhantomData<&'RENDER ()>,
        ::core::marker::PhantomData<&'a ()>,
        ::core::marker::PhantomData<&'bump ()>,
    ),
}
impl<'RENDER, 'a, 'bump: 'RENDER, TypedBuilderFields> Clone
for ParentRenderArgsBuilder<'RENDER, 'a, 'bump, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
pub trait ParentRenderArgsBuilder_Optional<T> {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T;
}
impl<T> ParentRenderArgsBuilder_Optional<T> for () {
    fn into_value<F: FnOnce() -> T>(self, default: F) -> T {
        default()
    }
}
impl<T> ParentRenderArgsBuilder_Optional<T> for (T,) {
    fn into_value<F: FnOnce() -> T>(self, _: F) -> T {
        self.0
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
impl<'RENDER, 'a, 'bump: 'RENDER> ParentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
    ///Finalise the builder and create its [`ParentRenderArgs`] instance
    #[allow(clippy::default_trait_access)]
    pub fn build(self) -> ParentRenderArgs<'RENDER, 'a, 'bump> {
        let () = self.fields;
        let __Asteracea__phantom = ::core::default::Default::default();
        ParentRenderArgs {
            __Asteracea__phantom,
        }
    }
}
#[allow(non_snake_case)]
struct Parent {
    __Asteracea__0: Container,
    __Asteracea__1: Content,
    __Asteracea__pinned: ::std::marker::PhantomPinned,
}
impl Parent {
    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn __Asteracea__0_pinned(
        self: ::std::pin::Pin<&Self>,
    ) -> ::std::pin::Pin<&Container> {
        unsafe { self.map_unchecked(|this| &this.__Asteracea__0) }
    }
    #[allow(non_snake_case)]
    #[allow(dead_code)]
    fn __Asteracea__1_pinned(self: ::std::pin::Pin<&Self>) -> ::std::pin::Pin<&Content> {
        unsafe { self.map_unchecked(|this| &this.__Asteracea__1) }
    }
}
const _: fn() = || {
    trait AmbiguousIfImpl<A> {
        fn some_item() {}
    }
    impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
    {
        #[allow(dead_code)]
        struct Invalid;
        impl<T: ?Sized + ::std::marker::Unpin> AmbiguousIfImpl<Invalid> for T {}
    }
    let _ = <Parent as AmbiguousIfImpl<_>>::some_item;
};
impl Parent {
    /// <!-- (suppress `missing_docs`) -->
    pub fn new<'parent_resource_node_borrow, 'a>(
        parent_node: ::core::pin::Pin<
            &'parent_resource_node_borrow ::asteracea::include::dependency_injection::ResourceNode,
        >,
        args: ParentNewArgs<'_, 'a>,
    ) -> ::std::result::Result<
            (
                Self,
                ::asteracea::include::dependency_injection::SparseResourceNodeHandle<
                    'parent_resource_node_borrow,
                >,
            ),
            ::asteracea::error::Escalation,
        >
    where
        Self: 'a + 'static,
    {
        let (ParentNewArgs { __Asteracea__phantom: _ }, ()) = (args, ());
        let mut resource_node = ::asteracea::include::dependency_injection::ResourceBob::new_for::<
            Self,
        >(parent_node);
        let local_resource_node = unsafe {
            ::core::pin::Pin::new_unchecked(&mut resource_node)
        };
        {}
        {}
        let resource_node = resource_node.into_sparse_handle();
        ::std::result::Result::Ok((
            {
                let resource_node = resource_node.as_ref();
                {
                    let sparse_resource_node_0: ::asteracea::include::dependency_injection::SparseResourceNodeHandle;
                    let sparse_resource_node_1: ::asteracea::include::dependency_injection::SparseResourceNodeHandle;
                    Parent {
                        __Asteracea__0: ({
                            let child_constructed = Container::new(
                                resource_node.as_ref(),
                                Container::new_args_builder().build(),
                            )?;
                            sparse_resource_node_0 = child_constructed.1;
                            child_constructed.0
                        }),
                        __Asteracea__1: ({
                            let child_constructed = Content::new(
                                sparse_resource_node_0.as_ref(),
                                Content::new_args_builder().build(),
                            )?;
                            sparse_resource_node_1 = child_constructed.1;
                            child_constructed.0
                        }),
                        __Asteracea__pinned: ::std::marker::PhantomPinned,
                    }
                }
            },
            resource_node,
        ))
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn new_args_builder<'NEW, 'a: 'NEW>() -> ParentNewArgsBuilder<'NEW, 'a, ()> {
        ParentNewArgs::builder()
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render<'a, 'bump>(
        self: ::std::pin::Pin<&'a Self>,
        bump: &'bump asteracea::bumpalo::Bump,
        args: ParentRenderArgs<'_, 'a, 'bump>,
    ) -> ::std::result::Result<
            ::asteracea::lignin::Guard<'bump, ::asteracea::lignin::ThreadSafe>,
            ::asteracea::error::Escalation,
        > {
        let ParentRenderArgs { __Asteracea__phantom: _ } = args;
        let mut on_vdom_drop: ::core::option::Option<
            ::asteracea::lignin::guard::ConsumedCallback,
        > = None;
        let this = self;
        ::std::result::Result::Ok(
            ::asteracea::lignin::Guard::new(
                {
                    let rendered = this
                        .__Asteracea__0_pinned()
                        .render(
                            bump,
                            Container::render_args_builder()
                                .__Asteracea__anonymous_content((
                                    {
                                        let phantom = [];
                                        if false {
                                            <[_; 0] as ::core::iter::IntoIterator>::into_iter(phantom)
                                                .next()
                                                .unwrap()
                                        } else {
                                            ::asteracea::__::infer_builder(phantom).build()
                                        }
                                    },
                                    ::std::boxed::Box::new(|
                                        bump: &'bump ::asteracea::bumpalo::Bump,
                                    | -> ::std::result::Result<
                                        _,
                                        ::asteracea::error::Escalation,
                                    > {
                                        let mut on_vdom_drop: ::core::option::Option<
                                            ::asteracea::lignin::guard::ConsumedCallback,
                                        > = None;
                                        ::core::result::Result::Ok(
                                            ::asteracea::lignin::Guard::new(
                                                {
                                                    let rendered = this
                                                        .__Asteracea__1_pinned()
                                                        .render(bump, Content::render_args_builder().build())?;
                                                    let guard = unsafe {
                                                        use asteracea::lignin::guard::auto_safety::Deanonymize;
                                                        (&&::core::mem::ManuallyDrop::new(rendered)).deanonymize()
                                                    };
                                                    let vdom = unsafe {
                                                        guard
                                                            .peel(
                                                                &mut on_vdom_drop,
                                                                || bump.alloc_with(|| ::core::mem::MaybeUninit::uninit()),
                                                            )
                                                    };
                                                    vdom
                                                },
                                                on_vdom_drop,
                                            ),
                                        )
                                    }),
                                ))
                                .build(),
                        )?;
                    let guard = unsafe {
                        use asteracea::lignin::guard::auto_safety::Deanonymize;
                        (&&::core::mem::ManuallyDrop::new(rendered)).deanonymize()
                    };
                    let vdom = unsafe {
                        guard
                            .peel(
                                &mut on_vdom_drop,
                                || bump.alloc_with(|| ::core::mem::MaybeUninit::uninit()),
                            )
                    };
                    vdom
                },
                on_vdom_drop,
            ),
        )
    }
    /// <!-- (suppress `missing_docs`) -->
    pub fn render_args_builder<'RENDER, 'a, 'bump: 'RENDER>() -> ParentRenderArgsBuilder<
            'RENDER,
            'a,
            'bump,
            (),
        > {
        ParentRenderArgs::builder()
    }
    #[doc(hidden)]
    pub fn __Asteracea__ref_render_args_builder<'RENDER, 'a, 'bump: 'RENDER>(
        &self,
    ) -> ParentRenderArgsBuilder<'RENDER, 'a, 'bump, ()> {
        let _ = self;
        ParentRenderArgs::builder()
    }
}
/// Asteracea components do not currently support custom [`Drop`](`::std::ops::Drop`) implementations.
impl ::std::ops::Drop for Parent {
    fn drop(&mut self) {
        unsafe {}
    }
}
