use std::rc::Rc;

use yew::prelude::{function_component, html, use_state, Callback, Html};
use yew::{use_callback, use_reducer};
use yew_router::prelude::use_navigator;
use yew_router::{components::Link, hooks::use_location, BrowserRouter, Routable, Switch};

use crate::calculator::{ElemRange, ElemRangeName};
use crate::store::{AddedFertilizerAction, AppStore, StoreAction};
use crate::ui_components::added_fertilizers::{
    store_history_of_added_fertilizers, AddedFertilizers,
};
use crate::ui_components::deficite_description::DeficiteDescription;
use crate::ui_components::elem_ranges::NutrientRatios;
use crate::ui_components::intro::Intro;
use crate::ui_components::known_fertilizers::KnownFertilizers;
use crate::ui_components::mixture_solution::MixtureSolution;
use crate::ui_components::status_bar::StatusBar;
use crate::ui_components::total_mass::TotalMassInput;

use crate::calculator::query::MixtureQuery;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[at("/ru/calculator/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(StartPage)]
pub fn start_page() -> Html {
    let location = use_location();
    let navigator = use_navigator();
    let query = location
        .and_then(|location| location.query::<Vec<(String, String)>>().ok())
        .and_then(|query_map| MixtureQuery::from_query_map(&query_map));
    let state = use_reducer(|| match query {
        Some(ref query) => AppStore::from_mixture_query(query),
        None => AppStore::default(),
    });

    let is_valid = (*state).is_valid();

    let query = state.to_mixture_query();
    let solution = query.find_solution();
    let deficites = Rc::new(query.find_deficites());

    let show_solution = use_state(|| solution.is_ok());

    let on_calculate = {
        let show_solution = show_solution.clone();
        let added_fertilizers = state.added_fertilizers.clone();
        Callback::from(move |_| {
            store_history_of_added_fertilizers(&added_fertilizers);
            if let Some(ref navigator) = navigator {
                navigator
                    .replace_with_query(&Route::Home, &query.to_url_query())
                    .ok();
                show_solution.set(true);
            }
        })
    };

    let on_calc_another = {
        let show_solution = show_solution.clone();
        Callback::from(move |_| {
            show_solution.set(false);
        })
    };

    let on_added_changed = use_callback(
        |action: AddedFertilizerAction, dispatcher| {
            dispatcher.dispatch(StoreAction::ChangeAdded(action));
        },
        state.dispatcher(),
    );

    let on_ratio_change = use_callback(
        move |(range_name, range_value): (ElemRangeName, ElemRange), dispatcher| {
            dispatcher.dispatch(StoreAction::UpdateRatio(range_name, range_value))
        },
        state.dispatcher(),
    );

    let on_mass_changed = use_callback(
        |value, dispatcher| dispatcher.dispatch(StoreAction::UpdateMass(value)),
        state.dispatcher(),
    );

    html! {
        <>
            if !*show_solution {
                <Intro />
                <KnownFertilizers
                    store_dispatcher={state.dispatcher()}
                    fertilizers_status={state.permanent_fertilizers.clone()}
                    deficites={deficites.clone()} />
                <AddedFertilizers
                    fertilizers={state.added_fertilizers.clone()}
                    on_change={on_added_changed} />
                    <NutrientRatios {on_ratio_change} ratios={state.ratios.clone()} />
                <TotalMassInput value={state.mass} on_change={on_mass_changed} />
                <StatusBar
                    {deficites}
                    on_show_solution={on_calculate}
                    state_is_valid={is_valid} />
            } else if let Ok(solution) = solution {
                <MixtureSolution {solution} on_calc_another={&on_calc_another} />
            } else {
                <DeficiteDescription {deficites} on_calc_another={&on_calc_another} />
            }
        </>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <StartPage /> },
        Route::NotFound => html! {
            <>
                <h1>{"Здесь ничего нет"}</h1>
                <p>{"Вероятно, вы перешли по старой ссылке. Попробуйте "}
                <Link<Route> to={Route::Home}>{"Рассчитать смесь заново"}</Link<Route>></p>
            </>
        },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
