//! Transformer to be used by markdown renderer

use std::rc::Rc;

use yew::Html;

#[derive(Debug)]
pub enum TransformRequest {
    Link {
        url: String,
        title: String,
        content: Html,
    },
    Image {
        url: String,
        title: String,
    },
}

pub type TransformResponse = Option<Html>;

/// A transformer is a callback-like thing.
///
/// Whenever the Markdown renderer encounters a link or image which
/// would have been resolved via the broken-link callback, we pass
/// that information to the given transformer which can either return
/// a replacement [`Html`] or else nothing, in which case the renderer
/// will indicate the short-code was unknown/bad
#[derive(Clone)]
pub struct Transformer {
    func: Rc<dyn Fn(TransformRequest) -> TransformResponse>,
}

impl PartialEq for Transformer {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::vtable_address_comparisons)]
        Rc::ptr_eq(&self.func, &other.func)
    }
}

impl<F: Fn(TransformRequest) -> TransformResponse + 'static> From<F> for Transformer {
    fn from(func: F) -> Self {
        Self {
            func: Rc::new(func),
        }
    }
}

impl Transformer {
    pub(crate) fn transform(&self, req: TransformRequest) -> TransformResponse {
        (*self.func)(req)
    }
}
