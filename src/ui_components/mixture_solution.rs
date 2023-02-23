use stylist::yew::styled_component;
use yew::prelude::*;

use crate::calculator::formatted_solution::{DynMicroFertInfo, FormattedSolution};
use crate::ui_components::html_chunks::{PhosphorusOxide, PotassiumOxide, MDASH};
use crate::yew_utils::{include_css, raw_html, FloatFormat};

const REFERENCES_HTML: &str = r##"
<div class="help d-print-none">
    <h2>Источники и примечания</h2>
    <ol>
        <li id="ref1">
            По данным из книги "Семейное овощеводство на узких грядах"
            Т.Ю. Угаровой (ISBN: 5-7856-0092-7, 2000 год), страницы 101-105, 108-110.
        </li>
        <li id="ref2">
            По данным из книги "Рассада" Т.Ю. Угаровой (ISBN: 5-94462-046-3,
            2002 год), страницы 377-380, 540
        </li>
        <li id="ref3">
            По данным из книги "The Mittleider Gardening Course" Джекоба Митлайдера
            (ISBN: 1929982038, 1999 год), страницы 50, 53, 106, 109
        </li>
        <li id="ref4">
            В современных смесях Митлайдера используется только хелат железа
            "Sequestrene 330 Fe" (содержание Fe 10%).
            Данные по железному купоросу могут быть староваты,
            т.к. взяты из более старой книги Митлайдера "Let's Grow Tomatoes"
            (ISBN: 1929982011, 1985 г.), где автор ещё использовал купорос.
            По словам Т.Ю. Угаровой, вместо хелата можно использовать сульфат железа
            (купорос), но в количестве вдвое большем ("Рассада", стр. 539).
        </li>
        <li id="ref5">
            Формулы соединений: "медного купороса" -
            CuSO<sub>4</sub>&sdot;5H<sub>2</sub>O,
            сульфата магния - MgSO<sub>4</sub>&sdot;7H<sub>2</sub>O,
            цинкового купороса - ZnSO<sub>4</sub>&sdot;7H<sub>2</sub>O,
            сульфата марганца - MnSO<sub>4</sub>&sdot;5H<sub>2</sub>O,
            железного купороса - FeSO<sub>4</sub>&sdot;7H<sub>2</sub>O,
            аммония молибденовокислого -
            (NH<sub>4</sub>)<sub>6</sub>Mo<sub>7</sub>O<sub>24</sub>&sdot;4H<sub>2</sub>O
        </li>
    </ol>
</div>
"##;

fn render_main_ingredients(
    solution: &FormattedSolution,
    on_calc_another_callback: Callback<MouseEvent>,
) -> Html {
    let component_rows = solution.components.iter().map(|(fertilizer, weight)| {
        html! {
            <tr class={classes!((*weight == 0.0).then_some("fert-not-used"))}>
                <td>{ &fertilizer.name }</td>
                <td>{ FloatFormat::new(*weight, 3) }</td>
                <td>{ fertilizer.N.to_string() }</td>
                <td>{ fertilizer.P.to_string() }</td>
                <td>{ fertilizer.K.to_string() }</td>
                <td>{ fertilizer.Mg.to_string() }</td>
            </tr>
        }
    });

    let recipe_remarks = solution.remarks.iter().map(|remark| {
        html! {
            <li class={ remark.class }>
            {raw_html(format!("<span>{}</span>", remark.text))}
            </li>
        }
    });

    html! {
        <>
            <p>
                { "Для приготовления " }
                <strong>{ FloatFormat::new(solution.total_weight, 3) }{ " кг. " }</strong>
                { " смеси №2 вам понадобится:" }
            </p>
            <div class="row">
                <div class="col-auto">
                    <table class="table resulting-mixture">
                        <tr>
                        <th>{ "Удобрение" }</th>
                        <th class="weight">{ "вес (кг)" }</th>
                        <th class="nutrient-N">{ "N, %" }</th>
                        <th class="nutrient-P"><span><PhosphorusOxide />{", %"}</span></th>
                        <th class="nutrient-K"><span><PotassiumOxide />{", %"}</span></th>
                        <th class="nutrient-Mg">{ "MgO, %" }</th>
                        </tr>
                        {for component_rows}
                    </table>
                </div>

                <div id="about" class="col">
                    <h2>{ "Характеристика смеси" }</h2>
                    <p>
                        <span>{"Концентрация N"}{MDASH}<PhosphorusOxide />{MDASH}
                            <PotassiumOxide />{MDASH}{"MgO (%):"}</span>
                        <br/>
                        {
                            raw_html(
                                format!(
                                    "<strong>{:.2}&mdash;{:.2}&mdash;{:.2}&mdash;{:.2}</strong>",
                                    solution.concentration.N,
                                    solution.concentration.P,
                                    solution.concentration.K,
                                    solution.concentration.Mg))
                        }
                    </p>
                    <p>{ format!("Соотношение N:P:K = {:.2}:1:{:.2}",
                                 solution.relation[0].1, solution.relation[1].1) }</p>
                    <p>{ format!("Соотношение P:Mg = 1:{:.2}",
                                 solution.relation[2].1) }</p>
                    <p class="d-print-none">
                        <a class="btn btn-secondary" href="#calculator"
                            onclick={on_calc_another_callback}>{ "Изменить состав" }
                        </a>
                    </p>
                </div>
            </div>

            if !solution.remarks.is_empty() {
            <div class="d-print-none">
                <h3>{ "Замечания по составу смеси" }</h3>
                <ul>
                    {for recipe_remarks}
                </ul>
            </div>
            }
        </>
    }
}

