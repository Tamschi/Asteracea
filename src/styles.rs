use {
    crate::rhizome::extractable,
    lazy_static::lazy_static,
    lignin_schema::{
        div,
        lignin::{bumpalo::Bump, Attribute, Node},
    },
    std::{collections::HashMap, sync::Mutex},
};

//TODO: This should be default-provided at root, but that requires a stateful implementation (i.e. captures).
//TODO: Reference vdom_provider and kick off a render when a style is added.
extractable! {
    pub abstract trait Styles

    // This would be nicer with 'bump, but that appears to count as generic when using higher kinded types to express it.
    fn add_boxed(&self, key: &str, factory: Box<dyn FnOnce(&'static Bump) -> Node<'static>>);

    fn render<'bump>(&self, bump: &'bump Bump) -> Node<'bump>;
}

impl dyn Styles {
    pub fn add(
        &self,
        key: &str,
        factory: impl for<'bump> FnOnce(&'bump Bump) -> Node<'bump> + 'static,
    ) {
        //TODO?: Constrain to style elements.
        self.add_boxed(key, Box::new(factory))
    }
}

pub struct TempImpl {
    bump: &'static Bump,
    styles: Mutex<HashMap<String, Node<'static>>>,
}

impl TempImpl {
	#[must_use]
	pub fn new() -> Self {
        Self {
            bump: Box::leak(Box::default()),
            styles: Mutex::default(),
        }
    }
}

impl Default for TempImpl {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    static ref ATTRIBUTES: [Attribute<'static>; 1] = [Attribute {
        name: "style",
        value: "display: none;"
    }];
}
impl Styles for TempImpl {
    fn add_boxed(&self, key: &str, factory: Box<dyn FnOnce(&'static Bump) -> Node<'static>>) {
        let mut styles = self.styles.lock().unwrap();
        if !styles.contains_key(key) {
            styles.insert(key.to_owned(), factory(self.bump));
        }
    }

    fn render<'bump>(&self, bump: &'bump Bump) -> Node<'bump> {
        let guard = self.styles.lock().unwrap();
        let children = bump.alloc_slice_fill_iter(guard.values().copied());
        let event_bindings = bump.alloc_with(|| []);
        (bump.alloc_with(move || div(&*ATTRIBUTES, &*children, &*event_bindings))).into()
    }
}
