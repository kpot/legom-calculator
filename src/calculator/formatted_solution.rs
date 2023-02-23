use crate::calculator::consts::STD_N_QUANTITY;
use crate::calculator::modified_simplex::LPSolution;
use crate::calculator::query::MixtureQuery;
use crate::calculator::{consts, Amounts, ElemRange, ElemRangeName, Fertilizer};

/// Вычисляет дозы микроудобрений на основании реальных доз макро-элементов
/// в вычисленной смеси и предварительно подсчитанных по книгам Угаровой
/// пропорций веществ в микроэлементами по отношению к данным макро-элементам.
/// Допустим, если на килограмм азота нам надо 0.15г борной кислоты, то для 10кг
/// кг азота в смеси, данная функция насчитает 150г борной.
#[allow(non_snake_case)]
fn calc_microferts(
    basis: &[MicroFertInfo],
    nutrient_quantities: &Amounts,
) -> Vec<DynMicroFertInfo> {
    let mut microferts: Vec<DynMicroFertInfo> = basis
        .iter()
        .map(|(name, slice)| (*name, Vec::with_capacity(slice.len())))
        .collect();
    let micro_ratio = nutrient_quantities.N / STD_N_QUANTITY;
    for ((_elem, ferts), (_, output_quant)) in basis.iter().zip(microferts.iter_mut()) {
        for (micro_fert, micro_quant) in ferts.iter() {
            output_quant.push((micro_fert, *micro_quant * micro_ratio));
        }
    }
    microferts
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SolutionRemarks {
    pub class: &'static str,
    pub text: String,
}

fn check_mixture_bonus_element<F>(
    components: &[(Fertilizer, f64)],
    perm_fertilizers: &[Fertilizer],
    message_prefix: &str,
    bonus_checker: F,
) -> Option<SolutionRemarks>
where
    F: Fn(&Fertilizer) -> bool,
{
    let weight: f64 = components.iter().map(|(_, weight)| *weight).sum();
    let has_enough_element = components.iter().any(|(fertilizer, comp_weight)| {
        let portion = *comp_weight / weight;
        // Отбрасываем компоненты, содержание которых в смеси - менее 10%
        let is_significant = portion > 0.1;
        is_significant && bonus_checker(fertilizer)
    });
    if has_enough_element {
        None
    } else {
        let mut text = String::from(message_prefix);
        let mut added_ferts = 0;
        for pf in perm_fertilizers {
            if bonus_checker(pf) {
                if added_ferts != 0 {
                    text.push_str(", ");
                }
                text.push_str(pf.name.as_ref());
                added_ferts += 1;
            }
        }
        Some(SolutionRemarks { text, class: "info" })
    }
}

/// Проверяет, присутствуют ли в среди компонентов смеси дополнительные соединения,
/// вроде Ca, S, Cl и т.п., и, если их нет, вносит в remarks своё замечание.
/// Используется в `format_solution`.
fn check_admixtures(components: &[(Fertilizer, f64)]) -> Vec<SolutionRemarks> {
    let mut result = Vec::<SolutionRemarks>::with_capacity(2);
    if let Some(s_remark) = check_mixture_bonus_element(
        components,
        consts::PERMANENT_FERTILIZERS,
        "В смеси нет или очень мало <strong>серы</strong> (S). \
         Если смесь предназначена для ящиков-гряд или для выращивания \
         рассады&nbsp;&mdash;&nbsp;рекомендуем использовать удобрения, \
         содержащие серу: ",
        |f| f.with_S,
    ) {
        result.push(s_remark);
    }
    if let Some(cl_remark) = check_mixture_bonus_element(
        components,
        consts::PERMANENT_FERTILIZERS,
        "Похоже, что в смеси нет или очень мало <strong>хлора</strong> (Cl).
         Обычно в почве его содержится достаточно, но если смесь делается для ящиков-гряд
         или для выращивания рассады на бедном опилочном грунте \
         &nbsp;&mdash;&nbsp;рекомендуем использовать хотя бы немного удобрений, \
         содержащих хлор: ",
        |f| f.with_Cl,
    ) {
        result.push(cl_remark);
    }
    result
}

/// Временно хранит как основные макро-компоненты смеси и их количества, так и результаты всех
/// дополнительных вычислений о количестве микроэлементов, дозы разведения, примечания об улучшении
/// состава. Используется для отображения результатов.
#[derive(Debug, PartialEq)]
pub(crate) struct FormattedSolution {
    pub components: Vec<(Fertilizer, f64)>,
    pub concentration: Amounts,
    pub relation: [(ElemRangeName, f64); 3],
    pub microferts: Vec<(&'static str, Vec<(&'static str, f64)>)>,
    pub microferts_2a: Vec<(&'static str, Vec<(&'static str, f64)>)>,
    pub microferts_mit: Vec<(&'static str, Vec<(&'static str, f64)>)>,
    pub ground_dozes: Vec<(&'static str, ElemRange)>,
    pub mit_ground_doze: f64,
    pub remarks: Vec<SolutionRemarks>,
    pub seedling_dozes: Vec<(f64, f64, f64)>,
    pub total_weight: f64,
}

