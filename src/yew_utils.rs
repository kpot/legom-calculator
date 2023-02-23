use std::fmt::Write;
use std::sync::atomic;

use yew::{AttrValue, Html};

macro_rules! include_css {
    ($i:literal) => {{
        const CSS_FILE_CONTENT: &str = include_str!($i);
        stylist::Style::new(CSS_FILE_CONTENT).expect(&format!("File {} must contain valid CSS", $i))
    }};
}
pub(crate) use include_css;

pub fn raw_html<T>(text: T) -> Html
where
    T: Into<AttrValue>,
{
    Html::from_html_unchecked(text.into())
}

pub struct FloatFormat {
    value: String,
}

/// Обёрка, которая "красиво" форматирует дробное значение, чтобы при отображении (через Display)
/// 1. не показывалось больше знаков после запятой, чем задано
/// 2. не отображать ненужные нули и саму запятую, если это не требуется для прочтения
impl FloatFormat {
    pub fn new(n: f64, precision: usize) -> Self {
        Self { value: format!("{:.*}", precision, n) }
    }

    fn trim(&self) -> &str {
        if let Some(dot_pos) = self.value.find('.') {
            let tail_to_leave = self.value[dot_pos..].trim_end_matches(['.', '0']).len();
            &self.value[..dot_pos + tail_to_leave]
        } else {
            self.value.as_str()
        }
    }
}

impl std::fmt::Display for FloatFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.trim())
    }
}

static NEXT_COMPONENT_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);

pub fn make_element_id<D>(prefix: &'static str) -> impl Fn(&D) -> AttrValue
where
    D: 'static + PartialEq,
{
    move |_: &D| {
        let new_id = NEXT_COMPONENT_ID.fetch_add(1, atomic::Ordering::Relaxed);
        let mut result = String::with_capacity(prefix.len() + 5);
        write!(result, "{}-{}", prefix, new_id).expect("Element ID must be formattable");
        result.into()
    }
}
