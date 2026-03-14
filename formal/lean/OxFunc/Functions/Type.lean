import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

inductive TypeSeedInput where
  | number
  | text
  | logical
  | err
  | array
  | reference
  | lambdaValue
  | missingArg
  | emptyCell
  deriving DecidableEq, Repr

def typeMeta : FunctionMeta := {
  functionId := "FUNC.TYPE"
  arity := Arity.exact 1
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

def evalTypeCode : TypeSeedInput → Nat
  | .number => 1
  | .text => 2
  | .logical => 4
  | .err => 16
  | .array => 64
  | .reference => 16
  | .lambdaValue => 64
  | .missingArg => 1
  | .emptyCell => 1

theorem evalType_blank_single_cell_reference_seed :
    evalTypeCode .emptyCell = 1 := by
  rfl

theorem evalType_array_seed :
    evalTypeCode .array = 64 := by
  rfl

theorem evalType_text_seed :
    evalTypeCode .text = 2 := by
  rfl

theorem typeMeta_profiles :
    typeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ typeMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ typeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [typeMeta]

end OxFunc.Functions
