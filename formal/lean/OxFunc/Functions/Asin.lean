import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAsin [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def asinMeta : FunctionMeta := {
  functionId := "FUNC.ASIN"
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

def asinInDomain (n : Rat) : Bool := decide ((-1 : Rat) ≤ n ∧ n ≤ (1 : Rat))

def evalAsinSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if asinInDomain n then .ok "number" else .error .num
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

def evalAsinLiftClass (inputs : List CoercionInput) : List (Except WorksheetErrorCode String) :=
  inputs.map evalAsinSurfaceClass

theorem evalAsin_numeric_text_admitted :
    evalAsinSurfaceClass (.text "1") = .ok "number" := by
  native_decide

theorem evalAsin_domain_error_num :
    evalAsinSurfaceClass (.number 2) = .error .num := by
  native_decide

theorem evalAsin_array_domain_element_errors :
    evalAsinLiftClass [.number 0, .number 2] = [.ok "number", .error .num] := by
  native_decide

theorem asinMeta_profiles :
    asinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ asinMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ asinMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ asinMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [asinMeta]

/-- W111 arithmetic graph selected by signed numeric observations. Each supplied
binary operation and square root publishes binary64 after an intermediate
64-bit-significand rounding. The second factor reuses the published `1 - x`;
replacing it with a separately computed `1 + x` changes observed negative inputs.
The arctangent backend remains explicit rather than claiming a universal proof. -/
def asinWithKernels (sub mul div : Float → Float → Float)
    (sqrt atan : Float → Float) (x : Float) : Except WorksheetErrorCode Float :=
  if !(x ≥ -1 && x ≤ 1) then .error .num
  else
    let t := sub 1 x
    let u := sub 2 t
    let product := mul t u
    let angle := atan (div x (sqrt product))
    .ok (if angle.abs < Float.ofBits 0x0010000000000000 then 0 else angle)

theorem asin_graph_domain_binding :
    (asinWithKernels Float.sub Float.mul Float.div Float.sqrt Float.atan 2).isOk = false := by
  native_decide

theorem asin_graph_zero_binding :
    ((asinWithKernels Float.sub Float.mul Float.div Float.sqrt Float.atan 0).toOption.map Float.toBits) = some 0 := by
  native_decide

theorem asin_half_graph_binding :
    ((asinWithKernels Float.sub Float.mul Float.div Float.sqrt Float.atan 0.5).toOption.map Float.toBits)
      = some 0x3fe0c152382d7366 := by
  native_decide

theorem asin_graph_subnormal_publication :
    ((asinWithKernels Float.sub Float.mul Float.div Float.sqrt
      (fun _ => Float.ofBits 1) 0.5).toOption.map Float.toBits) = some 0 := by
  native_decide

end OxFunc.Functions
