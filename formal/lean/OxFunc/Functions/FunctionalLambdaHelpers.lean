import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def lambdaHelperMetaBase : FunctionMeta := {
  functionId := "FUNC.LAMBDA_HELPER_BASE"
  arity := { min := 1, max := 3 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def isomittedMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.ISOMITTED"
    arity := Arity.exact 1
}

def mapMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.MAP"
    arity := { min := 2, max := 1024 }
}

def reduceMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.REDUCE"
    arity := Arity.exact 3
}

def scanMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.SCAN"
    arity := Arity.exact 3
}

def byrowMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.BYROW"
    arity := Arity.exact 2
}

def bycolMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.BYCOL"
    arity := Arity.exact 2
}

def makearrayMeta : FunctionMeta := {
  lambdaHelperMetaBase with
    functionId := "FUNC.MAKEARRAY"
    arity := Arity.exact 3
}

inductive CallableOriginKind where
  | helperLambda
  | definedNameCallable
  deriving DecidableEq, Repr

inductive CallableCaptureMode where
  | noCapture
  | lexicalCapture
  deriving DecidableEq, Repr

structure CallableArityShape where
  min : Nat
  max : Nat
  deriving DecidableEq, Repr

def CallableArityShape.exact (n : Nat) : CallableArityShape := { min := n, max := n }

def CallableArityShape.accepts (arity : CallableArityShape) (argc : Nat) : Bool :=
  arity.min <= argc && argc <= arity.max

inductive SeedCallableToken where
  | add1
  | sum2
  | mul2
  | sumArray
  | nonScalarPlus1
  | makearrayCoords
  | capAdd
  deriving DecidableEq, Repr

inductive HelperCell where
  | number (n : Int)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  deriving DecidableEq, Repr

structure HelperArray where
  rows : Nat
  cols : Nat
  payload : List HelperCell
  deriving DecidableEq, Repr

inductive HelperValue where
  | scalar (cell : HelperCell)
  | array (arr : HelperArray)
  | missingArg
  | lambda
      (token : SeedCallableToken)
      (origin : CallableOriginKind)
      (arity : CallableArityShape)
      (capture : CallableCaptureMode)
  deriving DecidableEq, Repr

def helperLambda
    (token : SeedCallableToken)
    (arity : CallableArityShape)
    (capture : CallableCaptureMode) : HelperValue :=
  .lambda token .helperLambda arity capture

def definedNameCallable
    (token : SeedCallableToken)
    (arity : CallableArityShape)
    (capture : CallableCaptureMode) : HelperValue :=
  .lambda token .definedNameCallable arity capture

def rowVector (cells : List HelperCell) : HelperValue :=
  .array { rows := 1, cols := cells.length, payload := cells }

def colVector (cells : List HelperCell) : HelperValue :=
  .array { rows := cells.length, cols := 1, payload := cells }

def matrix (rows cols : Nat) (cells : List HelperCell) : HelperValue :=
  .array { rows := rows, cols := cols, payload := cells }

def publishHelperValue : HelperValue → HelperValue
  | .lambda .. => .scalar (.error .calc)
  | other => other

def evalIsOmitted : HelperValue → HelperValue
  | .missingArg => .scalar (.logical true)
  | _ => .scalar (.logical false)

private def helperListGetD (xs : List α) (idx : Nat) (fallback : α) : α :=
  match xs.drop idx with
  | [] => fallback
  | x :: _ => x

def cellToValue : HelperCell → HelperValue
  | .number n => .scalar (.number n)
  | .logical b => .scalar (.logical b)
  | .error code => .scalar (.error code)

def valueToCell? : HelperValue → Option HelperCell
  | .scalar cell => some cell
  | _ => none

def scalarNumber? : HelperValue → Option Int
  | .scalar (.number n) => some n
  | _ => none

def scalarError? : HelperValue → Option WorksheetErrorCode
  | .scalar (.error code) => some code
  | _ => none

def helperArrayPayloadSum : List HelperCell → Option Int
  | [] => some 0
  | .number n :: rest =>
      match helperArrayPayloadSum rest with
      | some tail => some (n + tail)
      | none => none
  | _ => none

def helperArrayPlusOne (arr : HelperArray) : Option HelperValue :=
  let mapped := arr.payload.map (fun cell =>
    match cell with
    | .number n => some (.number (n + 1))
    | .error code => some (.error code)
    | _ => none)
  if mapped.all Option.isSome then
    some (.array { rows := arr.rows, cols := arr.cols, payload := mapped.map (fun x => x.getD (.error .value)) })
  else
    none

def materializeHelperInput : HelperValue → List HelperValue
  | .array arr => arr.payload.map cellToValue
  | other => [other]

def firstArrayShape? : List HelperValue → Option (Nat × Nat × Nat)
  | [] => none
  | .array arr :: _ => some (arr.rows, arr.cols, arr.payload.length)
  | _ :: rest => firstArrayShape? rest

