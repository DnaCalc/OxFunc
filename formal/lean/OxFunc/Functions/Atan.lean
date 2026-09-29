import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def atanMeta : FunctionMeta := {
  functionId := "FUNC.ATAN"
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

def evalAtanSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAtan_numeric_text_admitted :
    evalAtanSurfaceClass (.text "1") = .ok "number" := by
  have parsed : parseSimpleNumber "1" = some 1 := by native_decide
  simp [evalAtanSurfaceClass, coerceToNumber, parsed]

theorem atanMeta_profiles :
    atanMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ atanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [atanMeta]

/-- W111's executable operation graph has two explicitly bound numerical
substrates. `direct` is the public FPATAN(x,1) operation. `inverseDifference`
accepts a positive magnitude and retains both FPATAN(1,magnitude) and the
extended pi/2 constant through subtraction, publishing binary64 only at the end.
Neither Float.atan nor a binary64 pi constant is asserted to model that substrate.
The primitive alignment is empirical and remains an open formal/platform lane. -/
def atanWithSubstrates (direct inverseDifference : Float → Float) (x : Float) : Float :=
  if x.abs ≤ 1 then direct x
  else
    let result := inverseDifference x.abs
    if x < 0 then -result else result

theorem atan_direct_branch_binding (direct inverseDifference : Float → Float)
    (x : Float) (h : x.abs ≤ 1) :
    atanWithSubstrates direct inverseDifference x = direct x := by
  simp [atanWithSubstrates, h]

theorem atan_inverse_branch_binding (direct inverseDifference : Float → Float)
    (x : Float) (h : ¬ x.abs ≤ 1) :
    atanWithSubstrates direct inverseDifference x =
      (if x < 0 then -(inverseDifference x.abs) else inverseDifference x.abs) := by
  simp [atanWithSubstrates, h]

theorem atan_unit_boundary_uses_direct :
    (atanWithSubstrates (fun _ => 7) (fun _ => 9) 1).toBits = (7 : Float).toBits := by
  native_decide

theorem atan_negative_inverse_sign_binding :
    (atanWithSubstrates (fun _ => 7) (fun _ => 9) (-2)).toBits = (-9 : Float).toBits := by
  native_decide

end OxFunc.Functions
