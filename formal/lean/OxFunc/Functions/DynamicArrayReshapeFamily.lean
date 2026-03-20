import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def dynamicArrayReshapeMetaBase : FunctionMeta := {
  functionId := "FUNC.DYNAMIC_ARRAY_RESHAPE_BASE"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def chooseColsMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.CHOOSECOLS", arity := { min := 2, max := 255 }
}

def chooseRowsMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.CHOOSEROWS", arity := { min := 2, max := 255 }
}

def dropMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.DROP", arity := { min := 2, max := 3 }
}

def expandMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.EXPAND", arity := { min := 2, max := 4 }
}

def filterMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.FILTER", arity := { min := 2, max := 3 }
}

def sortMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.SORT", arity := { min := 1, max := 4 }
}

def sortByMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.SORTBY", arity := { min := 2, max := 30 }
}

def takeMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.TAKE", arity := { min := 2, max := 3 }
}

def toColMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.TOCOL", arity := { min := 1, max := 3 }
}

def toRowMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.TOROW", arity := { min := 1, max := 3 }
}

def transposeMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.TRANSPOSE", arity := Arity.exact 1
}

def uniqueMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.UNIQUE", arity := { min := 1, max := 3 }
}

def vstackMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.VSTACK", arity := { min := 1, max := 255 }
}

def wrapColsMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.WRAPCOLS", arity := { min := 2, max := 3 }
}

def wrapRowsMeta : FunctionMeta := {
  dynamicArrayReshapeMetaBase with functionId := "FUNC.WRAPROWS", arity := { min := 2, max := 3 }
}

inductive ShapeCell where
  | number (n : Int)
  | text (s : String)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  | blank
  deriving DecidableEq, Repr

structure ShapeArray where
  rows : Nat
  cols : Nat
  payload : List ShapeCell
  deriving DecidableEq, Repr

private def shapeListGetD (xs : List α) (idx : Nat) (fallback : α) : α :=
  match xs.drop idx with
  | [] => fallback
  | x :: _ => x

def concatLists (xss : List (List α)) : List α :=
  xss.foldr List.append []

def ShapeArray.rowMajorGet! (arr : ShapeArray) (row col : Nat) : ShapeCell :=
  shapeListGetD arr.payload (row * arr.cols + col) .blank

def ShapeArray.fromRows (rows : List (List ShapeCell)) : ShapeArray :=
  let rowCount := rows.length
  let colCount := match rows with
    | [] => 0
    | r :: _ => r.length
  { rows := rowCount, cols := colCount, payload := concatLists rows }

def ShapeArray.toRows (arr : ShapeArray) : List (List ShapeCell) :=
  (List.range arr.rows).map fun row =>
    (List.range arr.cols).map fun col => arr.rowMajorGet! row col

def normalizeSelector (len : Nat) (idx : Int) : Option Nat :=
  if idx = 0 then
    none
  else if idx > 0 then
    let n := Int.toNat (idx - 1)
    if n < len then some n else none
  else
    let n := Int.toNat (Int.ofNat len + idx)
    if n < len then some n else none

def chooseCols (arr : ShapeArray) (selectors : List Int) : Option ShapeArray := do
  let cols ← selectors.mapM (normalizeSelector arr.cols)
  let rows := (List.range arr.rows).map fun row =>
    cols.map fun col => arr.rowMajorGet! row col
  pure (ShapeArray.fromRows rows)

def chooseRows (arr : ShapeArray) (selectors : List Int) : Option ShapeArray := do
  let rows ← selectors.mapM (normalizeSelector arr.rows)
  pure (ShapeArray.fromRows (rows.map fun row =>
    (List.range arr.cols).map fun col => arr.rowMajorGet! row col))

def spanTake (len : Nat) (count : Int) : Option (Nat × Nat) :=
  if count = 0 then
    none
  else if count > 0 then
    let n := min (Int.toNat count) len
    some (0, n)
  else
    let n := min (Int.toNat (-count)) len
    some (len - n, len)

def spanDrop (len : Nat) (count : Int) : Option (Nat × Nat) :=
  let start := if count >= 0 then min (Int.toNat count) len else 0
  let stop := if count >= 0 then len else len - Int.toNat (-count)
  if start < stop then some (start, stop) else none