def buildMapArgs (inputs : List (List HelperValue)) (idx : Nat) : List HelperValue :=
  inputs.map (fun cells => helperListGetD cells idx (.scalar (.error .na)))

def natToInt (n : Nat) : Int :=
  Int.ofNat n

def invokeSeedCallable (callable : HelperValue) (args : List HelperValue) : HelperValue :=
  match callable with
  | .lambda token _ arity _ =>
      if ¬ arity.accepts args.length then
        .scalar (.error .value)
      else
        match token, args with
        | .add1, [arg] =>
            match scalarError? arg with
            | some code => .scalar (.error code)
            | none =>
                match scalarNumber? arg with
                | some n => .scalar (.number (n + 1))
                | none => .scalar (.error .value)
        | .sum2, [a, b] =>
            match scalarError? a with
            | some code => .scalar (.error code)
            | none =>
                match scalarError? b with
                | some code => .scalar (.error code)
                | none =>
                    match scalarNumber? a, scalarNumber? b with
                    | some x, some y => .scalar (.number (x + y))
                    | _, _ => .scalar (.error .value)
        | .mul2, [a, b] =>
            match scalarError? a with
            | some code => .scalar (.error code)
            | none =>
                match scalarError? b with
                | some code => .scalar (.error code)
                | none =>
                    match scalarNumber? a, scalarNumber? b with
                    | some x, some y => .scalar (.number (x * y))
                    | _, _ => .scalar (.error .value)
        | .sumArray, [.array arr] =>
            match helperArrayPayloadSum arr.payload with
            | some total => .scalar (.number total)
            | none => .scalar (.error .value)
        | .nonScalarPlus1, [.array arr] =>
            match helperArrayPlusOne arr with
            | some value => value
            | none => .scalar (.error .value)
        | .makearrayCoords, [r, c] =>
            match scalarNumber? r, scalarNumber? c with
            | some x, some y => .scalar (.number (x * 10 + y))
            | _, _ => .scalar (.error .value)
        | .capAdd, [y] =>
            match scalarNumber? y with
            | some n => .scalar (.number (2 + n))
            | none => .scalar (.error .value)
        | _, _ => .scalar (.error .value)
  | _ => .scalar (.error .value)

def helperMapShape (inputs : List HelperValue) (cellCount : Nat) : Nat × Nat :=
  match firstArrayShape? inputs with
  | some (rows, cols, payloadCount) =>
      if payloadCount = cellCount then
        (rows, cols)
      else
        (1, cellCount)
  | none => (1, cellCount)

def evalMap (inputs : List HelperValue) (callable : HelperValue) : HelperValue :=
  let materialized := inputs.map materializeHelperInput
  let cellCount := (materialized.map List.length).foldl Nat.max 0
  let payload := (List.range cellCount).map (fun idx =>
    match valueToCell? (invokeSeedCallable callable (buildMapArgs materialized idx)) with
    | some cell => cell
    | none => .error .value)
  let shape := helperMapShape inputs cellCount
  .array { rows := shape.1, cols := shape.2, payload := payload }

def evalReduceStep (acc : HelperValue) (next : HelperValue) (callable : HelperValue) : HelperValue :=
  invokeSeedCallable callable [acc, next]

def evalReduce (initial iterable callable : HelperValue) : HelperValue :=
  (materializeHelperInput iterable).foldl (fun acc next => evalReduceStep acc next callable) initial

def evalScanFold (acc : HelperValue × List HelperCell) (next : HelperValue) (callable : HelperValue) :
    HelperValue × List HelperCell :=
  let nextAcc := evalReduceStep acc.1 next callable
  let nextCell := (valueToCell? nextAcc).getD (.error .value)
  (nextAcc, acc.2 ++ [nextCell])

def evalScan (initial iterable callable : HelperValue) : HelperValue :=
  let folded := (materializeHelperInput iterable).foldl (fun acc next => evalScanFold acc next callable) (initial, [])
  rowVector folded.2

def splitRows : HelperValue → List HelperValue
  | .array arr =>
      (List.range arr.rows).map (fun rowIdx =>
        let start := rowIdx * arr.cols
        .array { rows := 1, cols := arr.cols, payload := arr.payload.drop start |>.take arr.cols })
  | _ => [rowVector [(.error .value)]]

def splitCols : HelperValue → List HelperValue
  | .array arr =>
      (List.range arr.cols).map (fun colIdx =>
        let cells := (List.range arr.rows).map (fun rowIdx =>
          helperListGetD arr.payload (rowIdx * arr.cols + colIdx) (.error .value))
        .array { rows := arr.rows, cols := 1, payload := cells })
  | _ => [colVector [(.error .value)]]

def requireScalarHelperResult : HelperValue → HelperCell
  | .array _ => .error .calc
  | .scalar cell => cell
  | .missingArg => .error .value
  | .lambda .. => .error .value