impl FormattedSolution {
    // Оформляет результаты вычислений solve_task в пригодный для показа вид, заодно проверяя
    // их на соответствие условиям задачи"
    #[allow(non_snake_case)]
    pub fn new(query: &MixtureQuery, solution: &LPSolution) -> Self {
        let scale_factor = query.mass / solution.function_value;
        let mut components = Vec::<(Fertilizer, f64)>::with_capacity(query.fertilizers.len());
        let mut concentration = Amounts::default();
        let mut quantity = Amounts::default();

        // quantity = {'N':0,'P':0,'K':0,'Mg':0}
        let mut remarks = Vec::<SolutionRemarks>::new();
        // Подсчёт концентрации и количества каждого элемента в смеси
        for (i, fert) in query.fertilizers.iter().enumerate() {
            // let name,composition = fert
            let weight = scale_factor * solution.params[i];
            let calc_concentration = |elem_percentage| elem_percentage * weight / query.mass;
            concentration.N += calc_concentration(fert.N);
            concentration.P += calc_concentration(fert.P);
            concentration.K += calc_concentration(fert.K);
            concentration.Mg += calc_concentration(fert.Mg);
            quantity.N += weight * fert.N / 100.0;
            quantity.P += weight * fert.P / 100.0;
            quantity.K += weight * fert.K / 100.0;
            quantity.Mg += weight * fert.Mg / 100.0;
            components.push((fert.clone(), weight));
        }
        // Дозы микроудобрений
        let microferts = calc_microferts(&consts::MICROFERTS, &quantity);
        let microferts_2a = calc_microferts(&consts::MICROFERTS_2A, &quantity);
        let microferts_mit = calc_microferts(&consts::MICROFERTS_MIT, &quantity);
        // Рассчёт дозы внесения на разных почвах (исходя из концентрации)
        let avg_N_K_concentration = (concentration.N + concentration.K) / 2.0;
        let doze_factor = consts::STD_N_K_CONCENTRATION / avg_N_K_concentration;
        if concentration.N.min(concentration.K) < consts::CRITICAL_LOW_N_K_CONCENTRATION {
            remarks.push(SolutionRemarks {
                class: "critical",
                text: String::from(
                    "Эту смесь крайне не рекомендуется применять! \
                   Она очень низкоконцентрированная, и в почву с ней попадёт \
                   слишком много лишних (а часто и вредных) веществ!",
                ),
            });
        }
        remarks.extend(check_admixtures(&components));
        let mut ground_dozes = Vec::new();
        for (ground_type, min_max) in consts::DOZES {
            let (min_doze, max_doze) = (min_max.from * doze_factor, min_max.to * doze_factor);
            ground_dozes.push((
                *ground_type,
                ElemRange {
                    from: min_doze.min(consts::MAX_DOZE),
                    to: max_doze.min(consts::MAX_DOZE),
                },
            ));
        }
        let mit_ground_doze =
            (100.0 * consts::MIT_N_PER_METER / concentration.N).min(consts::MIT_MAX_DOZE);
        // Рассчёт дозы для удобрительного полива рассады
        let doze_per_liter = consts::N_PER_LITER_2A / (concentration.N / 100.0);
        let doze_per_liter_mit = consts::N_PER_LITER_MIT / (concentration.N / 100.0);
        let seedling_dozes: Vec<_> = [3_f64, 5_f64, 10_f64]
            .iter()
            .map(|volume| {
                (
                    *volume,
                    volume * doze_per_liter,
                    volume * doze_per_liter_mit,
                )
            })
            .collect();
        // Соотношения элементов
        let relation = [
            (ElemRangeName::Nitrogen, concentration.N / concentration.P),
            (ElemRangeName::Potassium, concentration.K / concentration.P),
            (ElemRangeName::Magnesium, concentration.Mg / concentration.P),
        ];
        FormattedSolution {
            components,
            concentration,
            relation,
            microferts,
            microferts_2a,
            microferts_mit,
            ground_dozes,
            mit_ground_doze,
            remarks,
            seedling_dozes,
            total_weight: solution.function_value * scale_factor,
        }
    }
}

pub(crate) type MicroFertWeight = (&'static str, f64);
pub(crate) type MicroFertInfo = (&'static str, &'static [MicroFertWeight]);
pub(crate) type DynMicroFertInfo = (&'static str, Vec<MicroFertWeight>);