fn microfert_recipe_column(
    microfert_recipe: &[DynMicroFertInfo],
    elem_name: &str,
    for_seedlings: bool,
) -> Option<Html> {
    if let Some((_, components)) = microfert_recipe.iter().find(|item| item.0 == elem_name) {
        let component_chunks = components.iter().enumerate().map(|(i, (name, amount))| {
            html! {
                <li>
                if i != 0 { <strong>{" или "}</strong> }
                { FloatFormat::new(*amount, 1) }{" г. "}{ name }
                </li>
            }
        });
        Some(html! {
            <dd class={classes!(for_seedlings.then_some("seedling"))}>
                <ul>{for component_chunks}</ul>
            </dd>
        })
    } else {
        None
    }
}

fn render_micro_ingredients(solution: &FormattedSolution) -> Html {
    const MICROFERT_OUTPUT_ORDER: &[(&str, &str)] = &[
        ("Молибден", "Mo"),
        ("Бор", "B"),
        ("Железо", "Fe"),
        ("Марганец", "Mn"),
        ("Цинк", "Zn"),
        ("Медь", "Cu"),
    ];

    let generate_microfert_table = |microfert_solution| -> Html {
        let table_rows = MICROFERT_OUTPUT_ORDER.iter().map(|(row_title, elem_name)| {
            let amount_description = microfert_recipe_column(microfert_solution, elem_name, false);
            let element_not_needed = amount_description.is_none();
            let dd = amount_description
                .unwrap_or_else(|| html! { <dd class="fert-not-used">{ "не требуется" }</dd>});
            html! {
                <>
                    <dt class={classes!(element_not_needed.then_some("fert-not-used"))}>
                        { row_title }
                    </dt>
                    { dd }
                </>
            }
        });

        html! {
            <dl class="microferts">
                {for table_rows}
            </dl>
        }
    };

    html! {
        <div class="row microferts">
            <div class="col">
                <h3>
                    {"Для культур на грядах"}<br />
                    <small class="text-muted">
                        {"(смесь №2, по Угаровой"}
                        <sup><a href="#ref1">{"1"}</a></sup>
                        {", "}
                        <mark>{"идеал для начинающих"}</mark>
                        {")"}
                    </small>
                </h3>
                {generate_microfert_table(&solution.microferts)}
            </div>

            <div class="col">
                <h3 class="seedling">
                    {"Для рассады"}<br/>
                    <small class="text-muted">
                        {"(смесь №2А, по Угаровой, для рассады с подсветкой)"}
                        <sup><a href="#ref2">{"2"}</a></sup>
                    </small>
                </h3>
                {generate_microfert_table(&solution.microferts_2a)}
            </div>

            <div class="col">
                <h3 class="seedling">
                    {"По Митлайдеру "}
                    <small class="text-muted">
                        {"(для рассады и бедных по составу грунтов)"}
                        <a href="#ref3">{"3"}</a>
                    </small>
                </h3>
                {generate_microfert_table(&solution.microferts_mit)}
            </div>
        </div>
    }
}