def evalByRow (input callable : HelperValue) : HelperValue :=
  let payload := (splitRows input).map (fun rowVal =>
    requireScalarHelperResult (invokeSeedCallable callable [rowVal]))
  colVector payload

def evalByCol (input callable : HelperValue) : HelperValue :=
  let payload := (splitCols input).map (fun colVal =>
    requireScalarHelperResult (invokeSeedCallable callable [colVal]))
  rowVector payload

def evalMakeArray (rows cols : Nat) (callable : HelperValue) : HelperValue :=
  if rows = 0 || cols = 0 then
    .scalar (.error .value)
  else
    let payload :=
      (List.range (rows * cols)).map (fun idx =>
        let rowIdx := idx / cols
        let colIdx := idx % cols
        requireScalarHelperResult
          (invokeSeedCallable callable
            [.scalar (.number (natToInt (rowIdx + 1))), .scalar (.number (natToInt (colIdx + 1)))]))
    .array { rows := rows, cols := cols, payload := payload }

theorem lambdaHelperFamily_meta_profiles :
    mapMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ reduceMeta.arity = Arity.exact 3
    ∧ scanMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ byrowMeta.arity = Arity.exact 2
    ∧ bycolMeta.arity = Arity.exact 2
    ∧ makearrayMeta.arity = Arity.exact 3
    ∧ isomittedMeta.arity = Arity.exact 1 := by
  simp [lambdaHelperMetaBase, mapMeta, reduceMeta, scanMeta, byrowMeta, bycolMeta, makearrayMeta, isomittedMeta]

theorem publish_defined_name_callable_is_calc :
    publishHelperValue (definedNameCallable .add1 (CallableArityShape.exact 1) .noCapture) =
      .scalar (.error .calc) := by
  rfl

theorem isomitted_present_seed :
    evalIsOmitted (.scalar (.number 1)) = .scalar (.logical false) := by
  rfl

theorem isomitted_missing_seed :
    evalIsOmitted .missingArg = .scalar (.logical true) := by
  rfl

theorem direct_lambda_arity_mismatch_seed :
    invokeSeedCallable (helperLambda .sum2 (CallableArityShape.exact 2) .noCapture) [.scalar (.number 1)] =
      .scalar (.error .value) := by
  native_decide

theorem map_bare_spill_seed :
    evalMap [rowVector [.number 1, .number 2]]
        (helperLambda .add1 (CallableArityShape.exact 1) .noCapture) =
      rowVector [.number 2, .number 3] := by
  native_decide

theorem map_mismatch_arrays_seed :
    evalMap
        [rowVector [.number 1, .number 2], rowVector [.number 10]]
        (helperLambda .sum2 (CallableArityShape.exact 2) .noCapture) =
      rowVector [.number 11, .error .na] := by
  native_decide

theorem reduce_sum_seed :
    evalReduce (.scalar (.number 0)) (rowVector [.number 1, .number 2, .number 3])
        (helperLambda .sum2 (CallableArityShape.exact 2) .noCapture) =
      .scalar (.number 6) := by
  native_decide

theorem scan_spill_seed :
    evalScan (.scalar (.number 0)) (rowVector [.number 1, .number 2, .number 3])
        (helperLambda .sum2 (CallableArityShape.exact 2) .noCapture) =
      rowVector [.number 1, .number 3, .number 6] := by
  native_decide

theorem byrow_scalar_seed :
    evalByRow (matrix 2 2 [.number 1, .number 2, .number 3, .number 4])
        (helperLambda .sumArray (CallableArityShape.exact 1) .noCapture) =
      colVector [.number 3, .number 7] := by
  native_decide

theorem byrow_nonscalar_calc_seed :
    evalByRow (matrix 2 2 [.number 1, .number 2, .number 3, .number 4])
        (helperLambda .nonScalarPlus1 (CallableArityShape.exact 1) .noCapture) =
      colVector [.error .calc, .error .calc] := by
  native_decide

theorem bycol_scalar_seed :
    evalByCol (matrix 2 2 [.number 1, .number 2, .number 3, .number 4])
        (helperLambda .sumArray (CallableArityShape.exact 1) .noCapture) =
      rowVector [.number 4, .number 6] := by
  native_decide

theorem makearray_basic_seed :
    evalMakeArray 2 3 (helperLambda .makearrayCoords (CallableArityShape.exact 2) .noCapture) =
      matrix 2 3 [.number 11, .number 12, .number 13, .number 21, .number 22, .number 23] := by
  native_decide

theorem defined_name_direct_call_seed :
    invokeSeedCallable
        (definedNameCallable .add1 (CallableArityShape.exact 1) .noCapture)
        [.scalar (.number 4)] =
      .scalar (.number 5) := by
  native_decide

theorem defined_name_helper_call_seed :
    evalMap [rowVector [.number 1, .number 2]]
        (definedNameCallable .capAdd (CallableArityShape.exact 1) .lexicalCapture) =
      rowVector [.number 3, .number 4] := by
  native_decide

end OxFunc.Functions
