use std::rc::Rc;

use yew::prelude::*;

use crate::{
    calculator::{consts::PERMANENT_FERTILIZERS, Deficites, ElemName, Fertilizer},
    ui_components::html_chunks::nutrient_css_class,
};

#[derive(PartialEq, Properties)]
pub(crate) struct DeficiteDescriptionProps {
    pub deficites: Rc<Deficites>,
    pub on_calc_another: Callback<()>,
}

fn top_fertilizers_for_elem(fertilizers: &[Fertilizer], element: ElemName) -> AttrValue {
    let mut matches: Vec<_> = fertilizers
        .iter()
        .filter(|fertilizer| {
            let concentration = fertilizer[element];
            concentration > 0.0
        })
        .collect();
    matches.sort_by(|fert1, fert2| f64::total_cmp(&fert1[element], &fert2[element]));
    let mut result = String::new();
    for (i, m) in matches.iter().enumerate() {
        if i != 0 {
            result.push_str(", ")
        }
        result.push_str(&m.name)
    }
    AttrValue::from(result)
}

/// Объясняет, почему смесь не может быть составлена. В норме этот компонент никогда
/// не должен отображаться, так как валидация рецепта происходит ещё до нажатия кнопки
/// "Рассчитать" в статусной строке, и для дефицитного рецепта кнопка будет выключена.
/// Однако, при открытии ссылки на уже созданный рецепт (которая может быть поврежена
/// и содержать ошибочные параметры), предварительная проверка корректности не производится,
/// и надо показать какое-то сообщение о проблеме. Что и делает данный компонент.
#[function_component(DeficiteDescription)]
pub(crate) fn deficite_description(
    DeficiteDescriptionProps { deficites, on_calc_another }: &DeficiteDescriptionProps,
) -> Html {
    let on_calc_another_click = {
        let on_calc_another = on_calc_another.clone();
        move |_| {
            on_calc_another.emit(());
        }
    };

    let deficite_row = |name: &str, element: ElemName| -> Html {
        html! {
            if deficites[element] {
                <tr>
                    <td class={classes!("p-2", nutrient_css_class(element))}>{name}</td>
                    <td>{top_fertilizers_for_elem(PERMANENT_FERTILIZERS, element)}</td>
                </tr>
            }
        }
    };

    html! {
        <>
            <h1>{ "Вариант смеси №2 по методу Митлайдера" }</h1>
            <p class="warning">
                <span class="warning"><strong>
                    { "Из этих удобрений нельзя составить нужную смесь с заданными условиями" }
                </strong></span>
            </p>
            <table class="table">
                <thead>
                    <tr>
                        <th>{"В смеси не хватает"}</th>
                        <th>{"Попробуйте добавить эти удобрения, содержащих нужный элемент"}</th>
                    </tr>
                </thead>
                <tbody>
                {deficite_row("Азота (N)", ElemName::Nitrogen)}
                {deficite_row("Калия (K)", ElemName::Potassium)}
                {deficite_row("Фосфора (P)", ElemName::Phosphorus)}
                {deficite_row("Магния (Mg)", ElemName::Magnesium)}
                </tbody>
            </table>
          <div id="recommended">
            <a class="btn btn-secondary" href="#calculator" onclick={on_calc_another_click}>
            { "Исправить список удобрений" }
            </a>
          </div>
        </>
    }
}
