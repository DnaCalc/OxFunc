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

theorem reference_domain_only_reference :
    boundaryAllows .referenceDomain .referenceLike = true ∧
    boundaryAllows .referenceDomain .number = false ∧
    boundaryAllows .referenceDomain .array = false := by
  simp [boundaryAllows]

end OxFunc
