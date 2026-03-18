import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def acoshMeta : FunctionMeta := {
  functionId := "FUNC.ACOSH"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalAcoshSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if decide ((1 : Rat) ≤ n) then .ok "number" else .error .num
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAcosh_domain_error :
    evalAcoshSurfaceClass (.number 0) = .error .num := by
  have h : ¬ (1 : Rat) ≤ 0 := by decide
  simp [evalAcoshSurfaceClass, coerceToNumber, h]

theorem acoshMeta_profiles :
    acoshMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ acoshMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [acoshMeta]

end OxFunc.Functions
