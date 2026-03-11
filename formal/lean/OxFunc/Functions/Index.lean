import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptIndex [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

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

structure IndexArea where
  ref : ReferenceToken
  rows : Nat
  cols : Nat
  deriving DecidableEq, Repr

inductive IndexSource where
  | array (rows cols : Nat)
  | referenceAreas (areas : List IndexArea)
  deriving DecidableEq, Repr

inductive IndexResult where
  | payloadCell (row col : Nat)
  | arraySlice (rows cols : Nat)
  | referenceSlice (ref : ReferenceToken) (rows cols : Nat)
  deriving DecidableEq, Repr

def parseIndexNatDefault (arg : Option CoercionInput) (omittedDefault blankDefault : Nat) :
    Except CoercionError Nat :=
  match arg with
  | none => Except.ok omittedDefault
  | some .missingArg | some .emptyCell => Except.ok blankDefault
  | some other =>
      match coerceToNumber other with
      | Except.ok n =>
          if n.den = 1 ∧ 0 ≤ n then
            Except.ok n.num.natAbs
          else
            Except.error (CoercionError.unsupportedKind "invalid_index")
      | Except.error e => Except.error e

def parseAreaNumDefault (arg : Option CoercionInput) : Except CoercionError Nat :=
  match arg with
  | none => Except.ok 1
  | some .missingArg | some .emptyCell => Except.ok 1
  | some other =>
      match coerceToNumber other with
      | Except.ok n =>
          if n.den = 1 ∧ 1 ≤ n then
            Except.ok n.num.natAbs
          else
            Except.error (CoercionError.unsupportedKind "invalid_area")
      | Except.error e => Except.error e

def listGetAt : List α → Nat → Option α
  | [], _ => none
  | x :: _, 0 => some x
  | _ :: xs, n + 1 => listGetAt xs n

def selectReferenceResult (area : IndexArea) (row col : Nat) : Except String IndexResult :=
  if row > area.rows || col > area.cols then
    Except.error "out_of_bounds"
  else
    let kind := if row = 0 || col = 0 then RefKind.area else RefKind.a1
    let target :=
      if row = 0 && col = 0 then area.ref.target
      else area.ref.target ++ "#INDEX(" ++ toString row ++ "," ++ toString col ++ ")"
    Except.ok (.referenceSlice { kind := kind, target := target }
      (if row = 0 then area.rows else 1)
      (if col = 0 then area.cols else 1))

def selectArrayResult (rows cols row col : Nat) : Except String IndexResult :=
  if row > rows || col > cols then
    Except.error "out_of_bounds"
  else if row = 0 && col = 0 then
    Except.ok (.arraySlice rows cols)
  else if row = 0 then
    Except.ok (.arraySlice rows 1)
  else if col = 0 then
    Except.ok (.arraySlice 1 cols)
  else
    Except.ok (.payloadCell row col)

def evalIndexAdapter
    (source : IndexSource)
    (rowArg : CoercionInput)
    (colArg : Option CoercionInput := none)
    (areaArg : Option CoercionInput := none) : Except String IndexResult := do
  let row ←
    match parseIndexNatDefault (some rowArg) 0 0 with
    | Except.ok v => Except.ok v
    | Except.error _ => Except.error "invalid_index"
  let col ←
    match parseIndexNatDefault colArg 1 0 with
    | Except.ok v => Except.ok v
    | Except.error _ => Except.error "invalid_index"
  let areaNum ←
    match parseAreaNumDefault areaArg with
    | Except.ok v => Except.ok v
    | Except.error _ => Except.error "invalid_area"
  match source with
  | .array rows cols =>
      if areaNum ≠ 1 then
        Except.error "invalid_area"
      else
        selectArrayResult rows cols row col
  | .referenceAreas areas =>
      let some area := listGetAt areas (areaNum - 1)
        | Except.error "invalid_area"
      selectReferenceResult area row col

theorem evalIndexAdapter_missing_row_defaults :
    evalIndexAdapter
      (.referenceAreas [{
        ref := { kind := .area, target := "B1:C2" }
        rows := 2
        cols := 2
      }])
      .missingArg (some (.number 2)) =
      Except.ok (.referenceSlice { kind := .area, target := "B1:C2#INDEX(0,2)" } 2 1) := by
  native_decide

theorem evalIndexAdapter_missing_col_defaults :
    evalIndexAdapter
      (.referenceAreas [{
        ref := { kind := .area, target := "B1:C2" }
        rows := 2
        cols := 2
      }])
      (.number 2) (some .missingArg) =
      Except.ok (.referenceSlice { kind := .area, target := "B1:C2#INDEX(2,0)" } 1 2) := by
  native_decide

theorem evalIndexAdapter_multi_area_selection :
    evalIndexAdapter
      (.referenceAreas [
        { ref := { kind := .area, target := "A1:A2" }, rows := 2, cols := 1 },
        { ref := { kind := .area, target := "G1:G2" }, rows := 2, cols := 1 }
      ])
      (.number 2) (some (.number 1)) (some (.number 2)) =
      Except.ok (.referenceSlice { kind := .a1, target := "G1:G2#INDEX(2,1)" } 1 1) := by
  native_decide

theorem evalIndexAdapter_area_zero_zero_returns_whole_area :
    evalIndexAdapter
      (.referenceAreas [{ ref := { kind := .area, target := "A1:A2" }, rows := 2, cols := 1 }])
      .missingArg (some .missingArg) =
      Except.ok (.referenceSlice { kind := .area, target := "A1:A2" } 2 1) := by
  native_decide

theorem evalIndexAdapter_array_payload :
    evalIndexAdapter (.array 2 3) (.number 1) (some (.number 2)) =
      Except.ok (.payloadCell 1 2) := by
  native_decide

theorem evalIndexAdapter_array_zero_row_slice :
    evalIndexAdapter (.array 2 3) (.number 0) (some (.number 2)) =
      Except.ok (.arraySlice 2 1) := by
  native_decide

theorem evalIndexAdapter_array_zero_col_slice :
    evalIndexAdapter (.array 2 3) (.number 2) (some (.number 0)) =
      Except.ok (.arraySlice 1 3) := by
  native_decide

theorem indexMeta_profiles :
    indexMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ indexMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ indexMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [indexMeta]

end OxFunc.Functions
