import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

instance instDecidableEqExceptSpillRef [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def spillRefMeta : FunctionMeta := {
  functionId := "FUNC.OP_SPILL_REF"
  arity := { min := 1, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalSpillRef (ref : ReferenceToken) : Except String ReferenceToken :=
  if ref.target.isEmpty then
    Except.error "invalid_anchor"
  else if ref.target.endsWith "#" then
    Except.ok { kind := .spillAnchor, target := ref.target }
  else
    Except.ok { kind := .spillAnchor, target := ref.target ++ "#" }

theorem evalSpillRef_a1_adds_hash :
    evalSpillRef { kind := .a1, target := "B2" } =
      Except.ok { kind := .spillAnchor, target := "B2#" } := by
  native_decide

theorem evalSpillRef_existing_spill_anchor_stable :
    evalSpillRef { kind := .spillAnchor, target := "B2#" } =
      Except.ok { kind := .spillAnchor, target := "B2#" } := by
  native_decide

theorem spillRefMeta_profiles :
    spillRefMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ spillRefMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ spillRefMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [spillRefMeta]

end OxFunc.Functions
