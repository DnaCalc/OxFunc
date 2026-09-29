import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAcot [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def acotMeta : FunctionMeta := {
  functionId := "FUNC.ACOT"
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

def evalAcotSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAcot_numeric_text_admitted :
    evalAcotSurfaceClass (.text "1") = .ok "number" := by
  native_decide

theorem acotMeta_profiles :
    acotMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ acotMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [acotMeta]

/-- W111 prepared numeric graph. The production bindings are binary64 publication
after a 64-bit-significand reciprocal and the characterized reciprocal-reduced FPATAN graph.
These parameters expose the arithmetic dependency, not a universal parity proof. -/
def acotWithKernels (reciprocal atan : Float → Float) (x : Float) : Float :=
  if x == 0 then Float.ofBits 0x3ff921fb54442d18
  else
    let angle := atan (reciprocal x)
    let result := if x < 0 then angle + Float.ofBits 0x400921fb54442d18 else angle
    if result.abs < Float.ofBits 0x0010000000000000 then 0 else result

theorem acot_zero_graph_binding :
    (acotWithKernels (fun _ => 0) (fun _ => 0) 0).toBits = 0x3ff921fb54442d18 := by
  native_decide

theorem acot_reciprocal_angle_graph_binding :
    (acotWithKernels (fun _ => 0.5) Float.atan 2).toBits = 0x3fddac670561bb4f ∧
    (acotWithKernels (fun _ => -0.5) Float.atan (-2)).toBits = 0x40056c6e7397f5ae := by
  native_decide

theorem acot_subnormal_result_publishes_zero :
    (acotWithKernels (fun _ => 0) (fun _ => Float.ofBits 1) 1).toBits = 0 := by
  native_decide

end OxFunc.Functions
