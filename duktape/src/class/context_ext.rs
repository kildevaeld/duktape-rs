use super::super::context::Context;
use super::super::error::*;
use super::super::types::function::Function;
use super::builder::*;

pub trait ContextClassBuilder {
    fn create_class<'a, M: Method + 'static>(
        &'a self,
        method: M,
        parent: Option<Function>,
    ) -> DukResult<ClassBuilder<'a>>;
}

impl ContextClassBuilder for Context {
    fn create_class<'a, M: Method + 'static>(
        &'a self,
        method: M,
        parent: Option<Function>,
    ) -> DukResult<ClassBuilder<'a>> {
        ClassBuilder::new(self, method, parent)
    }
}
