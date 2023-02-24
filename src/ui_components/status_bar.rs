use std::rc::Rc;

use stylist::yew::styled_component;
use yew::{classes, html, Callback, Html, Properties};

use crate::{
    calculator::{Deficites, ElemName},
    ui_components::html_chunks::CHECK_MARK,
    yew_utils::include_css,
};

use super::html_chunks::nutrient_css_class;

#[derive(PartialEq, Properties)]
pub(crate) struct StatusBarProps {
    pub state_is_valid: bool,
    pub deficites: Rc<Deficites>,
    pub on_show_solution: Callback<()>,
}

/// Нижняя панель, которая отображает соблюдение баланса элементов
/// в ходе выбора удобрений для смеси, и позволяет перейти к просмотру результата.
#[styled_component]
pub(crate) fn StatusBar(props: &StatusBarProps) -> Html {
    const LACK: &str = "—";

    let stylesheet = include_css!("status_bar.css");

    let on_btn_click = {
        let on_show_solution = props.on_show_solution.clone();
        Callback::from(move |_| {
            on_show_solution.emit(());
        })
    };

    let nutrient_deficite_column = |element: ElemName, name: &str| -> Html {
        let deficite = props.deficites[element];
        let nutrient_class = nutrient_css_class(element);
        html! {
            <div class={classes!("col", "gx-2", "gx-sm-4", "py-2",
                                nutrient_class, deficite.then_some("deficite"))}>
                <span>if deficite {{LACK}} else {{CHECK_MARK} }</span>
                {" "}
                {name}
            </div>
        }
    };

    html! {
        <nav class={classes!(stylesheet, "navbar", "fixed-bottom", "bg-light")}>
            <div class="container-fluid">
                <div class="row mx-auto status-bar">
                    <div class="col d-none d-md-block">{"Баланс"}</div>
                    {nutrient_deficite_column(ElemName::Nitrogen, "азот")}
                    {nutrient_deficite_column(ElemName::Phosphorus, "фосфор")}
                    {nutrient_deficite_column(ElemName::Potassium, "калий")}
                    {nutrient_deficite_column(ElemName::Magnesium, "магний")}
                    <div class="col gx-sm-4 gx-1">
                        <button type="submit" class="btn btn-primary"
                            disabled={props.deficites.any() || !props.state_is_valid}
                            onclick={on_btn_click}>
                            {"Рассчитать!"}
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    }
}
