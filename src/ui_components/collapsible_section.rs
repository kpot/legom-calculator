use yew::{classes, html, use_state, AttrValue, Callback, Children, Html, Properties};

use stylist::{style, yew::styled_component};

#[derive(PartialEq, Properties)]
pub struct CollapsibleSectionProps {
    pub description: AttrValue,
    pub children: Children,
    pub visible: Option<bool>,
    pub printable: Option<bool>,
}

/// Используется для сворачиваемых сносок, которые при многократном использовании калькулятора
/// будут только занимать место на экране
#[styled_component]
pub fn CollapsibleSection(props: &CollapsibleSectionProps) -> Html {
    let css_display_in_print = if props.printable.unwrap_or_default() {
        "block"
    } else {
        "none"
    };
    let stylesheet = style!(
        & {border: 1px dashed silver; border-radius: var(--bs-border-radius);}
        & > a { color: gray; text-decoration: underline; display: block; padding: ${"1em"}; cursor: pointer; }
        & > a::after {content: " ▼";}
        & > a.active:after {content: " ▲";}
        & > div { background-color: lightgoldenrodyellow; padding: ${"1em"}; }
        @media only screen {
            & > div {display: none;}
            & > div.active {display: block;}
        }
        @media only print {
            & { display: ${css_display_in_print}; }
        }
    )
    .expect("The stylesheet has to be valid");

    let is_section_visible = use_state(|| props.visible.unwrap_or_default());
    let on_description_click = {
        let is_section_visible = is_section_visible.clone();
        Callback::from(move |_| {
            is_section_visible.set(!*is_section_visible);
        })
    };

    html! {
        <div class={stylesheet}>
            <a onclick={&on_description_click} class={classes!(is_section_visible.then_some("active"))}>
                {&props.description}
            </a>
            <div class={classes!(is_section_visible.then_some("active"))}>
            {for props.children.iter() }
            </div>
        </div>
    }
}
