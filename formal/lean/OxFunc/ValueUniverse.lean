namespace OxFunc

inductive WorksheetErrorCode where
  | null
  | div0
  | value
  | ref
  | name
  | num
  | na
  | gettingData
  | spill
  | calc
  | field
  | blocked
  | connect
  deriving DecidableEq, Repr

inductive ValueTag where
  | number
  | text
  | logical
  | error
  | array
  | richValue
  | referenceLike
  | missingArg
  | emptyCell
  | lambdaValue
  | extendedWrapper
  | nullLike
  deriving DecidableEq, Repr

inductive ValueBoundary where
  | cellContent
  | rawFunctionReturn
  | publishedFormulaResult
  | callArg
  | referenceDomain
  | extendedDomain
  deriving DecidableEq, Repr

/- `publishedFormulaResult` models the current OxFunc result domain. It is intentionally
   narrower than the broadest raw interop/UDF return domain. `W9-XLL-NIL-20260312` showed that
   raw scalar `xltypeNil` can exist at the XLL return boundary even though it normalizes to
   numeric-zero semantics before outer argument binding and worksheet publication. Raw array nil
   elements can survive longer inside intermediate arrays. -/

abbrev Utf16CodeUnit := UInt16

def excelTextMaxUtf16CodeUnits : Nat := 32767

structure ExcelText where
  utf16CodeUnits : List Utf16CodeUnit
  deriving DecidableEq, Repr

def textWithinCellCap (text : ExcelText) : Prop :=
  text.utf16CodeUnits.length <= excelTextMaxUtf16CodeUnits

def truncateUtf16ToCellCap (units : List Utf16CodeUnit) : ExcelText :=
  { utf16CodeUnits := units.take excelTextMaxUtf16CodeUnits }

def isHighSurrogate (u : Utf16CodeUnit) : Bool :=
  (0xD800 <= u.toNat) && (u.toNat <= 0xDBFF)

def hasDanglingHighSurrogateTail (text : ExcelText) : Bool :=
  match text.utf16CodeUnits.reverse with
  | [] => false
  | u :: _ => isHighSurrogate u

def boundaryAllows : ValueBoundary → ValueTag → Bool
  | .cellContent, .number => true
  | .cellContent, .text => true
  | .cellContent, .logical => true
  | .cellContent, .error => true
  | .cellContent, .emptyCell => true
  | .rawFunctionReturn, .number => true
  | .rawFunctionReturn, .text => true
  | .rawFunctionReturn, .logical => true
  | .rawFunctionReturn, .error => true
  | .rawFunctionReturn, .array => true
  | .rawFunctionReturn, .richValue => true
  | .rawFunctionReturn, .referenceLike => true
  | .rawFunctionReturn, .emptyCell => true
  | .rawFunctionReturn, .lambdaValue => true
  | .publishedFormulaResult, .number => true
  | .publishedFormulaResult, .text => true
  | .publishedFormulaResult, .logical => true
  | .publishedFormulaResult, .error => true
  | .publishedFormulaResult, .array => true
  | .publishedFormulaResult, .richValue => true
  | .publishedFormulaResult, .referenceLike => true
  | .publishedFormulaResult, .lambdaValue => true
  | .callArg, .number => true
  | .callArg, .text => true
  | .callArg, .logical => true
  | .callArg, .error => true
  | .callArg, .array => true
  | .callArg, .richValue => true
  | .callArg, .referenceLike => true
  | .callArg, .missingArg => true
  | .callArg, .emptyCell => true
  | .callArg, .lambdaValue => true
  | .referenceDomain, .referenceLike => true
  | .extendedDomain, .number => true
  | .extendedDomain, .text => true
  | .extendedDomain, .logical => true
  | .extendedDomain, .error => true
  | .extendedDomain, .array => true
  | .extendedDomain, .richValue => true
  | .extendedDomain, .referenceLike => true
  | .extendedDomain, .lambdaValue => true
  | .extendedDomain, .extendedWrapper => true
  | _, _ => false

structure RichValueKeyFlag where
  key : String
  flag : String
  value : Bool
  deriving DecidableEq, Repr

structure RichValueType where
  typeName : String
  requiredKeys : List String
  keyFlags : List RichValueKeyFlag
  deriving DecidableEq, Repr

inductive NumberFormatHint where
  | general
  | dateLike
  | percentage
  | currency
  | scientific
  | fraction
  | custom
  deriving DecidableEq, Repr

inductive CellStyleHint where
  | hyperlink
  deriving DecidableEq, Repr

structure PresentationHint where
  numberFormat : Option NumberFormatHint
  style : Option CellStyleHint
  deriving DecidableEq, Repr

mutual
  inductive RichValueData where
    | number (n : Float)
    | text (t : ExcelText)
    | logical (b : Bool)
    | error (code : WorksheetErrorCode)
    | emptyCell
    | array (arr : RichArray)
    | richValue (value : RichValue)
    deriving Repr

  structure RichArray where
    rows : Nat
    cols : Nat
    cells : List RichValueData
    deriving Repr

  structure RichValueKeyValue where
    key : String
    value : RichValueData
    deriving Repr

  structure RichValue where
    valueType : RichValueType
    fallback : RichValueData
    kvps : List RichValueKeyValue
    deriving Repr
end

def RichArray.wellFormed (arr : RichArray) : Prop :=
  arr.rows > 0 ∧ arr.cols > 0 ∧ arr.cells.length = arr.rows * arr.cols

theorem published_formula_result_disallows_missing_empty_null :
    boundaryAllows .publishedFormulaResult .missingArg = false ∧
    boundaryAllows .publishedFormulaResult .emptyCell = false ∧
    boundaryAllows .publishedFormulaResult .nullLike = false := by
  simp [boundaryAllows]

theorem raw_function_return_allows_empty_cell_but_not_missing_or_null :
    boundaryAllows .rawFunctionReturn .emptyCell = true ∧
    boundaryAllows .rawFunctionReturn .missingArg = false ∧
    boundaryAllows .rawFunctionReturn .nullLike = false := by
  simp [boundaryAllows]

theorem truncateUtf16ToCellCap_within :
    ∀ units : List Utf16CodeUnit, textWithinCellCap (truncateUtf16ToCellCap units) := by
  intro units
  unfold textWithinCellCap truncateUtf16ToCellCap excelTextMaxUtf16CodeUnits
  simpa using List.length_take_le 32767 units

theorem reference_domain_only_reference :
    boundaryAllows .referenceDomain .referenceLike = true ∧
    boundaryAllows .referenceDomain .number = false ∧
    boundaryAllows .referenceDomain .array = false := by
  simp [boundaryAllows]

theorem published_formula_result_allows_rich_value :
    boundaryAllows .publishedFormulaResult .richValue = true ∧
    boundaryAllows .callArg .richValue = true ∧
    boundaryAllows .extendedDomain .richValue = true := by
  simp [boundaryAllows]

def hyperlinkPresentationHint : PresentationHint :=
  { numberFormat := none, style := some .hyperlink }

def todayPresentationHint : PresentationHint :=
  { numberFormat := some .dateLike, style := none }

end OxFunc
