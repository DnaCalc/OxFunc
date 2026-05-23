use oxfunc_core::coercion::CoercionError;
use oxfunc_core::functions::{
    count::eval_count_surface, counta::eval_counta_surface, countblank_fn::eval_countblank_surface,
    sum::eval_sum_surface,
};
use oxfunc_core::resolver::{
    RefResolutionError, ReferenceResolver, ResolvedReferenceCell, ResolvedReferenceExtent,
    ResolvedReferenceValues, ResolverCapabilities,
};
use oxfunc_core::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceKind, ReferenceLike,
    WorksheetErrorCode,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

fn text(value: &str) -> ExcelText {
    ExcelText::from_utf16_code_units(value.encode_utf16().collect())
}

fn structured_arg(target: &str) -> CallArgValue {
    CallArgValue::Reference(ReferenceLike::new(ReferenceKind::Structured, target))
}

struct StructuredSparseResolver {
    caps: ResolverCapabilities,
    sparse_values_by_target: BTreeMap<String, ResolvedReferenceValues>,
    dense_calls: RefCell<Vec<ReferenceLike>>,
    sparse_calls: RefCell<Vec<ReferenceLike>>,
}

impl StructuredSparseResolver {
    fn with_values(values: BTreeMap<String, ResolvedReferenceValues>) -> Self {
        Self {
            caps: ResolverCapabilities::permissive_local(),
            sparse_values_by_target: values,
            dense_calls: RefCell::new(Vec::new()),
            sparse_calls: RefCell::new(Vec::new()),
        }
    }

    fn with_caps(caps: ResolverCapabilities) -> Self {
        Self {
            caps,
            sparse_values_by_target: BTreeMap::new(),
            dense_calls: RefCell::new(Vec::new()),
            sparse_calls: RefCell::new(Vec::new()),
        }
    }
}

impl ReferenceResolver for StructuredSparseResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        self.caps
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        self.dense_calls.borrow_mut().push(reference.clone());
        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
    }

    fn resolve_reference_values(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, RefResolutionError> {
        self.sparse_calls.borrow_mut().push(reference.clone());
        Ok(self.sparse_values_by_target.get(&reference.target).cloned())
    }
}

#[test]
fn aggregate_group_consumes_structured_table_sparse_readers_as_opaque_references() {
    let data_column = "treecalc-table://Revenue[#Data][Amount]";
    let whole_data = "treecalc-table://Revenue[#Data]";
    let headers = "treecalc-table://Revenue[#Headers]";
    let totals = "treecalc-table://Revenue[#Totals][Amount]";
    let current_row = "treecalc-table://Revenue[@Amount]";

    let mut sparse_values_by_target = BTreeMap::new();
    sparse_values_by_target.insert(
        data_column.to_string(),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(4, 1),
            vec![
                ResolvedReferenceCell::new(1, 1, ArrayCellValue::Number(2.0)),
                ResolvedReferenceCell::new(2, 1, ArrayCellValue::Text(text("ignored"))),
                ResolvedReferenceCell::new(3, 1, ArrayCellValue::Logical(true)),
                ResolvedReferenceCell::new(4, 1, ArrayCellValue::Number(3.0)),
            ],
            Some("reader:treecalc-table:Revenue:data-column:Amount".to_string()),
        ),
    );
    sparse_values_by_target.insert(
        whole_data.to_string(),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(2, 2),
            vec![
                ResolvedReferenceCell::new(1, 1, ArrayCellValue::Number(9.0)),
                ResolvedReferenceCell::new(1, 2, ArrayCellValue::Text(text("label"))),
                ResolvedReferenceCell::new(2, 1, ArrayCellValue::Logical(true)),
            ],
            Some("reader:treecalc-table:Revenue:data-body".to_string()),
        ),
    );
    sparse_values_by_target.insert(
        headers.to_string(),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 2),
            vec![
                ResolvedReferenceCell::new(1, 1, ArrayCellValue::Text(text("Amount"))),
                ResolvedReferenceCell::new(1, 2, ArrayCellValue::Text(text("Region"))),
            ],
            Some("reader:treecalc-table:Revenue:headers".to_string()),
        ),
    );
    sparse_values_by_target.insert(
        totals.to_string(),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 3),
            vec![
                ResolvedReferenceCell::new(1, 1, ArrayCellValue::Text(text(""))),
                ResolvedReferenceCell::new(1, 3, ArrayCellValue::Number(14.0)),
            ],
            Some("reader:treecalc-table:Revenue:totals".to_string()),
        ),
    );
    sparse_values_by_target.insert(
        current_row.to_string(),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 1),
            vec![ResolvedReferenceCell::new(
                1,
                1,
                ArrayCellValue::Number(11.0),
            )],
            Some("reader:treecalc-table:Revenue:this-row:Amount".to_string()),
        ),
    );

    let resolver = StructuredSparseResolver::with_values(sparse_values_by_target);

    assert_eq!(
        eval_sum_surface(&[structured_arg(data_column)], &resolver),
        Ok(EvalValue::Number(5.0))
    );
    assert_eq!(
        eval_count_surface(&[structured_arg(whole_data)], &resolver),
        Ok(EvalValue::Number(1.0))
    );
    assert_eq!(
        eval_counta_surface(&[structured_arg(headers)], &resolver),
        Ok(EvalValue::Number(2.0))
    );
    assert_eq!(
        eval_countblank_surface(&[structured_arg(totals)], &resolver),
        Ok(EvalValue::Number(2.0))
    );
    assert_eq!(
        eval_sum_surface(&[structured_arg(current_row)], &resolver),
        Ok(EvalValue::Number(11.0))
    );

    assert!(resolver.dense_calls.borrow().is_empty());
    let sparse_calls = resolver.sparse_calls.borrow();
    let seen: Vec<_> = sparse_calls
        .iter()
        .map(|reference| (reference.kind, reference.target.as_str()))
        .collect();
    assert_eq!(
        seen,
        vec![
            (ReferenceKind::Structured, data_column),
            (ReferenceKind::Structured, whole_data),
            (ReferenceKind::Structured, headers),
            (ReferenceKind::Structured, totals),
            (ReferenceKind::Structured, current_row),
        ]
    );
}

