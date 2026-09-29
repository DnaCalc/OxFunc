import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Asin

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

/-- W111 ACOS preserves the characterized ASIN result, then subtracts it from
binary64 PI/2 with the explicit staged subtraction primitive. -/
def acosWithKernels (sub : Float → Float → Float)
    (asin : Float → Except WorksheetErrorCode Float) (x : Float) : Except WorksheetErrorCode Float :=
  match asin x with
  | .ok angle => .ok (sub (Float.ofBits 0x3ff921fb54442d18) angle)
  | .error code => .error code

theorem acos_graph_zero_binding :
    ((acosWithKernels Float.sub (fun _ => .ok 0) 0).toOption.map Float.toBits)
      = some 0x3ff921fb54442d18 := by
  native_decide

theorem acos_graph_error_binding :
    (acosWithKernels Float.sub (fun _ => .error .num) 2).isOk = false := by
  native_decide

theorem acos_graph_half_binding :
    ((acosWithKernels Float.sub
      (asinWithKernels Float.sub Float.mul Float.div Float.sqrt Float.atan) 0.5).toOption.map Float.toBits)
      = some 0x3ff0c152382d7365 := by
  native_decide

end OxFunc.Functions
