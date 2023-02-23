use crate::calculator::formatted_solution::FormattedSolution;
use crate::calculator::modified_simplex::{LPTask, SimplexError};
use crate::calculator::{Amounts, Deficites, ElemRange, Fertilizer};
use std::iter::repeat;

use super::consts;
use super::modified_simplex::ConstraintOp;

#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub(crate) struct MixtureQuery {
    pub fertilizers: Vec<Fertilizer>,
    pub N_ratio: ElemRange,
    pub K_ratio: ElemRange,
    pub Mg_ratio: ElemRange,
    pub mass: f64,
}

/// Добавляет в систему сразу две строки ограничений вида
///
///     c1 * x1 + c2 * x2 + ... >= from
///     c1 * x1 + c2 * x2 + ... <= to
fn add_range_constraints(task: &mut LPTask, constraints: &[f64], range: &ElemRange) {
    task.add_constr(constraints, ConstraintOp::GreaterOrEqual, range.from);
    task.add_constr(constraints, ConstraintOp::LessOrEqual, range.to);
}

impl MixtureQuery {
    fn parse_amount(text: &str) -> Option<f64> {
        text.replace(',', ".")
            .parse::<f64>()
            .ok()
            .and_then(|v| if v >= 0.0 { Some(v) } else { None })
    }

    fn parse_bool(text: &str) -> Option<bool> {
        match text {
            "t" => Some(true),
            "f" => Some(false),
            _ => None,
        }
    }

    /// Разбирает строки вида "Азофоска:N:16.0,P:16.0,K:16.0,Mg:0.0" в соответсвующие структуры,
    /// удобрений, если это возможно.
    #[allow(non_snake_case)]
    fn parse_fertilizer_from_query(
        fert_str: &str,
        permanent_fertilizers: &[Fertilizer],
    ) -> Option<Fertilizer> {
        let (name, remainder) = fert_str.split_once(':')?;
        let (mut N, mut P, mut K, mut Mg, mut Cl, mut S, mut limit) =
            (None, None, None, None, None, None, None);
        for elem_amount_str in remainder.split(',') {
            let (element_str, amount_str) = elem_amount_str.split_once(':')?;
            match element_str {
                "Cl" => Cl = Some(Self::parse_bool(amount_str)?),
                "S" => S = Some(Self::parse_bool(amount_str)?),
                _ => {
                    let amount = Self::parse_amount(amount_str);
                    match element_str {
                        "N" => N = amount,
                        "P" => P = amount,
                        "K" => K = amount,
                        "Mg" => Mg = amount,
                        "lim" => limit = amount,
                        _ => return None,
                    }
                }
            }
        }
        let mut result = Fertilizer {
            limit,
            name: name.replace('+', " ").into(),
            N: N?,
            P: P?,
            K: K?,
            Mg: Mg?,
            with_Cl: Cl.unwrap_or_default(),
            with_S: S.unwrap_or_default(),
            id: Fertilizer::new_id(),
        };
        // старый legom не умел включать содержание серы в состав удобрения
        // в GET-запросе, но в базе этот флаг был, и данные выцеплялись оттуда.
        // Здесь мы делаем аналогичный трюк, для сохранения работоспособности ссылок.
        // Будем дополнительно уточнять для каждого распарсенного удобрения
        // если он присутствует в предопределённой таблице,
        // и вытаскивать оттуда уточнения по хлору и сере, если возможно.
        if Cl.is_none() || S.is_none() {
            let result_content_id = result.content_id();
            if let Some(permanent_match) = permanent_fertilizers
                .iter()
                .find(|f| f.content_id() == result_content_id)
            {
                if Cl.is_none() {
                    result.with_Cl = permanent_match.with_Cl;
                }
                if S.is_none() {
                    result.with_S = permanent_match.with_S;
                }
            }
        }
        Some(result)
    }

    pub fn from_query_map(query: &[(String, String)]) -> Option<Self> {
        const NON_FERT_PARAMS: usize = 7;
        let mut fertilizers =
            Vec::<Fertilizer>::with_capacity(query.len().max(NON_FERT_PARAMS) - NON_FERT_PARAMS);
        let (mut mg_from, mut mg_to, mut n_from, mut n_to, mut k_from, mut k_to, mut weight) =
            (None, None, None, None, None, None, None);
        for (param, data) in query {
            if param == "fert" {
                let fertilizer =
                    Self::parse_fertilizer_from_query(data, consts::PERMANENT_FERTILIZERS)?;
                fertilizers.push(fertilizer);
            } else {
                let amount = Self::parse_amount(data);
                match param.as_str() {
                    "PMg_from" => mg_from = amount,
                    "PMg_to" => mg_to = amount,
                    "NP_from" => n_from = amount,
                    "NP_to" => n_to = amount,
                    "PK_from" => k_from = amount,
                    "PK_to" => k_to = amount,
                    "need_weight" => weight = amount,
                    _ => {
                        return None;
                    }
                }
            }
        }
        Some(Self {
            fertilizers,
            N_ratio: ElemRange::try_new(n_from?, n_to?)?,
            Mg_ratio: ElemRange::try_new(mg_from?, mg_to?)?,
            K_ratio: ElemRange::try_new(k_from?, k_to?)?,
            mass: weight?,
        })
    }

