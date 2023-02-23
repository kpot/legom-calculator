use crate::calculator::formatted_solution::MicroFertInfo;
use std::borrow::Cow;

use super::{ElemRange, Fertilizer};

/// Содержание азота (кг) в стандартной смеси Т.Ю. Угаровой (необходимо для рассчёта микроудобрений)
pub const STD_N_QUANTITY: f64 = 1.1;

/// Содержание микроудобрений (в граммах) на одну дозу стандартной смеси (STD_N_QUANTITY)
/// для простой смеси Угаровой, применяемой на узких грядах
pub(crate) const MICROFERTS: [MicroFertInfo; 2] = [
    (
        "Mo",
        &[
            ("аммония молибденовокислого", 15.),
            ("молибденовой кислоты", 15.),
        ],
    ),
    ("B", &[("борной кислоты", 15.), ("буры", 20.)]),
];

/// Содержание микроэлементов (в грамма) на дозу стандартной смеси (STN_N_QUANTITY)
/// для смеси 2а Угаровой
pub(crate) const MICROFERTS_2A: [MicroFertInfo; 6] = [
    (
        "Mo",
        &[
            ("аммония молибденовокислого", 10.),
            ("молибденовой кислоты", 10.),
        ],
    ),
    ("B", &[("борной кислоты", 20.), ("буры", 30.9)]),
    (
        "Fe",
        &[("железного купороса", 240.), ("хелата железа", 120.)],
    ),
    ("Mn", &[("сульфата марганца", 24.0)]),
    ("Zn", &[("цинкового купороса", 16.)]),
    ("Cu", &[("медного купороса", 8.)]),
];

/// Содержание микроэлементов (в грамма) на дозу стандартной смеси (STN_N_QUANTITY)
/// для смеси Weekly Feed по Миттлайдеру
pub(crate) const MICROFERTS_MIT: [MicroFertInfo; 6] = [
    (
        "Mo",
        &[
            ("аммония молибденовокислого", 5.18476621033266),
            ("молибденовой кислоты", 5.18476621033266),
        ],
    ),
    (
        "B",
        &[("борной кислоты", 40.258184722), ("буры", 62.21719457)],
    ),
    (
        "Fe",
        &[
            ("хелата железа #330", 10.369532453),
            ("железного купороса", 140.),
        ],
    ),
    ("Mn", &[("сульфата марганца", 41.478129738)]),
    ("Zn", &[("цинкового купороса", 82.956259768)]),
    ("Cu", &[("медного купороса", 10.369532453)]),
];

/// Норма содержания азота в литре удобрительного раствора смеси 2а по Угаровой
pub(crate) const N_PER_LITER_2A: f64 = 0.3667;

/// Норма содержания азота в литре удобрительного раствора смеси для рассады по Миттлайдеру
pub(crate) const N_PER_LITER_MIT: f64 = 0.3325;

/// Стандартная концентрация N|K. Нужно для определения если смесь слишком
/// бедная и будет вносить много балласта в почву.
pub(crate) const STD_N_K_CONCENTRATION: f64 = 14.5;

/// Концентрация азота или калия в смеси, которая считается недопустимо низкой,
/// и приводит к выдаче предупреждения о большой доле балластных элементов
pub(crate) const CRITICAL_LOW_N_K_CONCENTRATION: f64 = 12.;

/// Дозы внесения на разных видах почвы (min,max) смеси стандартной концентрации N|K
pub(crate) const DOZES: &[(&str, ElemRange)] = &[
    ("Супеси и песчаные почвы", ElemRange { from: 20., to: 25. }),
    ("Остальные почвы", ElemRange { from: 25., to: 40. }),
];

/// Предельная доза внесения
pub(crate) const MAX_DOZE: f64 = 50.;

/// Доза азота на 1 метр гряды
pub(crate) const MIT_N_PER_METER: f64 = 6.60659;
pub(crate) const MIT_MAX_DOZE: f64 = 50.;

