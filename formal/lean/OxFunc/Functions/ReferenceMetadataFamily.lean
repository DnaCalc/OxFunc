import OxFunc.FunctionCore
import OxFunc.HostInfoSeam
import OxFunc.RefResolverSeam
import OxFunc.ValueUniverse
import OxFunc.NumericTextRendering

namespace OxFunc.Functions

open OxFunc

def addressMeta : FunctionMeta := {
  functionId := "FUNC.ADDRESS"
  arity := { min := 2, max := 5 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def areasMeta : FunctionMeta := {
  functionId := "FUNC.AREAS"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def formulaTextMeta : FunctionMeta := {
  functionId := "FUNC.FORMULATEXT"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def sheetMeta : FunctionMeta := {
  functionId := "FUNC.SHEET"
  arity := { min := 0, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def sheetsMeta : FunctionMeta := {
  functionId := "FUNC.SHEETS"
  arity := { min := 0, max := 1 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def columnLabelDigits : Nat → Nat → String
  | 0, _ => ""
  | fuel + 1, col =>
      if col = 0 then "" else
        columnLabelDigits fuel ((col - 1) / 26) ++
          String.singleton (Char.ofNat (65 + (col - 1) % 26))

def columnLabelFromIndex (col : Nat) : Option String :=
  if col = 0 || col > 16384 then none else some (columnLabelDigits 3 col)

-- Integer layer after ADDRESS's observed tolerant floor conversion. Relative R1C1 axes carry
-- signed offsets; A1 and absolute R1C1 axes carry one-based sheet coordinates.

-- Adapter binding: `upper` is ceil(x), `gap` is the exact nonnegative dyadic
-- upper-x. On the admitted coordinate/mode range this is the observable
-- RN64-significand -> RN53 rounding boundary of x+2^31, followed by floor.
def addressIntegerFromCeiling (upper : Int) (gap : Rat) : Int :=
  let threshold : Rat := if upper > 0 then 2049 / 8589934592 else 2049 / 17179869184
  if gap ≤ threshold then upper else upper - 1

theorem addressInteger_inclusive_rounding_boundaries :
    addressIntegerFromCeiling 1 (2049 / 8589934592) = 1
    ∧ addressIntegerFromCeiling 1 (2050 / 8589934592) = 0
    ∧ addressIntegerFromCeiling 0 (2049 / 17179869184) = 0
    ∧ addressIntegerFromCeiling 0 (2050 / 17179869184) = -1
    ∧ addressIntegerFromCeiling (-1) (1 / 2) = -2 := by
  native_decide

-- The sheet prefix is already formed before body capacity is considered.
-- Literal fragments clip to available UTF-16 units; integer digits either
-- all fit or contribute no units. The minus sign is a separate literal.
def addressAppendLiteral (buffer text : List UInt16) : List UInt16 :=
  buffer ++ text.take (258 - buffer.length)

def addressAppendDigits (buffer digits : List UInt16) : List UInt16 :=
  if digits.length ≤ 258 - buffer.length then buffer ++ digits else buffer

theorem addressBody_fragment_capacity :
    (addressAppendLiteral (List.replicate 256 120) [36, 88, 70, 68]).length = 258
    ∧ addressAppendDigits (List.replicate 256 120) [49, 48, 52] = List.replicate 256 120
    ∧ (addressAppendDigits (List.replicate 256 120) [49, 50]).length = 258
    ∧ addressAppendLiteral (List.replicate 259 120) [82] = List.replicate 259 120 := by
  native_decide
def addressCoordinateValid (value limit : Int) (relative : Bool) : Bool :=
  if relative then decide (-limit < value ∧ value < limit)
  else decide (1 ≤ value ∧ value ≤ limit)

def formatAddressBody (row col : Int) (absNum : Nat) (a1Style : Bool) : Option String := do
  if absNum = 0 || absNum > 4 then none else pure ()
  let rowRelative := !a1Style && (absNum = 3 || absNum = 4)
  let colRelative := !a1Style && (absNum = 2 || absNum = 4)
  if !addressCoordinateValid row 1048576 rowRelative ||
      !addressCoordinateValid col 16384 colRelative then none else pure ()
  if a1Style then
    let colText ← columnLabelFromIndex col.toNat
    let rowText := toString row
    let colPart := match absNum with
      | 1 | 3 => "$" ++ colText
      | 2 | 4 => colText
      | _ => ""
    let rowPart := match absNum with
      | 1 | 2 => "$" ++ rowText
      | 3 | 4 => rowText
      | _ => ""
    if absNum = 0 || absNum > 4 then none else some (colPart ++ rowPart)
  else
    let rowPart := match absNum with
      | 1 | 2 => "R" ++ toString row
      | 3 | 4 => if row = 0 then "R" else "R[" ++ toString row ++ "]"
      | _ => ""
    let colPart := match absNum with
      | 1 | 3 => "C" ++ toString col
      | 2 | 4 => if col = 0 then "C" else "C[" ++ toString col ++ "]"
      | _ => ""
    if absNum = 0 || absNum > 4 then none else some (rowPart ++ colPart)

-- NameStart/NameContinue and reference-token classification are adapter
-- bindings backed by the versioned class evidence and independent composed
-- name replay. This substrate receives the classification rather than
-- duplicating those tables. Raw UTF-16 units are preserved, even surrogates.
structure AddressPreparedSheet where
  units : List UInt16
  bare : Bool
  deriving DecidableEq, Repr

def addressAsciiUnits (text : String) : List UInt16 :=
  text.toList.map (fun c => UInt16.ofNat c.toNat)

def addressSheetPrefix (sheet : AddressPreparedSheet) : Option (List UInt16) :=
  let escaped := sheet.units.flatMap (fun unit => if unit = 39 then [unit, unit] else [unit])
  if sheet.units.length > 255 || escaped.length > 256 then none
  else some ((if sheet.bare then sheet.units else [39] ++ escaped ++ [39]) ++ [33])

def addressAppendInteger (buffer : List UInt16) (number : Int) : List UInt16 :=
  let signed := if number < 0 then addressAppendLiteral buffer [45] else buffer
  addressAppendDigits signed (addressAsciiUnits (toString number.natAbs))

def addressAppendAxis (buffer : List UInt16) (label : UInt16)
    (number : Int) (relative : Bool) : List UInt16 :=
  let buffer := addressAppendLiteral buffer [label]
  if relative && number = 0 then buffer else
    let buffer := if relative then addressAppendLiteral buffer [91] else buffer
    let buffer := addressAppendInteger buffer number
    if relative then addressAppendLiteral buffer [93] else buffer

def renderAddress (row col : Int) (absNum : Nat) (a1Style : Bool)
    (sheetText? : Option AddressPreparedSheet) : Option (List UInt16) := do
  let _ ← formatAddressBody row col absNum a1Style
  let buffer ← match sheetText? with
    | none => some []
    | some sheet => addressSheetPrefix sheet
  if a1Style then
    let label ← columnLabelFromIndex col.toNat
    let buffer := if absNum = 1 || absNum = 3 then addressAppendLiteral buffer [36] else buffer
    let buffer := addressAppendLiteral buffer (addressAsciiUnits label)
    let buffer := if absNum = 1 || absNum = 2 then addressAppendLiteral buffer [36] else buffer
    pure (addressAppendInteger buffer row)
  else
    let buffer := addressAppendAxis buffer 82 row (absNum = 3 || absNum = 4)
    pure (addressAppendAxis buffer 67 col (absNum = 2 || absNum = 4))

-- Initial decimal rounding binds to ROUND's 15-digit, midpoint-toward-zero
-- primitive; scientific narrowing is a second, decimal half-away step.
def addressScientificDigits (exponent : Int) : Nat :=
  numericTextScientificDigits exponent

def addressReduceDecimalDigits (significand divisor : Nat) : Nat :=
  numericTextReduceDigits significand divisor

theorem addressNumericText_width_and_second_rounding :
    addressScientificDigits 98 = 15 ∧ addressScientificDigits 99 = 14
    ∧ addressScientificDigits (-99) = 14
    ∧ addressReduceDecimalDigits 123456789012345 10 = 12345678901235 := by
  native_decide

theorem addressSheetPrefix_preserves_raw_units_and_escape_bounds :
    addressSheetPrefix ⟨[0xD800, 122], true⟩ = some [0xD800, 122, 33]
    ∧ addressSheetPrefix ⟨[0xDC00, 122], false⟩ = some [39, 0xDC00, 122, 39, 33]
    ∧ (addressSheetPrefix ⟨List.replicate 128 39, false⟩).map List.length = some 259
    ∧ addressSheetPrefix ⟨List.replicate 129 39, false⟩ = none := by
  native_decide

def countAreas (target : String) : Nat :=
  if target = "(A1,B2:B3)" then 2 else 1

theorem referenceMetadataFamily_meta_profiles :
    addressMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ areasMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ formulaTextMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ sheetMeta.fecDependencyProfile = FecDependencyProfile.composite
    ∧ sheetsMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [addressMeta, areasMeta, formulaTextMeta, sheetMeta, sheetsMeta]

theorem renderAddress_seed_a1_absolute :
    renderAddress 3 2 1 true none = some (addressAsciiUnits "$B$3") := by
  native_decide

theorem renderAddress_seed_r1c1_with_sheet_text :
    renderAddress 3 2 4 false (some ⟨addressAsciiUnits "Alpha", true⟩) = some (addressAsciiUnits "Alpha!R[3]C[2]") := by
  native_decide

theorem renderAddress_seed_quoted_sheet_text :
    renderAddress 3 2 1 true (some ⟨addressAsciiUnits "Quarter 1", false⟩) = some (addressAsciiUnits "'Quarter 1'!$B$3") := by
  native_decide

theorem renderAddress_relative_signed_and_zero :
    renderAddress (-1) 0 4 false none = some (addressAsciiUnits "R[-1]C")
    ∧ renderAddress 0 1 3 false none = some (addressAsciiUnits "RC1")
    ∧ renderAddress 0 0 4 false none = some (addressAsciiUnits "RC") := by
  native_decide

theorem renderAddress_sheet_limits :
    renderAddress 1048576 16384 1 true none = some (addressAsciiUnits "$XFD$1048576")
    ∧ renderAddress 1048577 1 1 true none = none
    ∧ renderAddress 1 16385 1 false none = none
    ∧ renderAddress 1048576 16384 4 false none = none := by
  native_decide

theorem countAreas_seed_union :
    countAreas "(A1,B2:B3)" = 2 := by
  rfl

theorem hostInfoSeam_supports_reference_metadata_queries :
    providerCanServeFormulaTextQuery "A1" = true
    ∧ providerCanServeSheetIdentity (.currentSheet)
    ∧ providerCanServeSheetIdentity (.reference "Beta!A1")
    ∧ providerCanServeSheetIdentity (.sheetNameText "Alpha")
    ∧ providerCanServeSheetCount (.workbook)
    ∧ providerCanServeSheetCount (.reference "'Quarter 1':Alpha!A1") := by
  simp [providerCanServeFormulaTextQuery, providerCanServeSheetIdentity, providerCanServeSheetCount]

end OxFunc.Functions