    pub fn to_url_query(&self) -> Option<Vec<(&'static str, String)>> {
        let mut output = if self.fertilizers.is_empty() {
            return None;
        } else {
            let mut output =
                Vec::<(&'static str, String)>::with_capacity(self.fertilizers.len() + 7);
            for fert in self.fertilizers.iter().map(|f| f.urlencode()) {
                output.push(("fert", fert));
            }
            output
        };
        output.push(("PMg_from", self.Mg_ratio.from.to_string()));
        output.push(("PMg_to", self.Mg_ratio.to.to_string()));
        output.push(("NP_from", self.N_ratio.from.to_string()));
        output.push(("NP_to", self.N_ratio.to.to_string()));
        output.push(("PK_from", self.K_ratio.from.to_string()));
        output.push(("PK_to", self.K_ratio.to.to_string()));
        output.push(("need_weight", self.mass.to_string()));
        Some(output)
    }

    /// По указанным параметрам, строит систему ограничений - задачу линейного программирования
    pub fn build_task(self: &MixtureQuery, extra_fertilizers: &[Fertilizer]) -> LPTask {
        const P_NEIGHBOR: f64 = 1e-9;
        const P_RATIO: ElemRange = ElemRange { from: 1.0 - P_NEIGHBOR, to: 1.0 + P_NEIGHBOR };
        // ferts_count = len(checked_ferts)
        let num_ferts = self.fertilizers.len() + extra_fertilizers.len();
        let func_vec: Vec<f64> = repeat(1.0).take(num_ferts).collect();
        let mut task = LPTask::new(&func_vec);
        let mut n_constr = Vec::with_capacity(num_ferts);
        let mut p_constr = Vec::with_capacity(num_ferts);
        let mut k_constr = Vec::with_capacity(num_ferts);
        let mut mg_constr = Vec::with_capacity(num_ferts);
        let mut fert_constraints_buffer = vec![1.0f64; num_ferts]; // все кроме одного будут = 1.0
        for (fert_idx, fertilizer) in self
            .fertilizers
            .iter()
            .chain(extra_fertilizers.iter())
            .enumerate()
        {
            n_constr.push(fertilizer.N / 100.0);
            p_constr.push(fertilizer.P / 100.0);
            k_constr.push(fertilizer.K / 100.0);
            mg_constr.push(fertilizer.Mg / 100.0);
            if let Some(limit) = fertilizer.limit {
                // Добавляем уравнение, которое отражает ограниченную долю удобрения "n" в смеси.
                // Известно, что $R_n = limit_n / mass$ - это максимально возможная доля удобрения
                // n с массой $X_n$ в финальной смеси.
                // Также известно, что $mass = X_1 + X_2 + .. + X_n$.
                // Объединяя, получаем неравенство $X_n / (X_1 + X_2 + .. + X_n) <= R$.
                // После преобразования, имеем каноническую форму ограничения для системы:
                // $X_1 + X_2 + .. + (1 - 1/R) * X_n >= 0$
                let max_rate = limit / self.mass;
                fert_constraints_buffer[fert_idx] = 1.0 - 1.0 / max_rate; // остальные уже заполнены
                task.add_constr(&fert_constraints_buffer, ConstraintOp::GreaterOrEqual, 0.0);
                fert_constraints_buffer[fert_idx] = 1.0; // восстанавливаем заливку для нового цикла
            }
        }
        add_range_constraints(&mut task, &n_constr, &self.N_ratio);
        add_range_constraints(&mut task, &k_constr, &self.K_ratio);
        add_range_constraints(&mut task, &mg_constr, &self.Mg_ratio);
        add_range_constraints(&mut task, &p_constr, &P_RATIO);
        task
    }

    pub fn find_solution(&self) -> Result<FormattedSolution, SimplexError> {
        let lp_solution = self.build_task(&[]).solve_min()?;
        Ok(FormattedSolution::new(self, &lp_solution))
    }

    fn fake_fert_concentration<F>(&self, elem_extractor: F) -> f64
    where
        F: Fn(&Fertilizer) -> f64,
    {
        0.5 * self
            .fertilizers
            .iter()
            .filter_map(|f| {
                let concentration = elem_extractor(f);
                if concentration > 0.0 {
                    Some(concentration)
                } else {
                    None
                }
            })
            .min_by(f64::total_cmp)
            .unwrap_or(2e-3)
    }

    /// Вычислет дефицит каждого элемента через добавление псевдо-удобрений,
    /// каждое содержащее только один элемент питания в экстремально низкой
    /// концентрации. Если это удобрение будет выбрано, значит данного элемента
    /// отчаянно недостаёт. Также будет понятно, насколько велика недостача.
    pub fn find_deficites(&self) -> Deficites {
        let fake_npkmg_ferts = [
            Fertilizer { N: self.fake_fert_concentration(|f| f.N), ..Default::default() },
            Fertilizer { P: self.fake_fert_concentration(|f| f.P), ..Default::default() },
            Fertilizer { K: self.fake_fert_concentration(|f| f.K), ..Default::default() },
            Fertilizer { Mg: self.fake_fert_concentration(|f| f.Mg), ..Default::default() },
        ];

        let mut lacking = Amounts::default();
        if let Ok(solution) = self.build_task(&fake_npkmg_ferts).solve_min() {
            let scale_factor = self.mass / solution.function_value;
            for (i, fert) in fake_npkmg_ferts.iter().enumerate() {
                // let name,composition = fert
                let weight = scale_factor * solution.params[self.fertilizers.len() + i];
                lacking.N += weight * fert.N / 100.0;
                lacking.P += weight * fert.P / 100.0;
                lacking.K += weight * fert.K / 100.0;
                lacking.Mg += weight * fert.Mg / 100.0;
            }
        }
        Deficites {
            N: lacking.N > 0.0,
            P: lacking.P > 0.0,
            K: lacking.K > 0.0,
            Mg: lacking.Mg > 0.0,
        }
    }
}
