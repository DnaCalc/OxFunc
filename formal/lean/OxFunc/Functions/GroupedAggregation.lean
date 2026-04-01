import OxFunc.FunctionCore

namespace OxFunc.Functions

inductive GroupCell where
  | text (s : String)
  | number (n : Int)
  | blank
  deriving DecidableEq, Repr

structure GroupEntry where
  key : String
  value : Int
  deriving DecidableEq, Repr

structure NestedGroupEntry where
  outerKey : String
  innerKey : String
  value : Int
  deriving DecidableEq, Repr

structure PivotEntry where
  rowKey : String
  colKey : String
  value : Int
  deriving DecidableEq, Repr

def orderedDistinct [DecidableEq α] (xs : List α) : List α :=
  xs.foldl (fun acc x => if x ∈ acc then acc else acc ++ [x]) []

def sumInts (xs : List Int) : Int :=
  xs.foldl (fun acc x => acc + x) 0

def filterWithMask (xs : List α) (mask : List Bool) : List α :=
  (List.zip xs mask).foldr (fun pair acc => if pair.snd then pair.fst :: acc else acc) []

def insertByScoreDesc (score : α → Int) (x : α) : List α → List α
  | [] => [x]
  | y :: ys => if score x > score y then x :: y :: ys else y :: insertByScoreDesc score x ys

def sortByScoreDesc (score : α → Int) (xs : List α) : List α :=
  xs.foldr (insertByScoreDesc score) []

def groupBySum (entries : List GroupEntry) : List (String × Int) :=
  let keys := orderedDistinct (entries.map fun e => e.key)
  keys.map fun key =>
    let values := (entries.filter fun e => e.key = key).map fun e => e.value
    (key, sumInts values)

def appendGrandTotal (rows : List (String × Int)) : List (String × Int) :=
  rows ++ [("Total", sumInts (rows.map Prod.snd))]

def renderGroupedRows (rows : List (String × Int)) : List (List GroupCell) :=
  rows.map fun row => [GroupCell.text row.fst, GroupCell.number row.snd]

def renderGroupedRowsWithTotal (rows : List (String × Int)) : List (List GroupCell) :=
  renderGroupedRows (appendGrandTotal rows)

def groupBySumDescending (entries : List GroupEntry) : List (String × Int) :=
  sortByScoreDesc Prod.snd (groupBySum entries)

def hierarchicalSubtotalRows (entries : List NestedGroupEntry) : List (List GroupCell) :=
  let outerKeys := orderedDistinct (entries.map fun e => e.outerKey)
  let groupedKeys := orderedDistinct (entries.map fun e => (e.outerKey, e.innerKey))
  let rendered :=
    outerKeys.map fun outer =>
      let detailRows :=
        (groupedKeys.filter fun key => key.fst = outer).map fun key =>
          let subtotal := sumInts ((entries.filter fun e => e.outerKey = key.fst && e.innerKey = key.snd).map fun e => e.value)
          [GroupCell.text key.fst, GroupCell.text key.snd, GroupCell.number subtotal]
      let subtotalRow :=
        [GroupCell.text outer, GroupCell.blank, GroupCell.number (sumInts ((entries.filter fun e => e.outerKey = outer).map fun e => e.value))]
      detailRows ++ [subtotalRow]
  let grandTotal := [GroupCell.text "Grand Total", GroupCell.blank, GroupCell.number (sumInts (entries.map fun e => e.value))]
  List.join rendered ++ [grandTotal]

def orderedRowKeys (entries : List PivotEntry) : List String :=
  orderedDistinct (entries.map fun e => e.rowKey)

def orderedColKeys (entries : List PivotEntry) : List String :=
  orderedDistinct (entries.map fun e => e.colKey)

def pivotValueAt (entries : List PivotEntry) (rowKey colKey : String) : Int :=
  sumInts ((entries.filter fun e => e.rowKey = rowKey && e.colKey = colKey).map fun e => e.value)

def pivotRowTotal (entries : List PivotEntry) (rowKey : String) : Int :=
  sumInts ((entries.filter fun e => e.rowKey = rowKey).map fun e => e.value)

