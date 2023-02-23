pub(crate) mod consts;
pub(crate) mod formatted_solution;
pub(crate) mod modified_simplex;
pub(crate) mod query;

use std::borrow::Cow;
use std::fmt::Write;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct ElemRange {
    pub from: f64,
    pub to: f64,
}

/// Используется для индексации в структурах удобрений, дефицитов, и т.п.
#[derive(Clone, Copy)]
pub(crate) enum ElemName {
    Nitrogen,
    Phosphorus,
    Potassium,
    Magnesium,
}

/// Используется для индексации в диапазонах макроудобрений
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ElemRangeName {
    Nitrogen,
    Potassium,
    Magnesium,
}

/// Соотношения макроэлементов к фосфору
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ElemRatios {
    pub n_to_p: ElemRange,
    pub k_to_p: ElemRange,
    pub mg_to_p: ElemRange,
}

impl Default for ElemRatios {
    fn default() -> Self {
        Self {
            n_to_p: ElemRange { from: 1.75, to: 1.85 },
            k_to_p: ElemRange { from: 1.75, to: 1.85 },
            mg_to_p: ElemRange { from: 0.25, to: 0.45 },
        }
    }
}

impl ElemRatios {
    pub fn all_valid(&self) -> bool {
        self.n_to_p.is_valid() && self.k_to_p.is_valid() && self.mg_to_p.is_valid()
    }
}

impl std::ops::Index<ElemRangeName> for ElemRatios {
    type Output = ElemRange;

    fn index(&self, index: ElemRangeName) -> &Self::Output {
        match index {
            ElemRangeName::Nitrogen => &self.n_to_p,
            ElemRangeName::Potassium => &self.k_to_p,
            ElemRangeName::Magnesium => &self.mg_to_p,
        }
    }
}

impl std::ops::IndexMut<ElemRangeName> for ElemRatios {
    fn index_mut(&mut self, index: ElemRangeName) -> &mut Self::Output {
        match index {
            ElemRangeName::Nitrogen => &mut self.n_to_p,
            ElemRangeName::Potassium => &mut self.k_to_p,
            ElemRangeName::Magnesium => &mut self.mg_to_p,
        }
    }
}

/// Всё, что мы знаем про удобрение. Используется как для предопределённых (permanent) удобрений,
/// так и для удобрений, добавленных пользователем (в последнем случае, содержание хлора или серы
/// не указывается, так как обычно этой информации всё равно нет на упаковке)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct Fertilizer {
    pub name: Cow<'static, str>,
    pub N: f64,
    pub P: f64,
    pub K: f64,
    pub Mg: f64,
    pub with_Cl: bool,
    pub with_S: bool,
    /// Предельно доступное количество удобрения
    pub limit: Option<f64>,
    /// Идентификатор у удобрения, упрощающий последующую работу с ним.
    /// Он динамический, регенерируется заново при каждом перезапуске
    /// (если не считать перманентных удобрений), а значит сериализовать его бесполезно.
    /// При десериализации надо просто сгенерировать новый.
    pub id: usize,
}

/// Глобальный счётчик идентификаторов удобрений, ведущий отчёт с наибольшего
/// ID среди перманентых удобрений + 1.
static NEXT_FERT_ID: AtomicUsize = AtomicUsize::new(consts::FIRST_EXTRA_FERT_ID);

impl Fertilizer {
    pub fn new_id() -> usize {
        NEXT_FERT_ID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn re_id(&mut self) {
        self.id = Self::new_id();
    }

    pub fn content_id(&self) -> String {
        format!("{}_{}_{}_{}_{}", self.name, self.N, self.P, self.K, self.Mg)
    }

    /// Кодирует информацию об удобрении в нечто вроде
    /// `fert=Азофоска:N:16.0,P:16.0,K:16.0,Mg:0.0,Cl:f,S:t`
    /// для последующей вставки в ссылки.
    pub fn urlencode(&self) -> String {
        let cl_flag = if self.with_Cl { "t" } else { "f" };
        let s_flag = if self.with_S { "t" } else { "f" };
        let mut result = format!(
            "{}:N:{},P:{},K:{},Mg:{},Cl:{},S:{}",
            self.name, self.N, self.P, self.K, self.Mg, cl_flag, s_flag
        );
        if let Some(lim) = self.limit {
            write!(&mut result, ",lim:{}", lim).expect("Limit must be serializable");
        }
        result
    }
}

impl std::ops::Index<ElemName> for Fertilizer {
    type Output = f64;

    fn index(&self, index: ElemName) -> &Self::Output {
        match index {
            ElemName::Nitrogen => &self.N,
            ElemName::Phosphorus => &self.P,
            ElemName::Potassium => &self.K,
            ElemName::Magnesium => &self.Mg,
        }
    }
}

impl std::ops::IndexMut<ElemName> for Fertilizer {
    fn index_mut(&mut self, index: ElemName) -> &mut Self::Output {
        match index {
            ElemName::Nitrogen => &mut self.N,
            ElemName::Phosphorus => &mut self.P,
            ElemName::Potassium => &mut self.K,
            ElemName::Magnesium => &mut self.Mg,
        }
    }
}

impl Default for Fertilizer {
    fn default() -> Self {
        Self {
            name: Default::default(),
            N: Default::default(),
            P: Default::default(),
            K: Default::default(),
            Mg: Default::default(),
            with_Cl: Default::default(),
            with_S: Default::default(),
            limit: Default::default(),
            id: Self::new_id(),
        }
    }
}

impl ElemRange {
    pub fn try_new(from: f64, to: f64) -> Option<Self> {
        let result = Self { from, to };
        if result.is_valid() {
            Some(result)
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.from > 0.0 && self.to > 0.0 && self.from <= self.to
    }
}

/// Имеет двойное назначение:
/// 1. хранит концентрации элементов (оксидов) в удобренииях,
/// 2. также хранит абсолютные содержания этих элементов в килограммах, на последних стадиях
///    вывода результатов.
#[derive(Default, Debug, PartialEq)]
#[allow(non_snake_case)]
pub(crate) struct Amounts {
    pub N: f64,
    pub P: f64,
    pub K: f64,
    pub Mg: f64,
}

impl std::ops::Index<ElemName> for Amounts {
    type Output = f64;

    fn index(&self, index: ElemName) -> &Self::Output {
        match index {
            ElemName::Nitrogen => &self.N,
            ElemName::Phosphorus => &self.P,
            ElemName::Potassium => &self.K,
            ElemName::Magnesium => &self.Mg,
        }
    }
}

/// При подсчёте результатов, хранит информацию о дефицитности элементов
/// (true - значит, элемента в смеси недостаточно).
#[derive(Debug, PartialEq)]
#[allow(non_snake_case)]
pub(crate) struct Deficites {
    pub N: bool,
    pub P: bool,
    pub K: bool,
    pub Mg: bool,
}

impl Deficites {
    pub fn any(&self) -> bool {
        self.N || self.P || self.K || self.Mg
    }
}

impl std::ops::Index<ElemName> for Deficites {
    type Output = bool;

    fn index(&self, index: ElemName) -> &Self::Output {
        match index {
            ElemName::Nitrogen => &self.N,
            ElemName::Phosphorus => &self.P,
            ElemName::Potassium => &self.K,
            ElemName::Magnesium => &self.Mg,
        }
    }
}
