use oxfunc_core::coercion::CoercionError;
use oxfunc_core::functions::{
    and_fn::eval_and_surface,
    average::eval_average_surface,
    cell::eval_cell_surface,
    columns_fn::eval_columns_surface_with_resolver,
    count::eval_count_surface,
    counta::eval_counta_surface,
    countblank_fn::eval_countblank_surface,
    criteria_family::{eval_countif_surface, eval_sumifs_surface},
    index::{IndexEvalError, eval_index_surface},
    match_fn::eval_match_surface,
    max_fn::eval_max_surface,
    reference_metadata_family::{eval_areas_surface, eval_formulatext_surface},
    rows_fn::eval_rows_surface_with_resolver,
    subtotal_aggregate_family::{eval_aggregate_surface, eval_subtotal_surface},
    sum::eval_sum_surface,
    textjoin::eval_textjoin_surface,
    xlookup::eval_xlookup_surface,
};
use oxfunc_core::host_info::{
    AggregateCellContext, AggregateReferenceContext, CellInfoQuery, HostInfoError, HostInfoProvider,
};
use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceIdentityKey,
    ReferenceResolutionError, ReferenceSystemCapabilities, ReferenceSystemError,
    ReferenceSystemOperation, ReferenceSystemProvider, ReferenceTransformKind,
    ReferenceTransformRequest, ResolvedReferenceCell, ResolvedReferenceExtent,
    ResolvedReferenceValues, reference_identity_key,
};
use oxfunc_core::value::{
    CalcArray, CalcValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

fn text(value: &str) -> ExcelText {
    ExcelText::from_utf16_code_units(value.encode_utf16().collect())
}

fn structured_arg(target: &str) -> CalcValue {
    CalcValue::reference(ReferenceLike::new(ReferenceKind::Structured, target))
}

fn structured_key(target: &str) -> ReferenceIdentityKey {
    reference_identity_key(&ReferenceLike::new(ReferenceKind::Structured, target))
}

fn scalar_number(n: f64) -> CalcValue {
    CalcValue::number(n)
}

fn scalar_text(value: &str) -> CalcValue {
    CalcValue::text(text(value))
}

fn scalar_logical(value: bool) -> CalcValue {
    CalcValue::logical(value)
}

fn sparse_values(
    rows: usize,
    cols: usize,
    cells: Vec<(usize, usize, CalcValue)>,
) -> ResolvedReferenceValues {
    ResolvedReferenceValues::new(
        ResolvedReferenceExtent::new(rows, cols),
        cells
            .into_iter()
            .map(|(row, col, value)| ResolvedReferenceCell::new(row, col, value))
            .collect(),
        Some(format!("reader:test:{rows}x{cols}")),
    )
}

struct StructuredSparseResolver {
    caps: ReferenceSystemCapabilities,
    sparse_values_by_reference: BTreeMap<ReferenceIdentityKey, ResolvedReferenceValues>,
    dense_calls: RefCell<Vec<ReferenceLike>>,
    sparse_calls: RefCell<Vec<ReferenceLike>>,
}

impl StructuredSparseResolver {
    fn with_values(values: BTreeMap<ReferenceIdentityKey, ResolvedReferenceValues>) -> Self {
        Self {
            caps: ReferenceSystemCapabilities::permissive_local(),
            sparse_values_by_reference: values,
            dense_calls: RefCell::new(Vec::new()),
            sparse_calls: RefCell::new(Vec::new()),
        }
    }

    fn with_caps(caps: ReferenceSystemCapabilities) -> Self {
        Self {
            caps,
            sparse_values_by_reference: BTreeMap::new(),
            dense_calls: RefCell::new(Vec::new()),
            sparse_calls: RefCell::new(Vec::new()),
        }
    }
}

impl ReferenceSystemProvider for StructuredSparseResolver {
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        self.caps
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let reference = &request.reference;
        self.dense_calls.borrow_mut().push(reference.clone());
        Err(ReferenceResolutionError::UnresolvedReference {
            target: reference.target().to_string(),
        })
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        let reference = &request.reference;
        self.sparse_calls.borrow_mut().push(reference.clone());
        Ok(self
            .sparse_values_by_reference
            .get(&reference_identity_key(reference))
            .cloned())
    }

    fn transform_reference(
        &self,
        request: &ReferenceTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        match request.transform {
            ReferenceTransformKind::Index { row, col, area: 1 } => Ok(ReferenceLike::new(
                request.reference.kind(),
                format!("{}#INDEX({row},{col})", request.reference.target()),
            )),
            _ => Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::Transform,
            }),
        }
    }
}