#[test]
fn structured_reference_capability_denial_is_generic_and_precedes_provider_calls() {
    let resolver = StructuredSparseResolver::with_caps(ResolverCapabilities {
        allow_eval_time_deref: true,
        allow_three_d_refs: true,
        allow_structured_refs: false,
        allow_spill_anchor_refs: true,
        allow_external_refs: false,
    });

    let got = eval_sum_surface(
        &[structured_arg("treecalc-table://Revenue[#Data][Amount]")],
        &resolver,
    );

    assert!(matches!(
        got,
        Err(oxfunc_core::functions::sum::SumEvalError::Coercion(
            CoercionError::RefResolution(RefResolutionError::CapabilityDenied {
                kind: ReferenceKind::Structured,
                capability: "allow_structured_refs",
            })
        ))
    ));
    assert!(resolver.dense_calls.borrow().is_empty());
    assert!(resolver.sparse_calls.borrow().is_empty());
}

#[test]
fn direct_scalar_and_array_paths_are_unchanged_by_structured_reference_guardrail() {
    let resolver = StructuredSparseResolver::with_values(BTreeMap::new());

    assert_eq!(
        eval_sum_surface(
            &[
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Text(text("2"))),
            ],
            &resolver,
        ),
        Ok(EvalValue::Number(3.0))
    );
    assert_eq!(
        eval_count_surface(
            &[
                CallArgValue::Eval(EvalValue::Number(1.0)),
                CallArgValue::Eval(EvalValue::Logical(true)),
                CallArgValue::Eval(EvalValue::Text(text("2"))),
                CallArgValue::Eval(EvalValue::Text(text("not numeric"))),
            ],
            &resolver,
        ),
        Ok(EvalValue::Number(3.0))
    );
    assert_eq!(
        eval_counta_surface(
            &[
                CallArgValue::EmptyCell,
                CallArgValue::Eval(EvalValue::Text(text("")))
            ],
            &resolver
        ),
        Ok(EvalValue::Number(1.0))
    );

    let countblank_array = EvalArray::from_rows(vec![vec![
        ArrayCellValue::Text(text("a")),
        ArrayCellValue::Text(text("")),
    ]])
    .unwrap();
    assert_eq!(
        eval_countblank_surface(
            &[CallArgValue::Eval(EvalValue::Array(countblank_array))],
            &resolver
        ),
        Ok(EvalValue::Array(
            EvalArray::from_rows(vec![vec![
                ArrayCellValue::Error(WorksheetErrorCode::Value),
                ArrayCellValue::Error(WorksheetErrorCode::Value),
            ]])
            .unwrap()
        ))
    );
    assert!(resolver.dense_calls.borrow().is_empty());
    assert!(resolver.sparse_calls.borrow().is_empty());
}
