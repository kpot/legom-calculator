/// Реализация модифицированного симплекс-метода, описанного
/// в книге Хемди А. Таха "Введение в исследование операций"
/// [7-е издание] (стр. 329)
// from base import SimplexMethod
// import numpy as np
// from math import factorial
use nalgebra::{self, DMatrix, DVector};

const MAX_COMBINATIONS: usize = 400000;
const MAX_STEPS: usize = 1500;

/// Вычисление всех возможных сочетаний без повторений.
/// Используется при нахождении начального базисного решения.
struct Combinator {
    used: Vec<usize>,
    n: usize,
    k: usize,
    first_step: bool,
}

impl Combinator {
    fn new(n: usize, k: usize) -> Self {
        Combinator { n, k, used: (0..k).collect(), first_step: true }
    }

    pub fn sequence_len(&self) -> usize {
        fn factorial(n: usize) -> usize {
            (2..=n).fold(1, usize::saturating_mul)
        }
        factorial(self.n) / (factorial(self.k) * factorial(self.n - self.k))
    }

    fn next(&mut self) -> Option<&[usize]> {
        if self.first_step {
            self.first_step = false;
            Some(&self.used[..])
        } else {
            for i in (0..self.k).rev() {
                let max_current = if i + 1 == self.k {
                    self.n - 1
                } else {
                    self.used[i + 1] - 1
                };
                if self.used[i] < max_current {
                    self.used[i] += 1;
                    for t in (i + 1)..self.k {
                        self.used[t] = self.used[t - 1] + 1;
                    }
                    return Some(&self.used[..]);
                }
            }
            None
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SimplexError {
    // система составлена неверно, т.к. не содержит"
    //  небазисных переменных. увеличьте число дополнительных"
    // переменных (например, заменив равенства на неравенства).")
    #[error("problem is not feasible")]
    NotFeasible,
    #[error("the problem is too big: {0}")]
    BigTask(DMatrix<f64>),
    #[error("problem is not bounded")]
    Unbounded,
    #[error("maximum number of execution steps has been reached")]
    TooManySteps,
    #[error("no base solution exists")]
    NoBaseSolution,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConstraintOp {
    Equal,
    Less,
    Greater,
    GreaterOrEqual,
    LessOrEqual,
}

#[derive(Debug)]
pub struct Constraint {
    pub left: DVector<f64>, // Row of the "A" matrix
    pub op: ConstraintOp,
    pub right: f64, // the "b" value from the right
}

impl From<(&[f64], ConstraintOp, f64)> for Constraint {
    fn from(value: (&[f64], ConstraintOp, f64)) -> Self {
        Self { left: DVector::from(value.0.to_vec()), op: value.1, right: value.2 }
    }
}

/// Задача линейного программирования
#[derive(Debug)]
pub struct LPTask {
    func_vec: DVector<f64>,
    constr: Vec<Constraint>,
}

impl LPTask {
    pub fn new(func_vec: &[f64]) -> Self {
        Self { func_vec: DVector::from_row_slice(func_vec), constr: Default::default() }
    }

    /// Добавляет новое ограничение, попутно приводя его к форме с положительной правой частью."
    pub fn add_constr(&mut self, left: &[f64], op: ConstraintOp, right: f64) {
        let mut constraint = Constraint { right, op, left: left.to_vec().into() };
        assert_eq!(
            constraint.left.shape(),
            self.func_vec.shape(),
            "Длина вектора ограничений должна совпадать с длинной \
                        вектора оптимизируемой функции"
        );
        // Приведение ограничения к нормальной форме, с положительной правой частью
        // TODO: Не добавлять дубликаты ограничений
        if constraint.right < 0.0 {
            for v in constraint.left.iter_mut() {
                *v = -*v;
            }
            constraint.right = -constraint.right;
            constraint.op = match constraint.op {
                ConstraintOp::GreaterOrEqual => ConstraintOp::LessOrEqual,
                ConstraintOp::LessOrEqual => ConstraintOp::GreaterOrEqual,
                ConstraintOp::Less => ConstraintOp::Greater,
                ConstraintOp::Greater => ConstraintOp::Less,
                ConstraintOp::Equal => ConstraintOp::Equal,
            };
        }
        self.constr.push(constraint);
    }

    #[allow(non_snake_case)]
    pub fn solve_min(&self) -> Result<LPSolution, SimplexError> {
        let A = self.get_source_matrix();
        let mut basis = find_basis_solution(&A)?;
        if basis.non_basis_cols.is_empty() {
            return Err(SimplexError::NotFeasible);
        }
        let mut counter = 0_usize;
        loop {
            if counter > MAX_STEPS {
                return Err(SimplexError::TooManySteps);
            }
            let A_last_row = A.row(A.nrows() - 1);
            let C_B = A_last_row.select_columns(&basis.basis_cols);
            let A_but_last_rows = A.rows(0, A.nrows() - 1);
            let B = A_but_last_rows.select_columns(&basis.basis_cols);
            let b = A_but_last_rows.column(A.ncols() - 1);
            let inv_B = B.clone_owned().try_inverse().expect(
                "The matrix is expected to be invertible, as verified by find_basis_solution",
            );
            let X_B = &inv_B * b;
            let noB = A_but_last_rows.select_columns(&basis.non_basis_cols);
            let noC = A_last_row.select_columns(&basis.non_basis_cols);
            let z_minus_c = &C_B * &inv_B * &noB - &noC;
            if z_minus_c.iter().all(|i| *i <= 0.0) {
                // Если для всех небазисных переменных величины z - с >= 0
                // в задаче максимизации или z. - с <= 0
                // в задаче минимизации, то вычисления заканчиваются, так
                // как получено оптимальное решение
                let func_val = &C_B * &X_B;
                let solution = X_B.transpose();

                return Ok(LPSolution {
                    function_value: func_val[(0, 0)],
                    params: (0..self.func_vec.len())
                        .map(|i| match basis.basis_cols.iter().position(|n| *n == i) {
                            Some(var_pos) => solution[(0, var_pos)],
                            None => 0.0,
                        })
                        .collect(),
                });
            }
            // Ищем вводимую переменную
            let potential_intr_vec_ind = z_minus_c
                .row(0)
                .iter()
                .enumerate()
                .filter(|(_, v)| **v > 0.0)
                .max_by(|(_, v1), (_, v2)| v1.total_cmp(v2))
                .map(|(i, _)| i);
            let intr_vec_ind = match potential_intr_vec_ind {
                None => return Err(SimplexError::NotFeasible),
                Some(idx) if z_minus_c.row(0)[idx].is_nan() => {
                    return Err(SimplexError::NotFeasible);
                }
                Some(idx) => idx,
            };
            // Ищем исключаемую переменную
            let N = &inv_B * A_but_last_rows.column(basis.non_basis_cols[intr_vec_ind]);
            let candidates = X_B.component_div(&N);
            let excl_vec_ind = (0..candidates.nrows())
                .map(|i| (i, candidates.row(i)[0]))
                .filter(|(_, c)| c.is_finite() && *c > 0.0)
                .min_by(|(_, c1), (_, c2)| c1.total_cmp(c2))
                .map(|(i, _)| i)
                .ok_or(SimplexError::NotFeasible)?;
            let m_val = candidates[(excl_vec_ind, 0)];
            if m_val.is_nan() {
                return Err(SimplexError::Unbounded);
            }
            // Обмен между базисными и небазисными векторами
            basis
                .basis_cols
                .push(basis.non_basis_cols.remove(intr_vec_ind));
            basis
                .non_basis_cols
                .push(basis.basis_cols.remove(excl_vec_ind));
            counter += 1;
        }
    }

    /// Возвращает исходную матрицу A, с уже добавленными
    /// фиктивными переменными
    #[allow(non_snake_case)]
    fn get_source_matrix(&self) -> DMatrix<f64> {
        let fict_vars_count = self
            .constr
            .iter()
            .filter(|row| {
                matches!(
                    row.op,
                    ConstraintOp::LessOrEqual | ConstraintOp::GreaterOrEqual
                )
            })
            .count();
        let mut M = DMatrix::<f64>::zeros(
            self.constr.len() + 1,
            self.func_vec.len() + fict_vars_count + 1,
        );
        let mut fict_ind = 0;
        for (i, Constraint { left, op, right }) in self.constr.iter().enumerate() {
            let extra_columns = match op {
                ConstraintOp::Greater | ConstraintOp::GreaterOrEqual => {
                    let extra = DVector::<f64>::from_fn(fict_vars_count, |a, _| {
                        if a == fict_ind {
                            -1.0
                        } else {
                            0.0
                        }
                    });
                    fict_ind += 1;
                    extra
                }
                ConstraintOp::Less | ConstraintOp::LessOrEqual => {
                    let extra = DVector::<f64>::from_fn(fict_vars_count, |a, _| {
                        if a == fict_ind {
                            1.0
                        } else {
                            0.0
                        }
                    });
                    fict_ind += 1;
                    extra
                }
                ConstraintOp::Equal => DVector::<f64>::zeros(fict_vars_count),
            };
            let row_vec = DVector::from_iterator(
                left.len() + extra_columns.len() + 1,
                left.iter()
                    .chain(extra_columns.iter())
                    .copied()
                    .chain(std::iter::once(*right)),
            )
            .transpose();
            M.set_row(i, &row_vec);
        }

        let last_row = DVector::from_iterator(
            self.func_vec.len() + fict_vars_count + 1,
            self.func_vec
                .iter()
                .copied()
                .chain(std::iter::repeat(0.0).take(fict_vars_count + 1)),
        )
        .transpose();
        M.set_row(self.constr.len(), &last_row);
        M
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct BasisSolution {
    solution: DMatrix<f64>,
    basis_cols: Vec<usize>,
    non_basis_cols: Vec<usize>,
}

/// Подбор начального базисного решения. Это набор векторов, входящих
/// в базис на первой итерации модифицированного симплекс-метода.
/// Матрица базисных векторов должна иметь обратную (det(B) != 0)
/// и при решении BX=b вектор X не должен содержать отрицательных элементов
/// (т.к. x1,x2,x3... >=0).
#[allow(non_snake_case)]
fn find_basis_solution(A: &DMatrix<f64>) -> Result<BasisSolution, SimplexError> {
    // from numpy.linalg import inv, det
    let basis_size = A.nrows() - 1;
    // Размер матрицы базиса должен быть равен
    // числу элементов правой части системы
    // Перебор всех сочетаний (без повторений) векторов,
    // в поисках базисного сочетания
    //
    // taking the last column vector, shourt of 1 last element
    let A_but_last_row = A.rows(0, A.nrows() - 1);
    let b = A_but_last_row.column(A.ncols() - 1);

    let mut combinator = Combinator::new(A.ncols() - 1, basis_size);
    if combinator.sequence_len() > MAX_COMBINATIONS {
        return Err(SimplexError::BigTask(A.clone()));
    }
    while let Some(basis_cols) = combinator.next() {
        let B = A_but_last_row.select_columns(basis_cols);
        let B_det = B.determinant();
        if B_det.abs() > f64::MIN_POSITIVE {
            if let Some(B_inv) = B.clone_owned().try_inverse() {
                let Xn = &B_inv * b;
                let Xn_min = Xn
                    .iter()
                    .min_by(|a, b| a.total_cmp(b))
                    .copied()
                    .unwrap_or_default();
                if Xn_min >= 0.0 {
                    let var_ids = 0..(A.ncols() - 1);
                    let non_basis_cols = var_ids.filter(|i| !basis_cols.contains(i)).collect();
                    return Ok(BasisSolution {
                        non_basis_cols,
                        solution: B,
                        basis_cols: basis_cols.into(),
                    });
                }
            }
        }
    }
    Err(SimplexError::NoBaseSolution)
}

#[derive(Debug, Clone)]
pub struct LPSolution {
    pub function_value: f64,
    pub params: Vec<f64>,
}

#[cfg(test)]
mod test {
    use super::{Combinator, ConstraintOp, LPTask};

    const E_MAX: f64 = f64::EPSILON;
    const EXPECT_SOLUTION: &str = "Must be solvable";

    #[test]
    fn test_combinator() {
        let n = 5;
        let k = 3;
        let num_expected_combinations = 10;
        let mut c = Combinator::new(n, k);
        assert_eq!(c.sequence_len(), num_expected_combinations);
        let mut result = vec![];
        while let Some(v) = c.next() {
            result.push(Vec::from(v));
        }
        assert_eq!(result.len(), num_expected_combinations);
        assert_eq!(
            result,
            vec![
                [0, 1, 2],
                [0, 1, 3],
                [0, 1, 4],
                [0, 2, 3],
                [0, 2, 4],
                [0, 3, 4],
                [1, 2, 3],
                [1, 2, 4],
                [1, 3, 4],
                [2, 3, 4]
            ]
        );
    }

    fn vec_diff(v1: &[f64], v2: &[f64]) -> f64 {
        assert_eq!(v1.len(), v2.len());
        v1.iter()
            .zip(v2.iter())
            .map(|(x1, x2)| (x1 - x2).abs())
            .sum::<f64>()
            / v1.len() as f64
    }

    #[test]
    fn test_simplex1() {
        let mut t1 = LPTask::new(&[1., 1.]); // MIN must be (0.9, [0.2, 0.7])
        t1.add_constr(&[3., 2.], ConstraintOp::GreaterOrEqual, 2.);
        t1.add_constr(&[1., 4.], ConstraintOp::GreaterOrEqual, 3.);
        let solution = t1.solve_min().expect(EXPECT_SOLUTION);
        assert!((solution.function_value - 0.9).abs() < E_MAX);
        assert!(vec_diff(&solution.params, &[0.2, 0.7]) < E_MAX);
    }

    #[test]
    fn test_simplex2() {
        let mut t2 = LPTask::new(&[1., -2.]);
        t2.add_constr(&[1., 1.], ConstraintOp::GreaterOrEqual, 1.);
        t2.add_constr(&[2., -3.], ConstraintOp::GreaterOrEqual, 1.);
        t2.add_constr(&[4., -5.], ConstraintOp::Equal, 6.);
        let solution = t2.solve_min().expect(EXPECT_SOLUTION);
        assert!((solution.function_value - (-1.5)).abs() < E_MAX);
        assert!(vec_diff(&solution.params, &[6.5, 4.0]) < E_MAX);
    }

    #[test]
    fn test_simplex3() {
        let mut t3 = LPTask::new(&[1., 1., 1., 1.]);
        t3.add_constr(&[0., 0., 0., 0.08], ConstraintOp::GreaterOrEqual, 0.4);
        t3.add_constr(&[0., 0., 0., 0.08], ConstraintOp::LessOrEqual, 0.5);
        t3.add_constr(&[0.19, 0., 0.16, 0.], ConstraintOp::Equal, 1.0);
        t3.add_constr(&[0., 0.34, 0.16, 0.], ConstraintOp::GreaterOrEqual, 1.7);
        t3.add_constr(&[0., 0.34, 0.16, 0.], ConstraintOp::LessOrEqual, 1.9);
        t3.add_constr(&[0., 0., 0.16, 0.35], ConstraintOp::GreaterOrEqual, 1.7);
        t3.add_constr(&[0., 0., 0.16, 0.35], ConstraintOp::LessOrEqual, 1.9);
        let solution = t3.solve_min().expect(EXPECT_SOLUTION);
        assert!((solution.function_value - 14.970007739938081).abs() < E_MAX);
        assert!(
            vec_diff(
                &solution.params,
                &[
                    4.473684210526317,
                    4.5588235294117645,
                    0.9375000000000003,
                    5.0
                ]
            ) < E_MAX
        );
    }

    #[test]
    fn test_simplex4() {
        let mut t4 = LPTask::new(&[1., 1., 1., 1., 1., 1., 1.]);
        t4.add_constr(
            &[0., 0.08, 0.16, 0., 0., 0., 0.],
            ConstraintOp::GreaterOrEqual,
            0.2,
        );
        t4.add_constr(
            &[0., 0.08, 0.16, 0., 0., 0., 0.],
            ConstraintOp::LessOrEqual,
            0.5,
        );
        t4.add_constr(
            &[0.6, 0.35, 0., 0.16, 0., 0., 0.],
            ConstraintOp::GreaterOrEqual,
            1.7,
        );
        t4.add_constr(
            &[0.6, 0.35, 0., 0.16, 0., 0., 0.],
            ConstraintOp::LessOrEqual,
            1.9,
        );
        t4.add_constr(
            &[0., 0., 0., 0.16, 0.34, 0.12, 0.],
            ConstraintOp::GreaterOrEqual,
            1.7,
        );
        t4.add_constr(
            &[0., 0., 0., 0.16, 0.34, 0.12, 0.],
            ConstraintOp::LessOrEqual,
            1.9,
        );
        t4.add_constr(
            &[0., 0., 0., 0.16, 0., 0.52, 0.19],
            ConstraintOp::Equal,
            1.0,
        );
        let solution = t4.solve_min().expect(EXPECT_SOLUTION);
        assert!((solution.function_value - 10.119343891402714).abs() < E_MAX);
        assert!(
            vec_diff(
                &solution.params,
                &[
                    1.375,
                    2.5,
                    0.0,
                    0.0,
                    4.321266968325792,
                    1.9230769230769231,
                    0.0
                ]
            ) < E_MAX
        );
    }
}
