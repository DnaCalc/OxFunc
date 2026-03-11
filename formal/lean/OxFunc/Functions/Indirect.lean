import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

instance instDecidableEqExceptIndirect [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def indirectMeta : FunctionMeta := {
  functionId := "FUNC.INDIRECT"
  arity := { min := 1, max := 2 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.volatileContextual
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.callerContext
  surfaceFecDependencyProfile := FecDependencyProfile.callerContext
}

structure CallerContext where
  sheetPrefix : Option String
  rowIndex : Nat
  colIndex : Nat
  deriving DecidableEq, Repr

inductive IndirectTextSpec where
  | a1Cell (target : String)
  | a1Area (target : String)
  | wholeColumn (target : String)
  | wholeRow (target : String)
  | r1c1Absolute (row : Nat) (col : Nat)
  | r1c1Relative (rowDelta : Int) (colDelta : Int)
  deriving DecidableEq, Repr

def parseA1Style (arg : Option CoercionInput) : Except CoercionError Bool :=
  match arg with
  | none => Except.ok true
  | some .missingArg | some .emptyCell => Except.ok false
  | some other =>
      match coerceToNumber other with
      | Except.ok n => Except.ok (n ≠ 0)
      | Except.error e => Except.error e

def columnLabel (col : Nat) : String :=
  if col = 0 then
    ""
  else
    String.ofList [Char.ofNat (65 + ((col - 1) % 26))]

def attachPrefix (sheetPrefix : Option String) (body : String) : String :=
  match sheetPrefix with
  | some p => p ++ "!" ++ body
  | none => body

def renderA1Target (sheetPrefix : Option String) (row col : Nat) : String :=
  attachPrefix sheetPrefix (columnLabel col ++ toString row)

def resolveR1C1
    (ctx : CallerContext)
    (rowDelta : Int)
    (colDelta : Int) : Option (Nat × Nat) := do
  let row' := Int.ofNat ctx.rowIndex + rowDelta
  let col' := Int.ofNat ctx.colIndex + colDelta
  if row' > 0 then
    if col' > 0 then
      some (Int.toNat row', Int.toNat col')
    else
      none
  else
    none

def evalIndirectAdapter
    (spec : IndirectTextSpec)
    (a1StyleArg : Option CoercionInput := none)
    (caller : Option CallerContext := none) :
    Except String ReferenceToken := do
  let a1Style ←
    match parseA1Style a1StyleArg with
    | Except.ok v => Except.ok v
    | Except.error _ => Except.error "invalid_a1_style"
  if a1Style then
    match spec with
    | .a1Cell target => Except.ok { kind := .a1, target := target }
    | .a1Area target => Except.ok { kind := .area, target := target }
    | .wholeColumn target => Except.ok { kind := .area, target := target }
    | .wholeRow target => Except.ok { kind := .area, target := target }
    | .r1c1Absolute _ _ => Except.error "invalid_ref_text"
    | .r1c1Relative _ _ => Except.error "invalid_ref_text"
  else
    match spec with
    | .r1c1Absolute row col =>
        let sheetPref := caller.bind (fun ctx => ctx.sheetPrefix)
        Except.ok { kind := .a1, target := renderA1Target sheetPref row col }
    | .r1c1Relative rowDelta colDelta =>
        let some ctx := caller | Except.error "invalid_ref_text"
        let some (row, col) := resolveR1C1 ctx rowDelta colDelta | Except.error "invalid_ref_text"
        Except.ok { kind := .a1, target := renderA1Target ctx.sheetPrefix row col }
    | .a1Cell _ => Except.error "invalid_ref_text"
    | .a1Area _ => Except.error "invalid_ref_text"
    | .wholeColumn _ => Except.error "invalid_ref_text"
    | .wholeRow _ => Except.error "invalid_ref_text"

theorem evalIndirectAdapter_a1_ok :
    evalIndirectAdapter (.a1Cell "Sheet1!A1") =
      Except.ok { kind := .a1, target := "Sheet1!A1" } := by
  native_decide

theorem evalIndirectAdapter_missing_a1_style_behaves_false :
    evalIndirectAdapter (.r1c1Absolute 1 11) (some .missingArg) =
      Except.ok { kind := .a1, target := "K1" } := by
  native_decide

theorem evalIndirectAdapter_relative_r1c1_uses_caller_context :
    evalIndirectAdapter
      (.r1c1Relative (-2) (-1))
      (some (.number 0))
      (some { sheetPrefix := some "Sheet1", rowIndex := 3, colIndex := 3 }) =
      Except.ok { kind := .a1, target := "Sheet1!B1" } := by
  native_decide

theorem evalIndirectAdapter_whole_column_a1 :
    evalIndirectAdapter (.wholeColumn "Sheet1!K:K") =
      Except.ok { kind := .area, target := "Sheet1!K:K" } := by
  native_decide

theorem evalIndirectAdapter_whole_row_a1 :
    evalIndirectAdapter (.wholeRow "1:1") =
      Except.ok { kind := .area, target := "1:1" } := by
  native_decide

theorem evalIndirectAdapter_relative_r1c1_requires_context :
    evalIndirectAdapter (.r1c1Relative 1 1) (some (.number 0)) =
      Except.error "invalid_ref_text" := by
  native_decide

theorem indirectMeta_profiles :
    indirectMeta.volatility = VolatilityClass.volatileContextual
    ∧ indirectMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ indirectMeta.fecDependencyProfile = FecDependencyProfile.callerContext
    ∧ indirectMeta.surfaceFecDependencyProfile = FecDependencyProfile.callerContext := by
  simp [indirectMeta]

end OxFunc.Functions
