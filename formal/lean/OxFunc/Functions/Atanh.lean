import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAtanh [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def atanhMeta : FunctionMeta := {
  functionId := "FUNC.ATANH"
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

def evalAtanhSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n ≤ (-1 : Rat) ∨ 1 ≤ n then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

/--
W109 executable publication-route tag for the identified current-reference
ATANH graph. The tag records the load-bearing route order without duplicating
the Rust/x87 numeric backend: DAZ publication precedes the exact binary64
cubic/ratio threshold, and the ratio route has stored x87 add/sub/div nodes.
-/
inductive AtanhPublicationRoute where
  | denormalPositiveZero
  | binary64Cubic
  | storedX87Ratio
  deriving DecidableEq, Repr

def atanhPublicationRoute (denormal belowRatioThreshold : Bool) : AtanhPublicationRoute :=
  if denormal then
    .denormalPositiveZero
  else if belowRatioThreshold then
    .binary64Cubic
  else
    .storedX87Ratio

theorem evalAtanh_abs_one_is_num :
    evalAtanhSurfaceClass (.number 1) = .error .num := by
  native_decide

theorem atanhMeta_profiles :
    atanhMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ atanhMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ atanhMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [atanhMeta]

theorem atanh_publication_route_order_and_split :
    atanhPublicationRoute true false = .denormalPositiveZero
    ∧ atanhPublicationRoute false true = .binary64Cubic
    ∧ atanhPublicationRoute false false = .storedX87Ratio := by
  native_decide

end OxFunc.Functions