def pivotColTotal (entries : List PivotEntry) (colKey : String) : Int :=
  sumInts ((entries.filter fun e => e.colKey = colKey).map fun e => e.value)

def pivotGrandTotal (entries : List PivotEntry) : Int :=
  sumInts (entries.map fun e => e.value)

def pivotTableWithTotals (entries : List PivotEntry) (rowKeys : List String := orderedRowKeys entries) (colKeys : List String := orderedColKeys entries) : List (List GroupCell) :=
  let header := [GroupCell.blank] ++ (colKeys.map GroupCell.text) ++ [GroupCell.text "Total"]
  let body :=
    rowKeys.map fun rowKey =>
      [GroupCell.text rowKey] ++
      (colKeys.map fun colKey => GroupCell.number (pivotValueAt entries rowKey colKey)) ++
      [GroupCell.number (pivotRowTotal entries rowKey)]
  let totalRow :=
    [GroupCell.text "Total"] ++
    (colKeys.map fun colKey => GroupCell.number (pivotColTotal entries colKey)) ++
    [GroupCell.number (pivotGrandTotal entries)]
  header :: body ++ [totalRow]

def pivotTableWithoutTotals (entries : List PivotEntry) : List (List GroupCell) :=
  let rowKeys := orderedRowKeys entries
  let colKeys := orderedColKeys entries
  let header := [GroupCell.blank] ++ (colKeys.map GroupCell.text)
  let body :=
    rowKeys.map fun rowKey =>
      [GroupCell.text rowKey] ++ (colKeys.map fun colKey => GroupCell.number (pivotValueAt entries rowKey colKey))
  header :: body

def groupByDefaultExample : List (List GroupCell) :=
  renderGroupedRowsWithTotal <|
    groupBySum
      [ { key := "2024", value := 10 }
      , { key := "2024", value := 20 }
      , { key := "2025", value := 30 }
      , { key := "2025", value := 40 } ]

def groupByFilterSortExample : List (List GroupCell) :=
  renderGroupedRowsWithTotal <|
    groupBySumDescending <|
      filterWithMask
        [ { key := "A", value := 10 }
        , { key := "B", value := 20 }
        , { key := "A", value := 40 }
        , { key := "B", value := 50 } ]
        [true, false, true, false]

def groupByHierarchicalSubtotalExample : List (List GroupCell) :=
  hierarchicalSubtotalRows
    [ { outerKey := "East", innerKey := "A", value := 10 }
    , { outerKey := "East", innerKey := "B", value := 20 }
    , { outerKey := "East", innerKey := "A", value := 30 }
    , { outerKey := "West", innerKey := "A", value := 40 }
    , { outerKey := "West", innerKey := "B", value := 50 } ]

def pivotByDefaultExample : List (List GroupCell) :=
  pivotTableWithTotals
    [ { rowKey := "East", colKey := "A", value := 10 }
    , { rowKey := "East", colKey := "B", value := 20 }
    , { rowKey := "West", colKey := "A", value := 40 }
    , { rowKey := "West", colKey := "B", value := 50 } ]

def pivotByFilterZeroTotalsExample : List (List GroupCell) :=
  pivotTableWithoutTotals <|
    filterWithMask
      [ { rowKey := "East", colKey := "A", value := 10 }
      , { rowKey := "East", colKey := "B", value := 20 }
      , { rowKey := "West", colKey := "A", value := 40 }
      , { rowKey := "West", colKey := "B", value := 50 } ]
      [true, false, true, false]

def pivotBySortedExample : List (List GroupCell) :=
  let entries :=
    [ { rowKey := "East", colKey := "A", value := 10 }
    , { rowKey := "East", colKey := "B", value := 20 }
    , { rowKey := "West", colKey := "A", value := 40 }
    , { rowKey := "West", colKey := "B", value := 50 } ]
  let rowKeys := sortByScoreDesc (pivotRowTotal entries) (orderedRowKeys entries)
  let colKeys := sortByScoreDesc (pivotColTotal entries) (orderedColKeys entries)
  pivotTableWithTotals entries rowKeys colKeys

end OxFunc.Functions