def slice (arr : ShapeArray) (rowStart rowStop colStart colStop : Nat) : ShapeArray :=
  ShapeArray.fromRows <|
    (List.range (rowStop - rowStart)).map fun dRow =>
      let row := rowStart + dRow
      (List.range (colStop - colStart)).map fun dCol =>
        let col := colStart + dCol
        arr.rowMajorGet! row col

def takeSlice (arr : ShapeArray) (rowCount : Int) (colCount : Int) : Option ShapeArray := do
  let (rowStart, rowStop) ← spanTake arr.rows rowCount
  let (colStart, colStop) ← spanTake arr.cols colCount
  pure (slice arr rowStart rowStop colStart colStop)

def dropSlice (arr : ShapeArray) (rowCount : Int) (colCount : Int) : Option ShapeArray := do
  let (rowStart, rowStop) ← spanDrop arr.rows rowCount
  let (colStart, colStop) ← spanDrop arr.cols colCount
  pure (slice arr rowStart rowStop colStart colStop)

def expandWith (arr : ShapeArray) (targetRows targetCols : Nat) (fill : ShapeCell) : Option ShapeArray :=
  if targetRows < arr.rows || targetCols < arr.cols then
    none
  else
    pure <| ShapeArray.fromRows <|
      (List.range targetRows).map fun row =>
        (List.range targetCols).map fun col =>
          if row < arr.rows then
            if col < arr.cols then
              arr.rowMajorGet! row col
            else fill
          else fill

def transpose (arr : ShapeArray) : ShapeArray :=
  ShapeArray.fromRows <|
    (List.range arr.cols).map fun col =>
      (List.range arr.rows).map fun row => arr.rowMajorGet! row col

def flattenRowMajor (arr : ShapeArray) : List ShapeCell :=
  arr.payload

def flattenColMajor (arr : ShapeArray) : List ShapeCell :=
  concatLists <| (List.range arr.cols).map fun col =>
    (List.range arr.rows).map fun row => arr.rowMajorGet! row col

def ignoreCell (mode : Nat) (cell : ShapeCell) : Bool :=
  match mode, cell with
  | 0, _ => false
  | 1, .blank => true
  | 2, .error _ => true
  | 3, .blank => true
  | 3, .error _ => true
  | _, _ => false

def toCol (arr : ShapeArray) (ignoreMode : Nat := 0) (byCol : Bool := false) : Option ShapeArray :=
  let flat := (if byCol then flattenColMajor arr else flattenRowMajor arr).filter (fun c => !ignoreCell ignoreMode c)
  if flat.isEmpty then none else some { rows := flat.length, cols := 1, payload := flat }

def toRow (arr : ShapeArray) (ignoreMode : Nat := 0) (byCol : Bool := false) : Option ShapeArray :=
  let flat := (if byCol then flattenColMajor arr else flattenRowMajor arr).filter (fun c => !ignoreCell ignoreMode c)
  if flat.isEmpty then none else some { rows := 1, cols := flat.length, payload := flat }

def padRows (payload : List ShapeCell) (n : Nat) (fill : ShapeCell) : List ShapeCell :=
  payload ++ List.replicate (n - payload.length) fill

def vstack (arrays : List ShapeArray) : Option ShapeArray :=
  match arrays with
  | [] => none
  | first :: rest =>
      let all := first :: rest
      let cols := all.foldl (fun acc arr => max acc arr.cols) 0
      let rows := all.foldl (fun acc arr => acc + arr.rows) 0
      let fill := ShapeCell.error WorksheetErrorCode.na
      let payload := concatLists <| all.map fun arr =>
        concatLists <| (arr.toRows.map fun row => padRows row cols fill)
      some { rows := rows, cols := cols, payload := payload }

def wrapRows (flat : List ShapeCell) (wrapCount : Nat) (fill : ShapeCell) : Option ShapeArray :=
  if wrapCount = 0 then
    none
  else
    let rows := (flat.length + wrapCount - 1) / wrapCount
    let payload := padRows flat (rows * wrapCount) fill
    some { rows := rows, cols := wrapCount, payload := payload }

