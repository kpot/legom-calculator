use std::rc::Rc;

use crate::calculator::consts::PERMANENT_FERTILIZERS;
use crate::calculator::query::MixtureQuery;
use crate::calculator::{ElemRange, ElemRangeName, ElemRatios, Fertilizer};
use yew::Reducible;

/// Хранит идентификаторы "перманентных" удобрений, выбранных пользователем, а также
/// ограничения, наложенные на любые перманентные удобрения (не важно, выбранные или нет).
/// Ограничения для не выбранных удобрений позднее будут проигнорированы в подсчётах.
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct PermanentFertilizersState {
    pub selected: Rc<Vec<usize>>,
    pub limited: Rc<Vec<(usize, f64)>>,
}

impl PermanentFertilizersState {
    pub fn new(selected: Vec<usize>, limited: Vec<(usize, f64)>) -> Self {
        Self { selected: Rc::new(selected), limited: Rc::new(limited) }
    }

    pub fn toggle_selection(&mut self, fert_id: usize, is_selected: bool) {
        let new_selected = Rc::make_mut(&mut self.selected);
        match (is_selected, new_selected.iter().position(|v| *v == fert_id)) {
            (false, Some(index)) => {
                new_selected.remove(index);
            }
            (true, None) => {
                new_selected.push(fert_id);
            }
            _ => {}
        }
    }

    pub fn set_limit(&mut self, fert_id: usize, limit: Option<f64>) {
        let new_permanent_limits = Rc::make_mut(&mut self.limited);
        match (
            limit,
            new_permanent_limits.iter().position(|l| l.0 == fert_id),
        ) {
            (None, None) => {}
            (None, Some(old_limit_index)) => {
                new_permanent_limits.remove(old_limit_index);
            }
            (Some(new_limit), None) => {
                new_permanent_limits.push((fert_id, new_limit));
            }
            (Some(new_limit), Some(old_limit_index)) => {
                new_permanent_limits[old_limit_index] = (fert_id, new_limit);
            }
        }
    }

    pub fn is_selected(&self, fert_id: usize) -> bool {
        self.selected.iter().any(|f| *f == fert_id)
    }

    pub fn get_limit(&self, fert_id: usize) -> Option<f64> {
        self.limited
            .iter()
            .find(|record| record.0 == fert_id)
            .map(|record| record.1)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub(crate) struct AppStore {
    /// Выбранные "перманентные" удобрения и их ограничения
    pub permanent_fertilizers: Rc<PermanentFertilizersState>,
    /// Список добавленных вручную удобрений
    pub added_fertilizers: Rc<Vec<Fertilizer>>,
    /// Диапазон соотношения к фосфору для азота, калия и магния
    pub ratios: Rc<ElemRatios>,
    /// Итоговая масса смеси
    pub mass: f64,
}

pub(crate) enum AddedFertilizerAction {
    Add,
    AddRecent(Fertilizer),
    Remove(usize),
    Update(Fertilizer),
}

pub(crate) enum StoreAction {
    ToggleFertilizer(usize, bool),
    UpdatePermanentLimit(usize, Option<f64>),
    ChangeAdded(AddedFertilizerAction),
    UpdateRatio(ElemRangeName, ElemRange),
    UpdateMass(f64),
}

impl Reducible for AppStore {
    type Action = StoreAction;

    fn reduce(mut self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut new_self = Rc::make_mut(&mut self);
        match action {
            StoreAction::ToggleFertilizer(fert_id, is_selected) => {
                let new_permanent = Rc::make_mut(&mut new_self.permanent_fertilizers);
                new_permanent.toggle_selection(fert_id, is_selected);
            }
            StoreAction::ChangeAdded(sub_action) => {
                let new_added = Rc::make_mut(&mut new_self.added_fertilizers);
                match sub_action {
                    AddedFertilizerAction::Add => new_added.push(Fertilizer::default()),
                    AddedFertilizerAction::AddRecent(fertilizer) => new_added.push(fertilizer),
                    AddedFertilizerAction::Remove(fert_id) => {
                        if let Some(pos) = new_added.iter().position(|f| f.id == fert_id) {
                            new_added.remove(pos);
                        }
                    }
                    AddedFertilizerAction::Update(fertilizer) => {
                        if let Some(fert) = new_added.iter_mut().find(|f| f.id == fertilizer.id) {
                            *fert = fertilizer;
                        }
                    }
                }
            }
            StoreAction::UpdateRatio(nutrient, range) => {
                let new_ranges = Rc::make_mut(&mut new_self.ratios);
                new_ranges[nutrient] = range;
            }
            StoreAction::UpdateMass(mass) => {
                new_self.mass = mass;
            }
            StoreAction::UpdatePermanentLimit(fert_id, new_limit) => {
                let new_permanent = Rc::make_mut(&mut new_self.permanent_fertilizers);
                new_permanent.set_limit(fert_id, new_limit);
            }
        }
        self
    }
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            mass: 10.0,
            permanent_fertilizers: Default::default(),
            added_fertilizers: Default::default(),
            ratios: Default::default(),
        }
    }
}

impl AppStore {
    pub fn is_valid(&self) -> bool {
        let some_fertilizers_to_process =
            !(self.permanent_fertilizers.selected.is_empty() && self.added_fertilizers.is_empty());
        some_fertilizers_to_process && self.ratios.all_valid() && self.mass > 0.0
    }

    pub fn to_mixture_query(&self) -> MixtureQuery {
        let mut all_fertilizers: Vec<_> = self
            .permanent_fertilizers
            .selected
            .iter()
            .filter_map(|id| {
                PERMANENT_FERTILIZERS
                    .iter()
                    .find(|f| f.id == *id)
                    .map(|permanent_fertilizer| {
                        let mut updated_copy = permanent_fertilizer.clone();
                        updated_copy.limit = self
                            .permanent_fertilizers
                            .limited
                            .iter()
                            .find(|(id, _)| *id == permanent_fertilizer.id)
                            .map(|(_, limit)| *limit);
                        updated_copy
                    })
            })
            .collect();
        all_fertilizers.extend_from_slice(self.added_fertilizers.as_slice());
        MixtureQuery {
            fertilizers: all_fertilizers,
            N_ratio: self.ratios.n_to_p,
            K_ratio: self.ratios.k_to_p,
            Mg_ratio: self.ratios.mg_to_p,
            mass: self.mass,
        }
    }

    pub fn from_mixture_query(query: &MixtureQuery) -> Self {
        let mut selected_fertilizers = Vec::new();
        let mut permanent_ferts_limits = Vec::new();
        let mut added_fertilizers = Vec::<Fertilizer>::new();
        for f in query.fertilizers.iter() {
            let fert_content_id = f.content_id();
            match PERMANENT_FERTILIZERS
                .iter()
                .find(|pf| fert_content_id == pf.content_id())
            {
                Some(pf) => {
                    selected_fertilizers.push(pf.id);
                    if let Some(query_fert_limit) = f.limit {
                        permanent_ferts_limits.push((f.id, query_fert_limit));
                    }
                }
                None => added_fertilizers.push(f.clone()),
            }
        }
        Self {
            permanent_fertilizers: Rc::new(PermanentFertilizersState::new(
                selected_fertilizers,
                permanent_ferts_limits,
            )),
            added_fertilizers: Rc::new(added_fertilizers),
            ratios: Rc::new(ElemRatios {
                n_to_p: query.N_ratio,
                k_to_p: query.K_ratio,
                mg_to_p: query.Mg_ratio,
            }),
            mass: query.mass,
        }
    }
}
