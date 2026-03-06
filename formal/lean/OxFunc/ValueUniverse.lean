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
  | referenceLike
  | missingArg
  | emptyCell
  | lambdaValue
  | extendedWrapper
  | nullLike
  deriving DecidableEq, Repr

inductive ValueBoundary where
  | cellContent
  | evalResult
  | callArg
  | referenceDomain
  | extendedDomain
  deriving DecidableEq, Repr

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
  | .evalResult, .number => true
  | .evalResult, .text => true
  | .evalResult, .logical => true
  | .evalResult, .error => true
  | .evalResult, .array => true
  | .evalResult, .referenceLike => true
  | .evalResult, .lambdaValue => true
  | .callArg, .number => true
  | .callArg, .text => true
  | .callArg, .logical => true
  | .callArg, .error => true
  | .callArg, .array => true
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
  | .extendedDomain, .referenceLike => true
  | .extendedDomain, .lambdaValue => true
  | .extendedDomain, .extendedWrapper => true
  | _, _ => false

theorem eval_disallows_missing_empty_null :
    boundaryAllows .evalResult .missingArg = false ∧
    boundaryAllows .evalResult .emptyCell = false ∧
    boundaryAllows .evalResult .nullLike = false := by
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

end OxFunc
