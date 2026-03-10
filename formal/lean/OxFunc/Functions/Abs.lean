import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.FloatingPointEnv

namespace OxFunc.Functions

open OxFunc

def absMeta : FunctionMeta := {
  functionId := "FUNC.ABS"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.numToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def absKernel (n : Rat) : Rat :=
  if n < 0 then -n else n

def evalAbsAdapterArg (arg : CoercionInput) : Except CoercionError Rat :=
  match coerceToNumber arg with
  | Except.ok n => Except.ok (absKernel n)
  | Except.error e => Except.error e

def evalAbsAdapterScalar (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  match args with
  | [arg] =>
      match evalAbsAdapterArg arg with
      | Except.ok n => Except.ok (Value.number n)
      | Except.error e => Except.error (Sum.inr e)
  | _ => Except.error (Sum.inl (EvalError.arityMismatch 1 args.length))

def evalAbsAdapterLift (args : List CoercionInput) : List (Except CoercionError Rat) :=
  args.map evalAbsAdapterArg

inductive AbsFloatBoundaryCase where
  | formulaNegZeroLiteral
  | directTinyNegativeReciprocal
  | referenceTinyNegativeReciprocal
  deriving DecidableEq, Repr

def absFloatObservationEvidenceId : String :=
  "W5-ABS-FP-20260310"

def absObservedFloatBinding : AbsFloatBoundaryCase → FloatObservationBinding
  | .formulaNegZeroLiteral => {
      layer := FloatEnvironmentLayer.worksheetFormulaSurface
      outcome := WorksheetFloatOutcome.visibleZero
      evidenceId := absFloatObservationEvidenceId
      note := "ABS(-0) is worksheet-visible as zero in the build-scoped Excel baseline."
    }
  | .directTinyNegativeReciprocal => {
      layer := FloatEnvironmentLayer.worksheetFormulaSurface
      outcome := WorksheetFloatOutcome.div0Error
      evidenceId := absFloatObservationEvidenceId
      note := "ABS of a tiny negative candidate collapses to worksheet-visible zero strongly enough that a reciprocal yields #DIV/0!."
    }
  | .referenceTinyNegativeReciprocal => {
      layer := FloatEnvironmentLayer.worksheetFormulaSurface
      outcome := WorksheetFloatOutcome.div0Error
      evidenceId := absFloatObservationEvidenceId
      note := "Reference-fed ABS of a tiny negative candidate also collapses to worksheet-visible zero strongly enough that a reciprocal yields #DIV/0!."
    }

theorem absKernel_of_neg (n : Rat) (h : n < 0) :
    absKernel n = -n := by
  simp [absKernel, h]

theorem absKernel_of_nonneg (n : Rat) (h : ¬ n < 0) :
    absKernel n = n := by
  simp [absKernel, h]

theorem absObservedFloat_formulaNegZeroLiteral :
    absObservedFloatBinding .formulaNegZeroLiteral =
      { layer := FloatEnvironmentLayer.worksheetFormulaSurface
        outcome := WorksheetFloatOutcome.visibleZero
        evidenceId := absFloatObservationEvidenceId
        note := "ABS(-0) is worksheet-visible as zero in the build-scoped Excel baseline." } := by
  rfl

theorem absObservedFloat_directTinyNegativeReciprocal :
    absObservedFloatBinding .directTinyNegativeReciprocal =
      { layer := FloatEnvironmentLayer.worksheetFormulaSurface
        outcome := WorksheetFloatOutcome.div0Error
        evidenceId := absFloatObservationEvidenceId
        note := "ABS of a tiny negative candidate collapses to worksheet-visible zero strongly enough that a reciprocal yields #DIV/0!." } := by
  rfl

theorem absObservedFloat_referenceTinyNegativeReciprocal :
    absObservedFloatBinding .referenceTinyNegativeReciprocal =
      { layer := FloatEnvironmentLayer.worksheetFormulaSurface
        outcome := WorksheetFloatOutcome.div0Error
        evidenceId := absFloatObservationEvidenceId
        note := "Reference-fed ABS of a tiny negative candidate also collapses to worksheet-visible zero strongly enough that a reciprocal yields #DIV/0!." } := by
  rfl

theorem evalAbsScalar_rejects_nil :
    evalAbsAdapterScalar [] = Except.error (Sum.inl (EvalError.arityMismatch 1 0)) := by
  simp [evalAbsAdapterScalar]

theorem evalAbsScalar_rejects_two (a b : CoercionInput) :
    evalAbsAdapterScalar [a, b] = Except.error (Sum.inl (EvalError.arityMismatch 1 2)) := by
  simp [evalAbsAdapterScalar]

theorem evalAbsScalar_admitted_number_neg :
    evalAbsAdapterScalar [CoercionInput.number (-3)] =
      Except.ok (Value.number (absKernel (-3))) := by
  have h : (-3 : Rat) < 0 := by decide
  simp [evalAbsAdapterScalar, evalAbsAdapterArg, coerceToNumber, absKernel, h]

theorem evalAbsScalar_logical_true :
    evalAbsAdapterScalar [CoercionInput.logical true] = Except.ok (Value.number 1) := by
  have h : ¬ ((1 : Rat) < 0) := by decide
  simp [evalAbsAdapterScalar, evalAbsAdapterArg, coerceToNumber, absKernel, h]

theorem evalAbsScalar_text_bad :
    evalAbsAdapterScalar [CoercionInput.text "asd"] =
      Except.error (Sum.inr (CoercionError.nonNumericText "asd")) := by
  simp [evalAbsAdapterScalar, evalAbsAdapterArg, coerceToNumber, parseSimpleNumber]

theorem evalAbsLift_length (args : List CoercionInput) :
    (evalAbsAdapterLift args).length = args.length := by
  simp [evalAbsAdapterLift]

theorem evalAbsScalar_deterministic (args : List CoercionInput) :
    evalAbsAdapterScalar args = evalAbsAdapterScalar args := rfl

theorem absMeta_values_only_preparation :
    absMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := rfl

theorem absMeta_adapter_fec_none :
    absMeta.fecDependencyProfile = FecDependencyProfile.none := rfl

theorem absMeta_surface_fec_ref_only :
    absMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := rfl

theorem absMeta_kernel_signature_num_to_num :
    absMeta.kernelSignatureClass = KernelSignatureClass.numToNum := rfl

theorem absMeta_coercion_profile_unary_numeric_scalar_or_array :
    absMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise := rfl

end OxFunc.Functions
