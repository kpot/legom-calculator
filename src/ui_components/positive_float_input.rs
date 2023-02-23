use stylist::yew::styled_component;
use web_sys::HtmlInputElement;
use yew::{classes, html, use_node_ref, AttrValue, Callback, Html, NodeRef, Properties};

use crate::yew_utils::include_css;

#[derive(Properties, PartialEq)]
pub(crate) struct FloatInputProps {
    pub on_value_change: Callback<Option<f64>>,
    pub id: Option<AttrValue>,
    pub required: bool,
    pub placeholder: Option<&'static str>,
    pub value: Option<f64>,
    pub title: Option<AttrValue>,
    pub size: Option<AttrValue>,
    pub enforce_min: Option<f64>,
    pub enforce_max: Option<f64>,
    pub node_ref: Option<NodeRef>,
    pub class: Option<&'static str>,
}

fn optional_text_to_float(
    text: &str,
    default: Option<f64>,
    enforce_min: Option<f64>,
    enforce_max: Option<f64>,
) -> Option<f64> {
    if text.is_empty() {
        enforce_min
    } else {
        let normalized: String = text
            .chars()
            .map(|c| if c.is_numeric() { c } else { '.' })
            .collect();
        normalized
            .trim_end_matches('.')
            .parse::<f64>()
            .ok()
            .map(|num| {
                let mut forced = num.abs();
                if let Some(max) = enforce_max {
                    if forced > max {
                        forced = max;
                    }
                }
                if let Some(min) = enforce_min {
                    if forced < min {
                        forced = min;
                    }
                }
                forced
            })
            .or(default)
    }
}

/// Создаёт обработчик для события изменения поля ввода, который, независимо от типа
/// события, считывает значение поля, преобразует его в число (или None), и вызывает
/// какой-то другой обработчик, уже с этим номером.
fn make_change_handler<EventType>(
    input_ref: NodeRef,
    on_update: Callback<Option<f64>>,
    value: Option<f64>,
    enforce_min: Option<f64>,
    enforce_max: Option<f64>,
) -> impl Fn(EventType) {
    move |_: EventType| {
        if let Some(input) = input_ref.cast::<HtmlInputElement>() {
            let entered_text = input.value();
            let parsed_value =
                optional_text_to_float(&entered_text, value, enforce_min, enforce_max);
            on_update.emit(parsed_value);
        }
    }
}

/// Поле ввода для дробных значений, повсеместно используемое в калькуляторе.
/// Решение не самое элегантное, так как может автоматически вписывать корректную наибольшее
/// или наименьшее допустимое значение, что может запутать пользователя. Но, зато прост, не требует
/// валидации и сообщений об ошибках.
#[styled_component]
pub(crate) fn PositiveFloatInput(props: &FloatInputProps) -> Html {
    let local_node_ref = use_node_ref();
    let input_ref = props.node_ref.clone().unwrap_or(local_node_ref);

    let stylesheet = include_css!("positive_float_input.css");

    let on_num_keydown = Callback::from(|e: web_sys::KeyboardEvent| {
        let code = e.key_code();
        let key = e.key();
        let good_code = match code {
            8..=39 | 45 | 46 | 48..=57 | 96..=105 | 110 | 116 | 118 | 229 => true,
            188 | 190 if key == "." || key == "," => true,
            _ => false,
        };
        if !good_code {
            e.prevent_default()
        }
    });

    let on_change_cb = make_change_handler(
        input_ref.clone(),
        props.on_value_change.clone(),
        props.value,
        props.enforce_min,
        props.enforce_max,
    );

    let on_input_keyup = make_change_handler(
        input_ref.clone(),
        props.on_value_change.clone(),
        props.value,
        props.enforce_min,
        props.enforce_max,
    );

    html! {
        <input type="number" step="any"
            min={props.enforce_min.unwrap_or_default().to_string()}
            max={props.enforce_max.map(|v| v.to_string())}
            class={classes!(stylesheet, "form-control", props.class)}
            id={&props.id}
            required={props.required}
            placeholder={props.placeholder}
            ref={&input_ref}
            onchange={on_change_cb}
            onkeydown={&on_num_keydown}
            onkeyup={on_input_keyup}
            value={props.value.map(|v| v.to_string())}
            title={&props.title}
            size={&props.size} />
    }
}
