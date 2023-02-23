use std::rc::Rc;

use stylist::yew::styled_component;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::calculator::consts::PERMANENT_FERTILIZERS;
use crate::calculator::{Deficites, ElemName, Fertilizer};
use crate::store::{AppStore, PermanentFertilizersState, StoreAction};
use crate::ui_components::html_chunks::nutrient_css_class;
use crate::ui_components::positive_float_input::PositiveFloatInput;
use crate::yew_utils::include_css;

use super::html_chunks::MDASH;

#[derive(Properties, PartialEq)]
struct KnownFertRowProp {
    fertilizer: Fertilizer,
    selected: bool,
    limit: Option<f64>,
    on_limit_change: Callback<(usize, Option<f64>)>,
    on_toggle: Callback<(usize, bool)>,
}

#[function_component]
fn KnownFertRow(
    KnownFertRowProp { fertilizer, selected, limit, on_toggle, on_limit_change }: &KnownFertRowProp,
) -> Html {
    let checkbox_input_ref = use_node_ref();
    let fert_input_id = AttrValue::from(format!("perma-fert-checkbox-{}", fertilizer.id));

    let on_checkbox_change = {
        let checkbox_input_ref = checkbox_input_ref.clone();
        let fert_id = fertilizer.id;
        let on_toggle = on_toggle.clone();
        Callback::from(move |_| {
            if let Some(input) = checkbox_input_ref.cast::<HtmlInputElement>() {
                on_toggle.emit((fert_id, input.checked()));
            }
        })
    };

    let on_limit_change = {
        let fert_id = fertilizer.id;
        let on_limit_change = on_limit_change.clone();
        Callback::from(move |new_value: Option<f64>| {
            on_limit_change.emit((fert_id, new_value));
        })
    };

    let nutrient_column = |name: &str, element: ElemName| -> Html {
        html! {
            <div class={classes!("col", nutrient_css_class(element))}>
                <span class="fert-nutrient">
                    { name }<br/><span>{fertilizer[element].to_string()}{"%"}</span>
                </span>
            </div>
        }
    };

    html! {
        <div key={fertilizer.id} class={classes!("container-fluid", selected.then_some("select"))}>
            <div class="row pt-1">
                <div class="col">
                    <div class="form-check">
                        <input type="checkbox" class="form-check-input mt-2"
                            ref={checkbox_input_ref}
                            id={fert_input_id.clone()}
                            checked={*selected}
                            onchange={on_checkbox_change} />
                        <label for={fert_input_id} class="form-check-label fs-5">
                            { fertilizer.name.clone() }
                        </label>
                    </div>
                </div>
            </div>
            <div class="row pb-1 ps-4">
                {nutrient_column("Азот", ElemName::Nitrogen)}
                {nutrient_column("Фосфор", ElemName::Phosphorus)}
                {nutrient_column("Калий", ElemName::Potassium)}
                {nutrient_column("Магний", ElemName::Magnesium)}
                <div class="col g-0 fert-remainder">
                    <PositiveFloatInput
                        placeholder="∞ кг."
                        size="5"
                        value={limit}
                        on_value_change={on_limit_change}
                        required={false}
                        title="Сколько удобрения у вас осталось" />
                </div>
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub(crate) struct KnownFertilizersProps {
    pub store_dispatcher: UseReducerDispatcher<AppStore>,
    pub fertilizers_status: Rc<PermanentFertilizersState>,
    pub deficites: Rc<Deficites>,
}

/// Перечисляет все предопределённые (распространённые и часто используемые) удобрения,
/// позволяя выбрать некоторые из них, а также указать ограничение на доступное количество
#[styled_component(KnownFertilizers)]
pub(crate) fn known_fertilizers(props: &KnownFertilizersProps) -> Html {
    let stylesheet = include_css!("known_fertilizers.css");

    let on_fertilizer_toggle = {
        let store_dispatcher = props.store_dispatcher.clone();
        Callback::from(move |(fertilizer_id, state)| {
            store_dispatcher.dispatch(StoreAction::ToggleFertilizer(fertilizer_id, state));
        })
    };

    let on_limit_change = {
        let store_dispatcher = props.store_dispatcher.clone();
        Callback::from(move |(fert_id, fert_limit)| {
            store_dispatcher.dispatch(StoreAction::UpdatePermanentLimit(fert_id, fert_limit));
        })
    };

    let rendered_fertilizers = PERMANENT_FERTILIZERS.iter().map(|fertilizer| {
        html! {
            <KnownFertRow
                key={fertilizer.id}
                fertilizer={fertilizer.clone()}
                limit={props.fertilizers_status.get_limit(fertilizer.id)}
                selected={props.fertilizers_status.is_selected(fertilizer.id)}
                on_toggle={&on_fertilizer_toggle}
                on_limit_change={&on_limit_change} />
        }
    });

    html! {
        <>
            <h2>{"Шаг 1. Укажите доступные вам удобрения"}<br />
                <small class="text-muted">
                {"(чем больше "}{MDASH}{" тем лучше, следите за балансом в нижней панели)"}
                </small>
            </h2>
            <p>
                <strong>{"Удобно:"}</strong>
                {" Если какого-то удобрения у вас осталось недостаточно, \
                  укажите его остаток в поле "}
                <span class="input-field-reference">{"∞ кг."}</span>
                {", добавьте другое аналогичное удобрение в смесь, и калькулятор покроет нехватку \
                  за счёт аналога."}
            </p>
            <div class={classes!(stylesheet, "p-1")}>
                {for rendered_fertilizers}
            </div>
        </>
    }
}