fn render_application_tips(solution: &FormattedSolution) -> Html {
    let seedling_dozes_rows = solution
        .seedling_dozes
        .iter()
        .map(|(volume, dose, dose_mit)| {
            html! {
                <tr>
                    <td>{ volume.to_string() }{ " л." }</td>
                    <td>{ FloatFormat::new(*dose, 2) }{ " г." }</td>
                    <td>{ FloatFormat::new(*dose_mit, 2) }{ " г." }</td>
                </tr>
            }
        });

    let ground_dozes_rows =
        solution
            .ground_dozes
            .iter()
            .enumerate()
            .map(|(i, (ground_type, doze))| {
                html! {
                    <tr>
                        <td>{ ground_type }</td>
                        <td>
                            { FloatFormat::new(doze.from, 0) }{"-"}{ FloatFormat::new(doze.to, 0) }
                            <span>{"*"}</span>
                        </td>
                        if i == 0 {
                            <td rowspan={solution.ground_dozes.len().to_string()}>
                            { FloatFormat::new(solution.mit_ground_doze, 0) }
                            </td>
                        }
                    </tr>
                }
            });

    html! {
        <>
            <h3>{ "Для подкормок на грядах" }</h3>
            <table>
            <thead>
                <tr>
                    <th rowspan="2">
                        {"Тип почвы"}</th><th colspan="2">{"Доза внесения "}<br/>
                        {"(\"грамм/погонный метр)\""}
                    </th>
                </tr>
                <tr><th>{"По Угаровой"}</th><th>{"По Миттлайдеру"}</th></tr>
            </thead>
            <tbody>
                {for ground_dozes_rows}
            </tbody>
            </table>

            {raw_html(r##"<p class="help"><span>*</span> Максимальную дозу
                следует вносить в тёплое и солнечное лето в период быстрого
                роста растений и плодообразования!<br/>Минимальную же,
                наоборот - в холодное и пасмурное лето,
                при этом можно увеличить общее число подкормок на одну-две.</p>"##)}

            <h3>
                {"Для полива рассады"}<br />
                <small class="text-muted">
                {"(используйте только смеси со всеми микроэлементами, \
                    как \"2а\" по Угаровой или по Миттлайдеру)" }
                </small>
            </h3>
            <table>
            <thead>
                <tr>
                    <th rowspan="2">{ "Объём воды" }</th>
                    <th colspan="2">{ "Смесь с микроэлементами" }</th>
                </tr>
                <tr><th>{ "по Угаровой" }</th><th>{ "по Митлайдеру" }</th></tr>
            </thead>
            <tbody>
            {for seedling_dozes_rows}
            </tbody>
            </table>
        </>
    }
}

#[derive(Debug, PartialEq, Properties)]
pub(crate) struct MixtureSolutionProps {
    pub on_calc_another: Callback<()>,
    pub solution: FormattedSolution,
}

/// Отвечает за вывод всей информации о вычисленной смеси, включая и её состав и технику применения.
#[styled_component]
pub(crate) fn MixtureSolution(
    MixtureSolutionProps { solution, on_calc_another }: &MixtureSolutionProps,
) -> Html {
    let stylesheet = include_css!("mixture_solution.css");

    // Прокрутка к началу экрана при первом отображении результатов расчёта
    use_effect_with_deps(
        |_deps| {
            if let Some(window) = web_sys::window() {
                window.scroll_with_x_and_y(0.0, 0.0);
            }
        },
        (),
    );

    let on_calc_another_click = {
        let on_calc_another = on_calc_another.clone();
        Callback::from(move |_| {
            on_calc_another.emit(());
        })
    };

    html! {
        <div class={stylesheet}>
            <h1>{ "Вариант смеси №2 по методу Митлайдера" }</h1>

            <h2>{"Шаг 1. Смешайте основные удобрения"}</h2>
            {render_main_ingredients(solution, on_calc_another_click.clone())}

            <div class="recommended">
                <h2>
                    { "Шаг 2. Добавьте микроудобрения "}
                    <small class="text-muted">{"(один из вариантов)" }</small>
                </h2>
                {render_micro_ingredients(solution)}

                <h2>{"Шаг 3. Используйте смесь"}</h2>
                {render_application_tips(solution)}

                <p class="d-print-none">
                    <a class="btn btn-secondary" href="#calculator"
                        onclick={&on_calc_another_click}>{ "Рассчитать другой вариант смеси" }
                    </a>
                </p>

                {raw_html(REFERENCES_HTML)}

            </div>
        </div>
    }
}
