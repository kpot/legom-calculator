use std::rc::Rc;

use crate::calculator::{ElemName, Fertilizer};
use crate::store::AddedFertilizerAction;
use crate::ui_components::html_chunks::{nutrient_input_css_class, CROSS_MARK};
use crate::ui_components::positive_float_input::PositiveFloatInput;
use crate::yew_utils::{include_css, make_element_id, FloatFormat};
use gloo_storage::{LocalStorage, Storage};
use stylist::yew::styled_component;
use web_sys::HtmlInputElement;
use yew::prelude::*;

/// Ключ, используемый при хранении недавно добавленных удобрений в LocalStorage
const RECENT_FERTS_STORAGE_KEY: &str = "recently-added";
/// Максимальное число отображаемых удобрений в истории.
const MAX_RECENT_ADDED_DISPLAYED: usize = 10;
/// Максимиальное число удобрений, хранящихся в LocalStorage.
/// Должен быыть больше, чем MAX_RECENT_ADDED_DISPLAYED, поскольку
/// уже добавленные в смесь удобрения перестают показываться в истории.
const MAX_RECENT_ADDED_STORED: usize = 50;

#[derive(PartialEq, Properties)]
pub(crate) struct NewFertilizerInputProps {
    ondelete: Callback<usize>,
    onchange: Callback<Fertilizer>,
    edit: Fertilizer,
}

fn on_change_percentage_handler(
    edit: Fertilizer,
    onchange: Callback<Fertilizer>,
    element: ElemName,
) -> Callback<Option<f64>> {
    Callback::from(move |new_value: Option<f64>| {
        let mut edit = edit.clone();
        if let Some(new_unpacked_value) = new_value {
            edit[element] = new_unpacked_value;
        }
        onchange.emit(edit)
    })
}

