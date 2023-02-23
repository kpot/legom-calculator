//! Небольшое количество элементов, таких как формулы веществ, определённые символы,
//! которые повторяются снова и снова в различных компонентах.

use yew::{function_component, html, Html};

use crate::calculator::ElemName;

pub const CROSS_MARK: &str = " \u{2718}"; // ✘ symbol
pub const CHECK_MARK: &str = " \u{2713}"; // ✓ symbol
pub const MDASH: &str = "\u{2014}"; // — symbol

#[function_component]
pub fn PhosphorusOxide() -> Html {
    html! {<>{"P"}<sub>{"2"}</sub>{"O"}<sub>{"5"}</sub></>}
}

#[function_component]
pub fn PotassiumOxide() -> Html {
    html! {
        <>{"K"}<sub>{"2"}</sub>{"O"}</>
    }
}

pub(crate) fn nutrient_css_class(element: ElemName) -> &'static str {
    match element {
        ElemName::Nitrogen => "nutrient-N",
        ElemName::Phosphorus => "nutrient-P",
        ElemName::Potassium => "nutrient-K",
        ElemName::Magnesium => "nutrient-Mg",
    }
}

pub(crate) fn nutrient_input_css_class(element: ElemName) -> &'static str {
    match element {
        ElemName::Nitrogen => "nutrient-N-input",
        ElemName::Phosphorus => "nutrient-P-input",
        ElemName::Potassium => "nutrient-K-input",
        ElemName::Magnesium => "nutrient-Mg-input",
    }
}
