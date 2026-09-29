import OxFunc.NumericPublication

namespace OxFunc.AggregatePublication

/-- Numeric HARMEAN slice after aggregate preparation. Staged primitives are
empirical bindings, not an assertion of universal hardware equivalence. -/
def harmean (add divide : Float → Float → Float) (reciprocal : Float → Float)
    (values : List Float) : Except WorksheetErrorCode Float :=
  if values.isEmpty then .error .na
  else if values.any (fun x => x ≤ 0) then .error .num
  else
    let total := values.foldl (fun acc x => add acc (reciprocal x)) 0
    if !total.isFinite then .error .num
    else
      let result := reciprocal (divide total values.length.toFloat)
      if result.isFinite then .ok result else .error .num

/-- DEVSQ preserves left-to-right binary64 accumulation but publishes each
staged square before adding it. Prepared coercion remains a separate layer. -/
def devsq (multiply : Float → Float → Float) (values : List Float) :
    Except WorksheetErrorCode Float :=
  if values.isEmpty then .error .num
  else
    let mean := values.foldl (· + ·) 0 / values.length.toFloat
    let total := values.foldl (fun acc x =>
      let delta := x - mean
      acc + NumericPublication.flushTiny (multiply delta delta)) 0
    if total.isFinite then .ok total else .error .num

theorem harmean_zero_rejected :
    (harmean (· + ·) (· / ·) (fun x => 1 / x) [1, 0]).toOption = none := by
  native_decide

theorem harmean_intermediate_overflow_rejected :
    (harmean (fun _ _ => Float.ofBits 0x7ff0000000000000)
      (· / ·) (fun x => 1 / x) [1, 2]).toOption = none := by native_decide

theorem devsq_tiny_square_before_sum :
    ((devsq (fun _ _ => Float.ofBits 0x000fffffffffffff)
      [1, 2, 3]).toOption.map Float.toBits) = some 0 := by native_decide

theorem devsq_overflow_rejected :
    (devsq (fun _ _ => Float.ofBits 0x7ff0000000000000)
      [1, 2]).toOption = none := by native_decide

/-- Inputs after array/reference expansion. Numeric text parsing is supplied
by the existing shared coercion context; this model does not replace it. -/
inductive PreparedInput where
  | number (value : Float)
  | logical (value : Bool)
  | text (value : String)
  | empty | missing | ignored
  | error (code : WorksheetErrorCode)

def prepareItem (parseText : String → Except WorksheetErrorCode Float)
    (direct : Bool) (input : PreparedInput) : Except WorksheetErrorCode (Option Float) :=
  match input with
  | .number value => .ok (some value)
  | .error code => .error code
  | .empty | .ignored => .ok none
  | .missing => .ok (if direct then some 0 else none)
  | .logical value => .ok (if direct then some (if value then 1 else 0) else none)
  | .text value => if direct then (parseText value).map some else .ok none

/-- Left-to-right collection resolves coercion/error outcomes before any
aggregate numeric-domain validation. Blank and omitted scalar are distinct. -/
def collectPreparedInOrder (parseText : String → Except WorksheetErrorCode Float) :
    List (Bool × PreparedInput) → Except WorksheetErrorCode (List Float)
  | [] => .ok []
  | (direct, input) :: tail => do
      let value ← prepareItem parseText direct input
      let rest ← collectPreparedInOrder parseText tail
      pure (match value with | none => rest | some n => n :: rest)

/-- Direct scalar coercion errors are checked before collection-origin errors.
The subsequent full pass preserves the original numeric accumulation order. -/
def validateDirect (parseText : String → Except WorksheetErrorCode Float) :
    List (Bool × PreparedInput) → Except WorksheetErrorCode Unit
  | [] => .ok ()
  | (direct, input) :: tail => do
      if direct then
        let _ ← prepareItem parseText true input
        pure ()
      validateDirect parseText tail

def collectPrepared (parseText : String → Except WorksheetErrorCode Float)
    (inputs : List (Bool × PreparedInput)) : Except WorksheetErrorCode (List Float) := do
  validateDirect parseText inputs
  collectPreparedInOrder parseText inputs

def harmeanPrepared (parseText : String → Except WorksheetErrorCode Float)
    (add divide : Float → Float → Float) (reciprocal : Float → Float)
    (inputs : List (Bool × PreparedInput)) : Except WorksheetErrorCode Float := do
  let values ← collectPrepared parseText inputs
  harmean add divide reciprocal values

def devsqPrepared (parseText : String → Except WorksheetErrorCode Float)
    (multiply : Float → Float → Float) (inputs : List (Bool × PreparedInput)) :
    Except WorksheetErrorCode Float := do
  let values ← collectPrepared parseText inputs
  devsq multiply values

private def errorCode (result : Except WorksheetErrorCode Float) : Option WorksheetErrorCode :=
  match result with | .error code => some code | .ok _ => none

theorem prepared_empty_missing_origin_distinction :
    ((collectPrepared (fun _ => .error .value)
      [(true, .empty), (true, .missing), (false, .missing),
       (false, .logical true), (false, .text "2"), (true, .number 8)]).toOption.map
      (List.map Float.toBits)) = some [0, 0x4020000000000000] := by native_decide

theorem harmean_prepared_error_before_domain :
    errorCode (harmeanPrepared (fun _ => .error .value) (· + ·) (· / ·) (fun x => 1/x)
      [(true, .number 0), (true, .error .div0)]) = some .div0 ∧
    errorCode (harmeanPrepared (fun _ => .error .value) (· + ·) (· / ·) (fun x => 1/x)
      [(true, .number (-1)), (true, .text "x")]) = some .value := by native_decide

theorem prepared_no_values_publication :
    errorCode (harmeanPrepared (fun _ => .error .value) (· + ·) (· / ·) (fun x => 1/x)
      [(true, .empty), (false, .text "x")]) = some .na ∧
    errorCode (devsqPrepared (fun _ => .error .value) (· * ·)
      [(true, .empty), (false, .text "x")]) = some .num := by native_decide

theorem direct_scalar_errors_precede_collection_errors :
    errorCode (harmeanPrepared (fun _ => .error .value) (· + ·) (· / ·) (fun x => 1/x)
      [(false, .error .div0), (true, .error .num)]) = some .num ∧
    errorCode (devsqPrepared (fun _ => .error .value) (· * ·)
      [(false, .error .num), (true, .text "x")]) = some .value := by native_decide

theorem precheck_preserves_numeric_collection_order :
    ((collectPrepared (fun _ => .error .value)
      [(false, .number 3), (true, .number 1), (false, .number 2)]).toOption.map
      (List.map Float.toBits)) = some [0x4008000000000000, 0x3ff0000000000000,
        0x4000000000000000] := by native_decide

end OxFunc.AggregatePublication
