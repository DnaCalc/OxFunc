import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

inductive NSeedResult where
  | number (n : Rat)
  | error (code : WorksheetErrorCode)
  deriving DecidableEq, Repr

def nMeta : FunctionMeta := {
  functionId := "FUNC.N"
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

def evalNScalar : CoercionInput → NSeedResult
  | .number n => .number n
  | .logical true => .number 1
  | .logical false => .number 0
  | .text _ => .number 0
  | .error code => .error code
  | .missingArg => .number 0
  | .emptyCell => .number 0

def evalNArray : List CoercionInput → List NSeedResult :=
  List.map evalNScalar

theorem evalN_logical_true :
    evalNScalar (.logical true) = .number 1 := by
  rfl

theorem evalN_blank_scalar_zero :
    evalNScalar .emptyCell = .number 0 := by
  rfl

theorem evalN_array_text_zero_seed :
    evalNArray [.number 1, .text "x"] = [.number 1, .number 0] := by
  rfl

theorem nMeta_profiles :
    nMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ nMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ nMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [nMeta]

end OxFunc.Functions