#[derive(Default)]
struct StructuredHostInfo {
    aggregate_contexts: BTreeMap<String, AggregateReferenceContext>,
    formula_text_calls: RefCell<Vec<ReferenceLike>>,
    cell_info_calls: RefCell<Vec<ReferenceLike>>,
}

impl HostInfoProvider for StructuredHostInfo {
    fn query_formula_text(&self, reference: &ReferenceLike) -> Result<CalcValue, HostInfoError> {
        self.formula_text_calls.borrow_mut().push(reference.clone());
        Ok(CalcValue::text(text("=SUM(Table[Amount])")))
    }

    fn query_cell_info(
        &self,
        query: CellInfoQuery,
        reference: Option<&ReferenceLike>,
    ) -> Result<CalcValue, HostInfoError> {
        if let Some(reference) = reference {
            self.cell_info_calls.borrow_mut().push(reference.clone());
        }
        match query {
            CellInfoQuery::Filename => Ok(CalcValue::text(text("Book.xlsx"))),
            other => Err(HostInfoError::UnsupportedCellInfoQuery(other)),
        }
    }

    fn query_aggregate_reference_context(
        &self,
        reference: &ReferenceLike,
    ) -> Result<AggregateReferenceContext, HostInfoError> {
        self.aggregate_contexts
            .get(reference.target())
            .cloned()
            .ok_or(HostInfoError::UnsupportedAggregateReferenceContextQuery)
    }
}

