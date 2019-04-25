use super::argument_list::ArgumentList;
use super::callable::Callable;
use super::context::{Context, Idx, PropertyFlag};
use super::error::DukResult;
use super::from_context::FromDuktape;
use super::object::Object;
use super::reference::{JSValue, Reference};
use super::to_context::ToDuktape;

pub struct PropertyBuilder<'a, V: ToDuktape> {
    flags: PropertyFlag,
    setter: Option<Box<dyn Callable>>,
    getter: Option<Box<dyn Callable>>,
    value: Option<V>,
    name: &'a str,
}

impl<'a, V: ToDuktape> PropertyBuilder<'a, V> {
    pub fn configurable(mut self, on: bool) -> Self {
        if on {
            self.flags |= PropertyFlag::DUK_DEFPROP_SET_CONFIGURABLE;
        } else {
            self.flags |= PropertyFlag::DUK_DEFPROP_CLEAR_CONFIGURABLE;
        }
        self
    }

    pub fn enumerable(mut self, on: bool) -> Self {
        if on {
            self.flags |= PropertyFlag::DUK_DEFPROP_SET_ENUMERABLE;
        } else {
            self.flags |= PropertyFlag::DUK_DEFPROP_CLEAR_ENUMERABLE;
        }
        self
    }

    pub fn writable(mut self, on: bool) -> Self {
        if on {
            self.flags |= PropertyFlag::DUK_DEFPROP_SET_WRITABLE;
        } else {
            self.flags |= PropertyFlag::DUK_DEFPROP_CLEAR_WRITABLE;
        }
        self
    }

    pub fn setter<C: Callable + 'static>(mut self, setter: C) -> Self {
        self.clear_value();
        self.flags |= PropertyFlag::DUK_DEFPROP_HAVE_SETTER;
        self.setter = Some(Box::new(setter));
        self
    }

    pub fn getter<C: Callable + 'static>(mut self, getter: C) -> Self {
        self.clear_value();
        self.flags |= PropertyFlag::DUK_DEFPROP_HAVE_GETTER;
        self.setter = Some(Box::new(getter));
        self
    }

    pub fn value<Val: ToDuktape + 'static>(mut self, value: Val) -> PropertyBuilder<'a, Val> {
        self.clear_getter();
        self.clear_setter();
        self.flags |= PropertyFlag::DUK_DEFPROP_HAVE_VALUE;
        // self.value = val;
        PropertyBuilder {
            flags: self.flags,
            setter: None,
            getter: None,
            value: Some(value),
            name: self.name,
        }
    }

    pub fn clear_getter(&mut self) {
        self.flags.remove(PropertyFlag::DUK_DEFPROP_HAVE_GETTER);
        self.getter = None;
    }

    pub fn clear_setter(&mut self) {
        self.flags.remove(PropertyFlag::DUK_DEFPROP_HAVE_SETTER);
        self.setter = None;
    }
    pub fn clear_value(&mut self) {
        self.flags.remove(PropertyFlag::DUK_DEFPROP_HAVE_VALUE);
        self.value = None;
    }

    pub(crate) fn build(self, ctx: &Context, idx: Idx) -> DukResult<()> {
        ctx.push_string(self.name);
        if let Some(getter) = self.getter {
            ctx.push_function(getter);
        }

        if let Some(setter) = self.setter {
            ctx.push_function(setter);
        }

        if let Some(value) = self.value {
            value.to_context(ctx)?;
        }
        ctx.def_prop(idx, self.flags)?;

        Ok(())
    }
}

pub struct Property<'a> {
    pub(crate) _ref: Reference<'a>,
    pub(crate) prop: &'a str,
}

impl<'a> Property<'a> {
    pub fn build<'b>(name: &'b str) -> PropertyBuilder<'b, ()> {
        PropertyBuilder {
            flags: PropertyFlag::default(),
            setter: None,
            getter: None,
            value: None,
            name: name,
        }
    }

    pub fn call<A: ArgumentList, R: FromDuktape<'a>>(&self, args: A) -> DukResult<R> {
        self._ref.push();
        let idx = self._ref.ctx().normalize_index(-1);
        self._ref.ctx().push_string(self.prop);
        let len = args.len();
        args.push_args(self._ref.ctx())?;
        if let Err(e) = self._ref.ctx().call_prop(idx, len) {
            self._ref.ctx().pop(1);
            return Err(e);
        }
        self._ref.ctx().remove(-2);
        let ret = R::from_context(self._ref.ctx(), -1);
        self._ref.ctx().pop(1);
        ret
    }

    /// Construct a property on the object
    pub fn construct<A: ArgumentList>(&self, args: A) -> DukResult<Object<'a>> {
        self._ref.push();
        if !self._ref.ctx().has_prop_string(-1, self.prop) {
            return duk_type_error!("not a function");
        }

        let len = args.len();

        self._ref.ctx().get_prop_string(-1, self.prop);
        args.push_args(self._ref.ctx())?;
        if let Err(e) = self._ref.ctx().construct(len) {
            self._ref.ctx().pop(2);
            return Err(e);
        }

        self._ref.ctx().remove(-2);
        let o = Object::from_context(self._ref.ctx(), -1);
        self._ref.ctx().pop(1);
        o
    }
}
