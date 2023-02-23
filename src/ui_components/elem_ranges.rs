use std::rc::Rc;

use stylist::style;
use stylist::yew::styled_component;
use yew::prelude::*;

use crate::calculator::{ElemRange, ElemRangeName, ElemRatios};
use crate::ui_components::html_chunks::{PhosphorusOxide, PotassiumOxide};
use crate::ui_components::positive_float_input::PositiveFloatInput;
use crate::yew_utils::make_element_id;

const MIN_RANGE_DELTA: f64 = 0.01;

#[derive(PartialEq, Properties, Clone)]
pub(crate) struct ElemRangeProps {
    name: ElemRangeName,
    id: AttrValue,
    range: ElemRange,
    onchange: Callback<(ElemRangeName, ElemRange)>,
}

/// Отдельный диапазон значений "от" и "до", включающий два поля ввода, синхронизированных
/// так, чтобы правое значение всегда было больше левого как минимум на на `MIN_RANGE_DELTA`.
#[function_component(ElemRangeInput)]
pub(crate) fn elem_range_input(props: &ElemRangeProps) -> Html {
    let on_lower_bound_change = use_callback(
        |entered_value, (range_name, upper_bound, onchange)| {
            if let Some(value) = entered_value {
                let new_range = ElemRange { from: value, to: *upper_bound };
                if new_range.is_valid() {
                    onchange.emit((*range_name, new_range));
                }
            }
        },
        (props.name, props.range.to, props.onchange.clone()),
    );

    let on_upper_bound_change = use_callback(
        |entered_upper_value, (range_name, lower_bound, onchange)| {
            if let Some(value) = entered_upper_value {
                let new_range = ElemRange { from: *lower_bound, to: value };
                if new_range.is_valid() {
                    onchange.emit((*range_name, new_range));
                }
            }
        },
        (props.name, props.range.from, props.onchange.clone()),
    );

    html! {
        <div class="input-group">
            <PositiveFloatInput
                id={&props.id}
                enforce_min={Some(MIN_RANGE_DELTA)}
                enforce_max={Some((props.range.to - MIN_RANGE_DELTA).max(MIN_RANGE_DELTA))}
                on_value_change={&on_lower_bound_change}
                required={true}
                value={Some(props.range.from)}
                />
            <span class="input-group-text">{"–"}</span>
            <PositiveFloatInput
                enforce_min={Some(props.range.from + MIN_RANGE_DELTA)}
                on_value_change={&on_upper_bound_change}
                required={true}
                value={Some(props.range.to)}
                />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct NutrientRatiosProps {
    pub ratios: Rc<ElemRatios>,
    pub on_ratio_change: Callback<(ElemRangeName, ElemRange)>,
}

/// Отвечает за ввод трёх диапазонов, каждый задающий соотношение одного макро-элемента питания
/// к фосфору.
#[styled_component]
pub(crate) fn NutrientRatios(props: &NutrientRatiosProps) -> Html {
    let n_input_id = use_memo(make_element_id("n-ratio"), ());
    let k_input_id = use_memo(make_element_id("k-ratio"), ());
    let mg_input_id = use_memo(make_element_id("mg-ratio"), ());

    let stylesheet = style!(
        input[type=number] { max-width: ${"5em"}; min-width: ${"3.6em"}; }
        label small { display: block; }
        .input-group-text { padding-left: 4px; padding-right: 4px; }
    )
    .expect("Must have valid CSS");

    html! {
        <div class={stylesheet}>
            <h2>{ "Шаг 2. Баланс элементов в смеси" }
            <small class="text-muted">{" (можно пропустить)"}</small>
            </h2>
            <p>{ "Здесь приведены соотношения, рекомендуемые Т.Ю.Угаровой \
                  (минимум-максимум для каждого элемента). Но если вы опытный овощевод, \
                  можете внести свои коррективы" }</p>
            <div class="container-fluid">
                <div class="row">
                    <div class="col-2">
                        <label class="form-label">
                            <span>
                                {"фосфор "}
                                <small class="text-muted">{"("}<PhosphorusOxide />{")"}</small>
                            </span>
                        </label>
                        <p class="fs-2">{"1"}</p>
                    </div>

                    <div class="col">
                        <label for={n_input_id.as_ref()} class="form-label">
                            { "азот "}<small class="text-muted">{"(N)"}</small>
                        </label>
                        <ElemRangeInput
                            id={(*n_input_id).clone()}
                            name={ElemRangeName::Nitrogen}
                            range={props.ratios.n_to_p}
                            onchange={&props.on_ratio_change} />
                    </div>

                    <div class="col">
                        <label for={k_input_id.as_ref()} class="form-label">
                            <span>
                                {"калий "}
                                <small class="text-muted">{"("}<PotassiumOxide />{")"}</small>
                            </span>
                        </label>
                        <ElemRangeInput
                            id={(*k_input_id).clone()}
                            name={ElemRangeName::Potassium}
                            range={props.ratios.k_to_p}
                            onchange={&props.on_ratio_change} />
                    </div>

                    <div class="col">
                        <label for={mg_input_id.as_ref()} class="form-label">
                        { "магний "}<small class="text-muted">{"(MgO)"}</small>
                        </label>
                        <ElemRangeInput
                            id={(*mg_input_id).clone()}
                            name={ElemRangeName::Magnesium}
                            range={props.ratios.mg_to_p}
                            onchange={&props.on_ratio_change} />
                    </div>
                </div>
            </div>
        </div>
    }
}