#[test]
fn aggregate_group_consumes_structured_table_sparse_readers_as_opaque_references() {
    let data_column = "treecalc-table://Revenue[#Data][Amount]";
    let whole_data = "treecalc-table://Revenue[#Data]";
    let headers = "treecalc-table://Revenue[#Headers]";
    let totals = "treecalc-table://Revenue[#Totals][Amount]";
    let current_row = "treecalc-table://Revenue[@Amount]";

    let mut sparse_values_by_reference = BTreeMap::new();
    sparse_values_by_reference.insert(
        structured_key(data_column),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(4, 1),
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(2.0)),
                ResolvedReferenceCell::new(2, 1, CalcValue::text(text("ignored"))),
                ResolvedReferenceCell::new(3, 1, CalcValue::logical(true)),
                ResolvedReferenceCell::new(4, 1, CalcValue::number(3.0)),
            ],
            Some("reader:treecalc-table:Revenue:data-column:Amount".to_string()),
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(whole_data),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(2, 2),
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::number(9.0)),
                ResolvedReferenceCell::new(1, 2, CalcValue::text(text("label"))),
                ResolvedReferenceCell::new(2, 1, CalcValue::logical(true)),
            ],
            Some("reader:treecalc-table:Revenue:data-body".to_string()),
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(headers),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 2),
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::text(text("Amount"))),
                ResolvedReferenceCell::new(1, 2, CalcValue::text(text("Region"))),
            ],
            Some("reader:treecalc-table:Revenue:headers".to_string()),
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(totals),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 3),
            vec![
                ResolvedReferenceCell::new(1, 1, CalcValue::text(text(""))),
                ResolvedReferenceCell::new(1, 3, CalcValue::number(14.0)),
            ],
            Some("reader:treecalc-table:Revenue:totals".to_string()),
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(current_row),
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(1, 1),
            vec![ResolvedReferenceCell::new(1, 1, CalcValue::number(11.0))],
            Some("reader:treecalc-table:Revenue:this-row:Amount".to_string()),
        ),
    );

    let resolver = StructuredSparseResolver::with_values(sparse_values_by_reference);

    assert_eq!(
        eval_sum_surface(&[structured_arg(data_column)], &resolver),
        Ok(CalcValue::number(5.0))
    );
    assert_eq!(
        eval_count_surface(&[structured_arg(whole_data)], &resolver),
        Ok(CalcValue::number(1.0))
    );
    assert_eq!(
        eval_counta_surface(&[structured_arg(headers)], &resolver),
        Ok(CalcValue::number(2.0))
    );
    assert_eq!(
        eval_countblank_surface(&[structured_arg(totals)], &resolver),
        Ok(CalcValue::number(2.0))
    );
    assert_eq!(
        eval_sum_surface(&[structured_arg(current_row)], &resolver),
        Ok(CalcValue::number(11.0))
    );

    assert!(resolver.dense_calls.borrow().is_empty());
    let sparse_calls = resolver.sparse_calls.borrow();
    let seen: Vec<_> = sparse_calls
        .iter()
        .map(|reference| (reference.kind(), reference.target()))
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
    let resolver = StructuredSparseResolver::with_caps(ReferenceSystemCapabilities {
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
            CoercionError::RefResolution(ReferenceResolutionError::CapabilityDenied {
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
            &[(CalcValue::logical(true)), (CalcValue::text(text("2"))),],
            &resolver,
        ),
        Ok(CalcValue::number(3.0))
    );
    assert_eq!(
        eval_count_surface(
            &[
                (CalcValue::number(1.0)),
                (CalcValue::logical(true)),
                (CalcValue::text(text("2"))),
                (CalcValue::text(text("not numeric"))),
            ],
            &resolver,
        ),
        Ok(CalcValue::number(3.0))
    );
    assert_eq!(
        eval_counta_surface(
            &[CalcValue::empty(), (CalcValue::text(text("")))],
            &resolver
        ),
        Ok(CalcValue::number(1.0))
    );

    let countblank_array = CalcArray::from_rows(vec![vec![
        CalcValue::text(text("a")),
        CalcValue::text(text("")),
    ]])
    .unwrap();
    assert_eq!(
        eval_countblank_surface(&[(CalcValue::array(countblank_array))], &resolver),
        Ok(CalcValue::array(
            CalcArray::from_rows(vec![vec![
                CalcValue::error(WorksheetErrorCode::Value),
                CalcValue::error(WorksheetErrorCode::Value),
            ]])
            .unwrap()
        ))
    );
    assert!(resolver.dense_calls.borrow().is_empty());
    assert!(resolver.sparse_calls.borrow().is_empty());
}

#[test]
fn aggregate_statistical_logical_and_text_representatives_use_sparse_structured_refs() {
    let amounts = "treecalc-table://Revenue[#Data][Amount]";
    let flags = "treecalc-table://Revenue[#Data][Active]";
    let labels = "treecalc-table://Revenue[#Data][Region]";

    let mut sparse_values_by_reference = BTreeMap::new();
    sparse_values_by_reference.insert(
        structured_key(amounts),
        sparse_values(
            3,
            1,
            vec![
                (1, 1, CalcValue::number(2.0)),
                (2, 1, CalcValue::text(text("ignored"))),
                (3, 1, CalcValue::number(4.0)),
            ],
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(flags),
        sparse_values(
            2,
            1,
            vec![
                (1, 1, CalcValue::logical(true)),
                (2, 1, CalcValue::logical(true)),
            ],
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(labels),
        sparse_values(
            1,
            3,
            vec![
                (1, 1, CalcValue::text(text("North"))),
                (1, 3, CalcValue::text(text("West"))),
            ],
        ),
    );
    let resolver = StructuredSparseResolver::with_values(sparse_values_by_reference);

    assert_eq!(
        eval_average_surface(&[structured_arg(amounts)], &resolver),
        Ok(CalcValue::number(3.0))
    );
    assert_eq!(
        eval_max_surface(&[structured_arg(amounts)], &resolver),
        Ok(CalcValue::number(4.0))
    );
    assert_eq!(
        eval_and_surface(&[structured_arg(flags)], &resolver),
        Ok(CalcValue::logical(true))
    );
    assert_eq!(
        eval_textjoin_surface(
            &[
                scalar_text("|"),
                scalar_logical(true),
                structured_arg(labels)
            ],
            &resolver,
        ),
        Ok(CalcValue::text(text("North|West")))
    );

    assert!(resolver.dense_calls.borrow().is_empty());
}

#[test]
fn lookup_match_and_criteria_representatives_use_sparse_structured_refs() {
    let regions = "treecalc-table://Revenue[#Data][Region]";
    let amounts = "treecalc-table://Revenue[#Data][Amount]";
    let returns = "treecalc-table://Revenue[#Data][Return]";

    let mut sparse_values_by_reference = BTreeMap::new();
    sparse_values_by_reference.insert(
        structured_key(regions),
        sparse_values(
            1,
            3,
            vec![
                (1, 1, CalcValue::text(text("North"))),
                (1, 2, CalcValue::text(text("South"))),
                (1, 3, CalcValue::text(text("North"))),
            ],
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(amounts),
        sparse_values(
            1,
            3,
            vec![
                (1, 1, CalcValue::number(10.0)),
                (1, 2, CalcValue::number(20.0)),
                (1, 3, CalcValue::number(30.0)),
            ],
        ),
    );
    sparse_values_by_reference.insert(
        structured_key(returns),
        sparse_values(
            1,
            3,
            vec![
                (1, 1, CalcValue::number(100.0)),
                (1, 2, CalcValue::number(200.0)),
                (1, 3, CalcValue::number(300.0)),
            ],
        ),
    );
    let resolver = StructuredSparseResolver::with_values(sparse_values_by_reference);

    assert_eq!(
        eval_match_surface(
            &scalar_text("South"),
            &[structured_arg(regions)],
            Some(&scalar_number(0.0)),
            &resolver
        ),
        Ok(CalcValue::number(2.0))
    );
    assert_eq!(
        eval_xlookup_surface(
            &scalar_text("South"),
            &[structured_arg(regions)],
            &[structured_arg(returns)],
            None,
            None,
            None,
            &resolver,
        ),
        Ok(CalcValue::number(200.0))
    );
    assert_eq!(
        eval_countif_surface(&[structured_arg(regions), scalar_text("North")], &resolver),
        Ok(CalcValue::number(2.0))
    );
    assert_eq!(
        eval_sumifs_surface(
            &[
                structured_arg(amounts),
                structured_arg(regions),
                scalar_text("North"),
            ],
            &resolver,
        ),
        Ok(CalcValue::number(40.0))
    );

    assert!(resolver.dense_calls.borrow().is_empty());
}

#[test]
fn rows_columns_and_index_use_sparse_extent_without_selector_parsing() {
    let table = "treecalc-table://Revenue[#Data]";
    let mut sparse_values_by_reference = BTreeMap::new();
    sparse_values_by_reference.insert(structured_key(table), sparse_values(7, 3, Vec::new()));
    let resolver = StructuredSparseResolver::with_values(sparse_values_by_reference);

    assert_eq!(
        eval_rows_surface_with_resolver(&[structured_arg(table)], &resolver),
        Ok(CalcValue::number(7.0))
    );
    assert_eq!(
        eval_columns_surface_with_resolver(&[structured_arg(table)], &resolver),
        Ok(CalcValue::number(3.0))
    );
    assert_eq!(
        eval_index_surface(
            &[
                structured_arg(table),
                scalar_number(2.0),
                scalar_number(3.0)
            ],
            &resolver,
        ),
        Ok(CalcValue::reference(ReferenceLike::new(
            ReferenceKind::Structured,
            format!("{table}#INDEX(2,3)"),
        )))
    );
    assert!(matches!(
        eval_index_surface(
            &[
                structured_arg(table),
                scalar_number(8.0),
                scalar_number(1.0)
            ],
            &resolver,
        ),
        Err(IndexEvalError::OutOfBounds {
            rows: 7,
            cols: 3,
            row: 8,
            col: 1,
        })
    ));

    assert!(resolver.dense_calls.borrow().is_empty());
}

#[test]
fn subtotal_and_aggregate_use_sparse_values_plus_host_context() {
    let amounts = "treecalc-table://Revenue[#Data][Amount]";
    let mut sparse_values_by_reference = BTreeMap::new();
    sparse_values_by_reference.insert(
        structured_key(amounts),
        sparse_values(
            4,
            1,
            vec![
                (1, 1, CalcValue::number(10.0)),
                (2, 1, CalcValue::number(20.0)),
                (3, 1, CalcValue::number(30.0)),
                (4, 1, CalcValue::number(40.0)),
            ],
        ),
    );
    let resolver = StructuredSparseResolver::with_values(sparse_values_by_reference);
    let mut host = StructuredHostInfo::default();
    host.aggregate_contexts.insert(
        amounts.to_string(),
        AggregateReferenceContext::new(
            oxfunc_core::value::ArrayShape { rows: 4, cols: 1 },
            vec![
                AggregateCellContext {
                    row_hidden_manual: false,
                    row_filtered_out: false,
                    nested_subtotal_or_aggregate: false,
                },
                AggregateCellContext {
                    row_hidden_manual: true,
                    row_filtered_out: false,
                    nested_subtotal_or_aggregate: false,
                },
                AggregateCellContext {
                    row_hidden_manual: false,
                    row_filtered_out: true,
                    nested_subtotal_or_aggregate: false,
                },
                AggregateCellContext {
                    row_hidden_manual: false,
                    row_filtered_out: false,
                    nested_subtotal_or_aggregate: true,
                },
            ],
        )
        .expect("context shape is valid"),
    );

    assert_eq!(
        eval_subtotal_surface(
            &[scalar_number(109.0), structured_arg(amounts)],
            &resolver,
            Some(&host)
        ),
        Ok(CalcValue::number(10.0))
    );
    assert_eq!(
        eval_aggregate_surface(
            &[
                scalar_number(9.0),
                scalar_number(3.0),
                structured_arg(amounts)
            ],
            &resolver,
            Some(&host),
        ),
        Ok(CalcValue::number(10.0))
    );

    assert!(resolver.dense_calls.borrow().is_empty());
}

#[test]
fn metadata_lanes_pass_structured_references_to_host_or_count_opaque_areas() {
    let formula_cell = "treecalc-table://Revenue[#Totals][Amount]";
    let resolver = StructuredSparseResolver::with_values(BTreeMap::new());
    let host = StructuredHostInfo::default();

    assert_eq!(
        eval_areas_surface(&[structured_arg(formula_cell)]),
        Ok(CalcValue::number(1.0))
    );
    assert_eq!(
        eval_formulatext_surface(&[structured_arg(formula_cell)], Some(&host)),
        Ok(CalcValue::text(text("=SUM(Table[Amount])")))
    );
    assert_eq!(
        eval_cell_surface(
            &[scalar_text("filename"), structured_arg(formula_cell)],
            &resolver,
            Some(&host),
        ),
        Ok(CalcValue::text(text("Book.xlsx")))
    );

    assert_eq!(
        host.formula_text_calls.borrow().as_slice(),
        &[ReferenceLike::new(ReferenceKind::Structured, formula_cell)]
    );
    assert_eq!(
        host.cell_info_calls.borrow().as_slice(),
        &[ReferenceLike::new(ReferenceKind::Structured, formula_cell)]
    );
    assert!(resolver.dense_calls.borrow().is_empty());
    assert!(resolver.sparse_calls.borrow().is_empty());
}
