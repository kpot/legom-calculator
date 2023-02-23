use crate::{ui_components::positive_float_input::PositiveFloatInput, yew_utils::make_element_id};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct TotalMassInputProps {
    pub value: f64,
    pub on_change: Callback<f64>,
}

#[function_component(TotalMassInput)]
pub(crate) fn total_mass_input(props: &TotalMassInputProps) -> Html {
    let input_id = use_memo(make_element_id("weight-total"), ());
    let on_input_change = use_callback(
        |value: Option<f64>, on_change| {
            if let Some(value) = value {
                if value > 0.0 {
                    on_change.emit(value)
                }
            }
        },
        props.on_change.clone(),
    );
    html! {
        <>
        <h2>{"Шаг 3. Сколько смеси вам нужно?"}</h2>
        <div class="row mx-auto mb-5">
            <div class="col-auto g-0">
                <label for="{&*input_id}" class="col-form-label">{"Мне нужно"}</label>
            </div>
            <div class="col-auto">
                <div class="input-group">
                    <PositiveFloatInput
                        size="3"
                        id={(*input_id).clone()}
                        value={props.value}
                        on_value_change={on_input_change}
                        enforce_min={Some(0.0)}
                        required={true} />
                    <span class="input-group-text">{" кг."}</span>
                </div>
            </div>
        </div>
        </>
    }
}
