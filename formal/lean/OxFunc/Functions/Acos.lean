import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def acosMeta : FunctionMeta := {
  functionId := "FUNC.ACOS"
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

def evalAcosSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if decide ((-1 : Rat) ≤ n ∧ n ≤ (1 : Rat)) then .ok "number" else .error .num
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAcos_domain_error :
    evalAcosSurfaceClass (.number 2) = .error .num := by
  have h1 : (-1 : Rat) ≤ 2 := by decide
  have h2 : ¬ (2 : Rat) ≤ 1 := by decide
  simp [evalAcosSurfaceClass, coerceToNumber, h1, h2]

theorem acosMeta_profiles :
    acosMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ acosMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [acosMeta]

end OxFunc.Functions