#[function_component(NewFertilizerInput)]
pub(crate) fn new_fertilizer_input(
    NewFertilizerInputProps { ondelete, onchange, edit }: &NewFertilizerInputProps,
) -> Html {
    // Поля ввода, к которым мы будем обращаться напрямую
    let name_input_ref = use_node_ref();
    let cl_input_ref = use_node_ref();
    let s_input_ref = use_node_ref();

    // Идентификаторы всех полей, для связывания с метками <label>
    let n_input_id = use_memo(make_element_id("new-n-amount"), ());
    let p_input_id = use_memo(make_element_id("new-p-amount"), ());
    let k_input_id = use_memo(make_element_id("new-k-amount"), ());
    let mg_input_id = use_memo(make_element_id("new-mg-amount"), ());
    let cl_input_id = use_memo(make_element_id("new-cl-contains"), ());
    let s_input_id = use_memo(make_element_id("new-s-contains"), ());
    let limit_input_id = use_memo(make_element_id("limit-amount"), ());

    {
        // Установит фокус на имени удобрения при первом отображении компонента
        let main_input = name_input_ref.clone();
        use_effect_with_deps(
            move |_deps| {
                if let Some(input) = main_input.cast::<HtmlInputElement>() {
                    input.scroll_into_view();
                    input.focus().ok();
                }
            },
            (),
        );
    }

    let on_delete_pressed = {
        let ondelete = ondelete.clone();
        let fert_id = edit.id;
        Callback::from(move |_| ondelete.emit(fert_id))
    };

    let percentage_input_column =
        |input_id: Rc<AttrValue>, label: &str, element: ElemName| -> Html {
            let on_value_changed =
                on_change_percentage_handler(edit.clone(), onchange.clone(), element);
            html! {
            <div class="col">
                <label class="form-label" for={input_id.as_ref()}>{label}</label>
                <PositiveFloatInput size="10"
                    class={nutrient_input_css_class(element)}
                    on_value_change={on_value_changed}
                    id={input_id.as_ref()}
                    required={true}
                    value={ Some(edit[element]) }
                    enforce_min={Some(0.0)}
                    enforce_max={Some(100.0)} />
            </div>
            }
        };

    let on_name_changed = {
        let name_input_ref = name_input_ref.clone();
        let edit = edit.clone();
        let onchange = onchange.clone();
        Callback::from(move |_| {
            if let Some(input) = name_input_ref.cast::<HtmlInputElement>() {
                let entered_text = input.value();
                onchange.emit(Fertilizer { name: entered_text.into(), ..edit.clone() });
            }
        })
    };

    let on_limit_changed = {
        let edit = edit.clone();
        let onchange = onchange.clone();
        Callback::from(move |new_limit: Option<f64>| {
            if edit.limit != new_limit {
                onchange.emit(Fertilizer { limit: new_limit, ..edit.clone() });
            }
        })
    };

    let on_s_cl_checkbox_change = {
        let edit = edit.clone();
        let onchange = onchange.clone();
        let s_input_ref = s_input_ref.clone();
        let cl_input_ref = cl_input_ref.clone();
        Callback::from(move |_| {
            if let (Some(s_input), Some(cl_input)) = (
                s_input_ref.cast::<HtmlInputElement>(),
                cl_input_ref.cast::<HtmlInputElement>(),
            ) {
                onchange.emit(Fertilizer {
                    with_Cl: cl_input.checked(),
                    with_S: s_input.checked(),
                    ..edit.clone()
                });
            }
        })
    };

    html! {
        <div class="container-fluid pt-3 pb-2">
            <div class="row">
                <div class="col">
                    <input type="text" placeholder="Название"
                        class="new-fert-name form-control"
                        ref={&name_input_ref}
                        onchange={on_name_changed}
                        value={ edit.name.as_ref().to_owned() } />
                </div>
                <div class="col-auto">
                    <button type="button" class="new-fert-delete btn btn-danger"
                    onclick={on_delete_pressed}>{CROSS_MARK} { " удалить" }</button>
                </div>
            </div>
            <div class="row pt-2">
                {percentage_input_column(n_input_id, "Азот, %", ElemName::Nitrogen)}
                {percentage_input_column(p_input_id, "Фосфор, %", ElemName::Phosphorus)}
                {percentage_input_column(k_input_id, "Калий, %", ElemName::Potassium)}
                {percentage_input_column(mg_input_id, "Магний, %", ElemName::Magnesium)}
                <div class="col">
                    <label class="form-label" for={&*limit_input_id}>{"Наличие"}</label>
                    <PositiveFloatInput
                        size="10"
                        placeholder="∞ кг."
                        required={false}
                        id={limit_input_id.as_ref()}
                        on_value_change={on_limit_changed}
                        value={edit.limit} />
                </div>
            </div>
            <div class="row pt-2">
                <div class="col-auto">
                {"Если знаете:"}
                </div>
                <div class="col-auto">
                    <div class="form-check">
                        <input type="checkbox" class="form-check-input"
                            id={&*cl_input_id}
                            ref={&cl_input_ref}
                            checked={edit.with_Cl}
                            onchange={&on_s_cl_checkbox_change}
                            />
                        <label class="form-check-label" for={&*cl_input_id}>
                        {"Содержит хлориды"}
                        </label>
                    </div>
                </div>
                <div class="col-auto">
                    <div class="form-check">
                        <input type="checkbox" class="form-check-input"
                            id={&*s_input_id}
                            ref={&s_input_ref}
                            checked={edit.with_S}
                            onchange={&on_s_cl_checkbox_change}
                            />
                        <label class="form-check-label" for={&*s_input_id}>
                        {"Содержит сульфаты"}
                        </label>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub(crate) struct AddedFertilizersProps {
    pub on_change: Callback<AddedFertilizerAction>,
    pub fertilizers: Rc<Vec<Fertilizer>>,
}

/// Возвращает историю всех ранее добавленных вручную удобрений,
/// но не включая в историю удобрения, указанные в заданном списке
/// (используется для игнорирования тех удобрений, что уже добавлены к смеси).
fn top_history_of_added_fertilizers(exclusion_list: &[Fertilizer]) -> Vec<Fertilizer> {
    let excluded_ferts_content_ids: Vec<String> = exclusion_list
        .iter()
        .map(|item| item.content_id())
        .collect();

    let mut full_history = history_of_added_fertilizers();
    let to_skip = if MAX_RECENT_ADDED_DISPLAYED > full_history.len() {
        0
    } else {
        full_history.len() - MAX_RECENT_ADDED_DISPLAYED
    };
    let history = full_history
        .drain(..)
        .filter_map(|mut fertilizer| {
            if excluded_ferts_content_ids.contains(&fertilizer.content_id()) {
                None
            } else {
                fertilizer.re_id();
                fertilizer.limit = None;
                Some(fertilizer)
            }
        })
        .skip(to_skip)
        .collect();
    history
}

fn history_of_added_fertilizers() -> Vec<Fertilizer> {
    LocalStorage::get::<Vec<Fertilizer>>(RECENT_FERTS_STORAGE_KEY).unwrap_or_default()
}

/// Сохраняет все добавленные вручную удобрения в локально храняющуюся историю,
/// что позволит позднее добавлять их в смеси двумя нажатиями. История
/// ограничена по размеру.
pub(crate) fn store_history_of_added_fertilizers(newly_added: &[Fertilizer]) {
    let mut recent_fertilizers = history_of_added_fertilizers();
    let added_content_ids: Vec<String> = newly_added.iter().map(|f| f.content_id()).collect();
    // Мы должны сохранить добавленные вручную удобрения в историю, следя за тем,
    // чтобы последние добавленные удобрения попадали быть в конец истории,
    // дабы не быть вытесненными, поскольку отображаться будут не все элементы.
    let mut recent_fertilizers_deduplicated: Vec<Fertilizer> = recent_fertilizers
        .drain(..)
        .filter(|fertilizer| !added_content_ids.contains(&fertilizer.content_id()))
        .collect();
    recent_fertilizers_deduplicated.extend_from_slice(newly_added);
    let total_ferts = recent_fertilizers_deduplicated.len();
    let to_store = if total_ferts > MAX_RECENT_ADDED_STORED {
        &recent_fertilizers_deduplicated[total_ferts - MAX_RECENT_ADDED_STORED..]
    } else {
        &recent_fertilizers_deduplicated[..]
    };
    LocalStorage::set(RECENT_FERTS_STORAGE_KEY, to_store).ok();
}

#[styled_component]
pub(crate) fn AddedFertilizers(props: &AddedFertilizersProps) -> Html {
    let stylesheet = include_css!("added_fertilizers.css");

    let on_delete = use_callback(
        |fert_index, on_change| {
            on_change.emit(AddedFertilizerAction::Remove(fert_index));
        },
        props.on_change.clone(),
    );

    let on_edit = use_callback(
        |data, on_change| {
            on_change.emit(AddedFertilizerAction::Update(data));
        },
        props.on_change.clone(),
    );

    let on_add_fertilizer = use_callback(
        |_event, on_change| {
            on_change.emit(AddedFertilizerAction::Add);
        },
        props.on_change.clone(),
    );

    let list_of_added = props.fertilizers.iter().map(|fertilizer| {
        let ondelete = on_delete.clone();
        let onchange = on_edit.clone();
        html! {
            <NewFertilizerInput key={fertilizer.id} edit={fertilizer.clone()} {ondelete} {onchange} />
        }
    });

    let recent_fertilizers_not_added_yet = top_history_of_added_fertilizers(&props.fertilizers);
    let recent_history_is_empty = recent_fertilizers_not_added_yet.is_empty();
    let recent_fertilizers_rendered = use_memo(
        |recent| -> Html {
            recent
                .iter()
                .map(|fert| {
                    let on_recent_add = {
                        let fert = fert.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |_| {
                            on_change.emit(AddedFertilizerAction::AddRecent(fert.clone()));
                        })
                    };

                    html! {
                        <li><a class="dropdown-item" onclick={on_recent_add}>
                            {&fert.name}{" ("}
                            {FloatFormat::new(fert.N, 2)}{"-"}
                            {FloatFormat::new(fert.P, 2)}{"-"}
                            {FloatFormat::new(fert.K, 2)}{"-"}
                            {FloatFormat::new(fert.Mg, 2)}{")"}
                        </a></li>
                    }
                })
                .collect()
        },
        recent_fertilizers_not_added_yet,
    );

    let add_fertilizer_button = html! {
        <button type="button" class="btn btn-secondary" onclick={on_add_fertilizer}>
            {" + Добавить удобрение"}
        </button>
    };

    html! {
        <div class={stylesheet} id="new-fert">
            if !props.fertilizers.is_empty() {
                <div class="new-fert-list">
                    <h2>{ "Добавленные удобрения" }</h2>
                    // <tr><th>{ "название" }</th><th>{ "азот, %" }</th><th>{ "фосфор, %" }</th><th>{ "калий, %" }</th><th>{ "магний, %" }</th></tr>
                    {for list_of_added}
                </div>
            }
            <div class="py-2">
                if recent_history_is_empty {
                    {add_fertilizer_button}
                } else {
                    <div class="btn-group">
                        {add_fertilizer_button}
                        <button type="button" class="btn btn-warning dropdown-toggle dropdown-toggle-split" data-bs-toggle="dropdown" aria-expanded="false">
                            <span class="visually-hidden">{"Недавно добавленные"}</span>
                        </button>
                        <ul class="dropdown-menu">
                            {recent_fertilizers_rendered.as_ref().clone()}
                        </ul>
                    </div>
                }
                {" которого нет в списке"}
            </div>

        </div>
    }
}