const fn max_fertilizer_id(fertilizers: &[Fertilizer], start_from: usize) -> Option<usize> {
    if start_from < fertilizers.len() {
        let current_id = fertilizers[start_from].id;
        match max_fertilizer_id(fertilizers, start_from + 1) {
            Some(max_id_left) => {
                if current_id < max_id_left {
                    Some(max_id_left)
                } else {
                    Some(current_id)
                }
            }
            None => Some(current_id),
        }
    } else {
        None
    }
}

/// Перманентные удобрения, доступные пользователю сразу, без необходимости их добавлять,
/// но и без возможности их удалять.
pub(crate) const PERMANENT_FERTILIZERS: &[Fertilizer] = &[
    // ВАЖНО: При внесении любых изменений, убедись, что все ID - уникальны
    Fertilizer {
        name: Cow::Borrowed("Аммиачная селитра"),
        N: 34.0,
        P: 0.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 0,
    },
    Fertilizer {
        name: Cow::Borrowed("Карбамид (мочевина)"),
        N: 46.2,
        P: 0.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 1,
    },
    Fertilizer {
        name: Cow::Borrowed("Азофоска"),
        N: 16.0,
        P: 16.0,
        K: 16.0,
        Mg: 0.0,
        with_Cl: true,
        with_S: false,
        limit: None,
        id: 2,
    },
    Fertilizer {
        name: Cow::Borrowed("Диаммоний фосфат"),
        N: 19.0,
        P: 49.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 3,
    },
    Fertilizer {
        name: Cow::Borrowed("Аммофос"),
        N: 12.0,
        P: 52.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 4,
    },
    Fertilizer {
        name: Cow::Borrowed("Суперфорсфат простой"),
        N: 0.0,
        P: 19.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 5,
    },
    Fertilizer {
        name: Cow::Borrowed("Суперфосфат гранулированный"),
        N: 0.0,
        P: 26.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 6,
    },
    Fertilizer {
        name: Cow::Borrowed("Суперфосфат гранулированный (с азотом)"),
        N: 6.0,
        P: 26.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 7,
    },
    Fertilizer {
        name: Cow::Borrowed("Суперфосфат двойной"),
        N: 0.0,
        P: 43.0,
        K: 0.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 8,
    },
    Fertilizer {
        name: Cow::Borrowed("Монофосфат калия"),
        N: 0.0,
        P: 52.0,
        K: 34.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 9,
    },
    Fertilizer {
        name: Cow::Borrowed("Сульфат калия"),
        N: 0.0,
        P: 0.0,
        K: 50.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 10,
    },
    Fertilizer {
        name: Cow::Borrowed("Хлорид калия"),
        N: 0.0,
        P: 0.0,
        K: 60.0,
        Mg: 0.0,
        with_Cl: true,
        with_S: false,
        limit: None,
        id: 11,
    },
    Fertilizer {
        name: Cow::Borrowed("Калийная селитра"),
        N: 13.0,
        P: 0.0,
        K: 46.0,
        Mg: 0.0,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 12,
    },
    Fertilizer {
        name: Cow::Borrowed("Калимаг"),
        N: 0.0,
        P: 0.0,
        K: 35.0,
        Mg: 8.0,
        with_Cl: true,
        with_S: false,
        limit: None,
        id: 13,
    },
    Fertilizer {
        name: Cow::Borrowed("Сульфат магния (магний сернокислый)"),
        N: 0.0,
        P: 0.0,
        K: 0.0,
        Mg: 16.0,
        with_Cl: false,
        with_S: true,
        limit: None,
        id: 14,
    },
    Fertilizer {
        name: Cow::Borrowed("Магниевая селитра"),
        N: 11.1,
        P: 0.0,
        K: 0.0,
        Mg: 15.5,
        with_Cl: false,
        with_S: false,
        limit: None,
        id: 15,
    },
];

/// Любые удобрения, добавленные к перманентным, будут иметь инкрементные идентификаторы,
/// начинающиеся с данного
pub(crate) const FIRST_EXTRA_FERT_ID: usize = match max_fertilizer_id(PERMANENT_FERTILIZERS, 0) {
    Some(id) => id + 1,
    None => 0,
};
