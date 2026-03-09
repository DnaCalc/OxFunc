import OxFunc.FunctionCore
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def indexMeta : FunctionMeta := {
  functionId := "FUNC.INDEX"
  arity := { min := 2, max := 4 }
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

inductive IndexSource where
  | array (rows cols : Nat)
  | reference (ref : ReferenceToken)
  deriving DecidableEq, Repr

inductive IndexResult where
  | array (rows cols : Nat)
  | reference (ref : ReferenceToken)
  | payloadUnavailable
  deriving DecidableEq, Repr

def evalIndexAdapter
    (source : IndexSource)
    (row col : Nat)
    (area : Nat := 1) : Except String IndexResult :=
  if area ≠ 1 then
    Except.error "invalid_area"
  else
    match source with
    | .reference ref =>
        if row = 0 && col = 0 then
          Except.ok (.reference ref)
        else
          Except.ok (.reference {
            kind := ref.kind
            target := ref.target ++ "#INDEX(" ++ toString row ++ "," ++ toString col ++ ")"
          })
    | .array rows cols =>
        if row = 0 || col = 0 then
          Except.ok (.array rows cols)
        else if row ≤ rows && col ≤ cols then
          Except.ok .payloadUnavailable
        else
          Except.error "out_of_bounds"

theorem evalIndexAdapter_deterministic
    (source : IndexSource) (row col area : Nat) :
    evalIndexAdapter source row col area = evalIndexAdapter source row col area := rfl

theorem evalIndexAdapter_array_payload_unavailable :
    evalIndexAdapter (.array 3 2) 1 1 1 = Except.ok .payloadUnavailable := by
  simp [evalIndexAdapter]

theorem indexMeta_profiles :
    indexMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ indexMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ indexMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [indexMeta]

end OxFunc.Functions
