import OxFunc.RefResolverSeam
import OxFunc.Functions.Abs

namespace OxFunc.Functions

open OxFunc

inductive AbsSurfaceArg where
  | prepared (v : CoercionInput)
  | reference (ref : ReferenceToken)
  deriving DecidableEq, Repr

inductive AbsSurfaceCoercionError where
  | refResolution (e : RefResolutionError)
  | coercion (e : CoercionError)
  deriving DecidableEq, Repr

inductive AbsSurfaceError where
  | arityMismatch (expected actual : Nat)
  | coercion (e : AbsSurfaceCoercionError)
  deriving DecidableEq, Repr

inductive AbsSurfaceLiftOutcome where
  | number (n : Rat)
  | error (e : AbsSurfaceCoercionError)
  deriving DecidableEq, Repr

def prepareAbsSurfaceArgValuesOnly (resolver : ReferenceResolver) (arg : AbsSurfaceArg) :
    Except AbsSurfaceCoercionError CoercionInput :=
  match arg with
  | .prepared v => Except.ok v
  | .reference ref =>
      match resolveRefToInput resolver ref with
      | Except.ok v => Except.ok v
      | Except.error e => Except.error (AbsSurfaceCoercionError.refResolution e)

def evalAbsSurfaceScalar (resolver : ReferenceResolver) (args : List AbsSurfaceArg) :
    Except AbsSurfaceError Rat :=
  match args with
  | [arg] =>
      match prepareAbsSurfaceArgValuesOnly resolver arg with
      | Except.error e => Except.error (AbsSurfaceError.coercion e)
      | Except.ok prepared =>
          match evalAbsAdapterArg prepared with
          | Except.ok n => Except.ok n
          | Except.error e => Except.error (AbsSurfaceError.coercion (.coercion e))
  | _ => Except.error (AbsSurfaceError.arityMismatch 1 args.length)

def evalAbsSurfaceScalarValue (resolver : ReferenceResolver) (args : List AbsSurfaceArg) :
    Except AbsSurfaceError Value := do
  let n ← evalAbsSurfaceScalar resolver args
  pure (Value.number n)

def evalAbsSurfaceLift (resolver : ReferenceResolver) (args : List AbsSurfaceArg) :
    List AbsSurfaceLiftOutcome :=
  args.map (fun arg =>
    match prepareAbsSurfaceArgValuesOnly resolver arg with
    | Except.error e => AbsSurfaceLiftOutcome.error e
    | Except.ok prepared =>
        match evalAbsAdapterArg prepared with
        | Except.ok n => AbsSurfaceLiftOutcome.number n
        | Except.error ce => AbsSurfaceLiftOutcome.error (.coercion ce))

theorem prepareAbsSurfaceArgValuesOnly_prepared_passthrough
    (resolver : ReferenceResolver) (v : CoercionInput) :
    prepareAbsSurfaceArgValuesOnly resolver (.prepared v) = Except.ok v := by
  simp [prepareAbsSurfaceArgValuesOnly]

theorem evalAbsSurfaceScalar_rejects_nil (resolver : ReferenceResolver) :
    evalAbsSurfaceScalar resolver [] = Except.error (AbsSurfaceError.arityMismatch 1 0) := by
  simp [evalAbsSurfaceScalar]

theorem evalAbsSurfaceScalar_prepared_number_neg (resolver : ReferenceResolver) :
    evalAbsSurfaceScalar resolver [.prepared (.number (-3))] = Except.ok (absKernel (-3)) := by
  have h : (-3 : Rat) < 0 := by decide
  simp [evalAbsSurfaceScalar, prepareAbsSurfaceArgValuesOnly, evalAbsAdapterArg, coerceToNumber, absKernel, h]

theorem evalAbsSurfaceScalar_prepared_logical_true (resolver : ReferenceResolver) :
    evalAbsSurfaceScalar resolver [.prepared (.logical true)] = Except.ok 1 := by
  have h : ¬ ((1 : Rat) < 0) := by decide
  simp [evalAbsSurfaceScalar, prepareAbsSurfaceArgValuesOnly, evalAbsAdapterArg, coerceToNumber, absKernel, h]

theorem evalAbsSurfaceScalar_prepared_text_bad (resolver : ReferenceResolver) :
    evalAbsSurfaceScalar resolver [.prepared (.text "asd")] =
      Except.error (AbsSurfaceError.coercion (.coercion (.nonNumericText "asd"))) := by
  simp [evalAbsSurfaceScalar, prepareAbsSurfaceArgValuesOnly, evalAbsAdapterArg, coerceToNumber, parseSimpleNumber]

theorem evalAbsSurfaceLift_length (resolver : ReferenceResolver) (args : List AbsSurfaceArg) :
    (evalAbsSurfaceLift resolver args).length = args.length := by
  simp [evalAbsSurfaceLift]

theorem evalAbsSurfacePrepared_matches_adapter_arg
    (resolver : ReferenceResolver) (arg : CoercionInput) :
    evalAbsSurfaceScalar resolver [.prepared arg] =
      match evalAbsAdapterArg arg with
      | Except.ok n => Except.ok n
      | Except.error ce => Except.error (AbsSurfaceError.coercion (.coercion ce)) := by
  simp [evalAbsSurfaceScalar, prepareAbsSurfaceArgValuesOnly]

theorem evalAbsSurfaceScalar_deterministic
    (resolver : ReferenceResolver) (args : List AbsSurfaceArg) :
    evalAbsSurfaceScalar resolver args = evalAbsSurfaceScalar resolver args := rfl

theorem evalAbsSurfaceFromRef_deterministic
    (resolver : ReferenceResolver) (ref : ReferenceToken) :
    evalAbsSurfaceScalar resolver [.reference ref] =
      evalAbsSurfaceScalar resolver [.reference ref] := rfl

end OxFunc.Functions
