import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.RefResolverSeam
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptOffset [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def offsetMeta : FunctionMeta := {
  functionId := "FUNC.OFFSET"
  arity := { min := 3, max := 5 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.callerContext
  surfaceFecDependencyProfile := FecDependencyProfile.callerContext
}

structure OffsetRef where
  startRow : Nat
  startCol : Nat
  endRow : Nat
  endCol : Nat
  sheetPrefix : Option String
  deriving DecidableEq, Repr

def OffsetRef.width (ref : OffsetRef) : Nat :=
  ref.endCol - ref.startCol + 1

def OffsetRef.height (ref : OffsetRef) : Nat :=
  ref.endRow - ref.startRow + 1

private def truncateOffsetNumber (q : Rat) : Int :=
  if q.num < 0 then
    -Int.ediv (-q.num) q.den
  else
    Int.ediv q.num q.den

private def parseOffsetNumber : CoercionInput → Except WorksheetErrorCode Int
  | input =>
      match coerceToNumber input with
      | .ok n => .ok (truncateOffsetNumber n)
      | .error (.worksheetError code) => .error code
      | .error _ => .error .value

private def parsePositiveDimension : Option CoercionInput → Except WorksheetErrorCode (Option Nat)
  | none => .ok none
  | some input =>
      match parseOffsetNumber input with
      | .ok n =>
          if n ≤ 0 then
            .error .ref
          else
            .ok (some n.toNat)
      | .error e => .error e

private def columnLabel : Nat → String
  | 0 => ""
  | n =>
      let q := (n - 1) / 26
      let rem := (n - 1) % 26
      let prefixText := if q = 0 then "" else columnLabel q
      prefixText ++ String.singleton (Char.ofNat (65 + rem))

private def renderOffsetRef (ref : OffsetRef) : String :=
  let start := s!"{columnLabel ref.startCol}{ref.startRow}"
  let finish := s!"{columnLabel ref.endCol}{ref.endRow}"
  let body := if start = finish then start else s!"{start}:{finish}"
  match ref.sheetPrefix with
  | some sheetPrefix => s!"{sheetPrefix}!{body}"
  | none => body

private def offsetReference
    (base : OffsetRef)
    (rowOffset colOffset : Int)
    (height width : Option Nat) : Except WorksheetErrorCode OffsetRef := do
  let top := (base.startRow : Int) + rowOffset
  let left := (base.startCol : Int) + colOffset
  if top ≤ 0 ∨ left ≤ 0 then
    .error .ref
  else
    let h := height.getD base.height
    let w := width.getD base.width
    if h = 0 ∨ w = 0 then
      .error .ref
    else
      let startRow := top.toNat
      let startCol := left.toNat
      .ok {
        startRow := startRow
        startCol := startCol
        endRow := startRow + h - 1
        endCol := startCol + w - 1
        sheetPrefix := base.sheetPrefix
      }

def evalOffsetPrepared
    (base : OffsetRef)
    (rows cols : CoercionInput)
    (height width : Option CoercionInput := none) :
    Except WorksheetErrorCode ReferenceToken := do
  let rowOffset ← parseOffsetNumber rows
  let colOffset ← parseOffsetNumber cols
  let heightValue ← parsePositiveDimension height
  let widthValue ← parsePositiveDimension width
  let shifted ← offsetReference base rowOffset colOffset heightValue widthValue
  .ok {
    kind := if shifted.width = 1 ∧ shifted.height = 1 then .a1 else .area
    target := renderOffsetRef shifted
  }

theorem evalOffsetPrepared_shifts_single_cell_reference :
    evalOffsetPrepared
      { startRow := 2, startCol := 2, endRow := 2, endCol := 2, sheetPrefix := none }
      (.number 1) (.number 2) =
      .ok { kind := .a1, target := "D3" } := by
  native_decide

theorem evalOffsetPrepared_resizes_area_reference :
    evalOffsetPrepared
      { startRow := 1, startCol := 1, endRow := 2, endCol := 2, sheetPrefix := none }
      (.number 0) (.number 1) (some (.number 1)) (some (.number 3)) =
      .ok { kind := .area, target := "B1:D1" } := by
  native_decide

theorem evalOffsetPrepared_defaults_height_and_width_to_base_shape :
    evalOffsetPrepared
      { startRow := 2, startCol := 2, endRow := 3, endCol := 3, sheetPrefix := none }
      (.number 1) (.number 1) =
      .ok { kind := .area, target := "C3:D4" } := by
  native_decide

theorem evalOffsetPrepared_preserves_sheet_prefix :
    evalOffsetPrepared
      { startRow := 2, startCol := 2, endRow := 2, endCol := 2, sheetPrefix := some "Sheet1" }
      (.number 1) (.number 2) =
      .ok { kind := .a1, target := "Sheet1!D3" } := by
  native_decide

theorem offsetMeta_profiles :
    offsetMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ offsetMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ offsetMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [offsetMeta]

end OxFunc.Functions