def wrapCols (flat : List ShapeCell) (wrapCount : Nat) (fill : ShapeCell) : Option ShapeArray :=
  if wrapCount = 0 then
    none
  else
    let cols := (flat.length + wrapCount - 1) / wrapCount
      let padded := padRows flat (wrapCount * cols) fill
      let payload :=
        concatLists <| (List.range wrapCount).map fun row =>
        (List.range cols).map fun col => shapeListGetD padded (col * wrapCount + row) fill
    some { rows := wrapCount, cols := cols, payload := payload }

def distinctRows (rows : List (List ShapeCell)) : List (List ShapeCell) :=
  rows.foldl
    (fun acc row => if row ∈ acc then acc else acc ++ [row])
    []

def exactOnceRows (rows : List (List ShapeCell)) : List (List ShapeCell) :=
  rows.filter (fun row => rows.count row = 1)

def uniqueRows (arr : ShapeArray) (exactlyOnce : Bool := false) : Option ShapeArray :=
  let rows := arr.toRows
  let kept := if exactlyOnce then exactOnceRows rows else distinctRows rows
  if kept.isEmpty then none else some (ShapeArray.fromRows kept)

def seededBase : ShapeArray :=
  ShapeArray.fromRows
    [
      [.number 1, .number 2, .number 3],
      [.number 4, .number 5, .number 6],
      [.number 7, .number 8, .number 9]
    ]

theorem dynamicArrayReshape_meta_profiles :
    chooseColsMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ filterMeta.hostInteraction = HostInteractionClass.none
    ∧ sortMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ transposeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ uniqueMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ vstackMeta.coercionLiftProfile = CoercionLiftProfile.custom := by
  simp [
    dynamicArrayReshapeMetaBase,
    chooseColsMeta,
    filterMeta,
    sortMeta,
    transposeMeta,
    uniqueMeta,
    vstackMeta
  ]

theorem chooseCols_seeded_example :
    chooseCols seededBase [3, 1] =
      some (ShapeArray.fromRows [[.number 3, .number 1], [.number 6, .number 4], [.number 9, .number 7]]) := by
  rfl

theorem chooseRows_seeded_example :
    chooseRows seededBase [-1, 1] =
      some (ShapeArray.fromRows [[.number 7, .number 8, .number 9], [.number 1, .number 2, .number 3]]) := by
  rfl

theorem take_seeded_example :
    takeSlice seededBase 2 (-2) =
      some (ShapeArray.fromRows [[.number 2, .number 3], [.number 5, .number 6]]) := by
  rfl

theorem drop_seeded_example :
    dropSlice seededBase 1 (-1) =
      some (ShapeArray.fromRows [[.number 4, .number 5], [.number 7, .number 8]]) := by
  native_decide

theorem transpose_seeded_example :
    transpose seededBase =
      ShapeArray.fromRows
        [[.number 1, .number 4, .number 7], [.number 2, .number 5, .number 8], [.number 3, .number 6, .number 9]] := by
  rfl

theorem toCol_seeded_example :
    toCol seededBase = some { rows := 9, cols := 1, payload := flattenRowMajor seededBase } := by
  native_decide

theorem wrapRows_seeded_example :
    wrapRows [.number 1, .number 2, .number 3, .number 4, .number 5] 2 (.error .na) =
      some (ShapeArray.fromRows [[.number 1, .number 2], [.number 3, .number 4], [.number 5, .error .na]]) := by
  native_decide

theorem wrapCols_seeded_example :
    wrapCols [.number 1, .number 2, .number 3, .number 4, .number 5] 2 (.error .na) =
      some (ShapeArray.fromRows [[.number 1, .number 3, .number 5], [.number 2, .number 4, .error .na]]) := by
  rfl

theorem vstack_seeded_example :
    vstack
      [ ShapeArray.fromRows [[.number 1, .number 2]]
      , ShapeArray.fromRows [[.number 3], [.number 4]]
      ] =
      some (ShapeArray.fromRows [[.number 1, .number 2], [.number 3, .error .na], [.number 4, .error .na]]) := by
  rfl

theorem unique_seeded_example :
    uniqueRows (ShapeArray.fromRows [[.number 1, .number 10], [.number 1, .number 10], [.number 2, .number 20]]) =
      some (ShapeArray.fromRows [[.number 1, .number 10], [.number 2, .number 20]]) := by
  native_decide

end OxFunc.Functions
