import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptEngineeringRadix [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

/-- Engineering radix width admission, finite-number substrate. Validation
precedes negative ten-digit encoding; a valid width is ignored for a negative
number, but an invalid width still rejects it. -/
def engineeringRadixWidth (places : Option Rat) : Option (Option Nat) :=
  match places with
  | none => some none
  | some p =>
    let width := p.num.tdiv (Int.ofNat p.den)
    if width < 1 ∨ width > 10 then none else some (some width.toNat)

def engineeringRadixPublishedWidth (number : Int) (places : Option Rat) : Option (Option Nat) := do
  let width ← engineeringRadixWidth places
  return if number < 0 then some 10 else width

theorem engineeringRadixWidth_admission :
    engineeringRadixWidth (some 0) = none ∧
    engineeringRadixWidth (some ((9 : Rat) / 10)) = none ∧
    engineeringRadixWidth (some ((109 : Rat) / 10)) = some (some 10) ∧
    engineeringRadixWidth (some 11) = none := by native_decide

theorem engineeringRadixPublishedWidth_negative :
    engineeringRadixPublishedWidth (-1) (some 0) = none ∧
    engineeringRadixPublishedWidth (-1) (some 1) = some (some 10) ∧
    engineeringRadixPublishedWidth (-1) (some 11) = none := by native_decide

/-- Function-specific preparation: the optional width is validated first,
including on negative encodings and before an error in the source argument. -/
def engineeringRadixNumber : CoercionInput → Except WorksheetErrorCode Rat
  | .logical _ => .error .value
  | .missingArg => .error .na
  | .emptyCell => .ok 0
  | arg => match coerceToNumber arg with
    | .ok n => .ok n
    | .error (.worksheetError code) => .error code
    | .error _ => .error .value

def engineeringRadixPlaces (places : Option CoercionInput) : Except WorksheetErrorCode (Option Nat) := do
  match places with
  | none | some .missingArg => .ok none
  | some arg =>
    let p ← engineeringRadixNumber arg
    match engineeringRadixWidth (some p) with
    | none => .error .num
    | some width => .ok width

def engineeringRadixDecimalInputs (number : CoercionInput) (places : Option CoercionInput) :
    Except WorksheetErrorCode (Int × Option Nat) := do
  let width ← engineeringRadixPlaces places
  let number ← engineeringRadixNumber number
  pure (number.num.tdiv (Int.ofNat number.den), width)

def engineeringAsciiDigit (c : Char) : Option Nat :=
  let n := c.toNat
  if 48 ≤ n ∧ n ≤ 57 then some (n - 48)
  else if 65 ≤ n ∧ n ≤ 90 then some (n - 65 + 10)
  else if 97 ≤ n ∧ n ≤ 122 then some (n - 97 + 10)
  else none

/-- The admitted integer substrate for ten-digit binary/octal/hexadecimal
source text. No whitespace normalization precedes this parser. -/
def engineeringRadixParse (radix bits : Nat) (text : String) : Option Int := do
  if text.length > 10 then none else
  let raw ← text.toList.foldlM (fun n c => do
    let digit ← engineeringAsciiDigit c
    if digit < radix then some (n * radix + digit) else none) 0
  pure (if text.length = 10 ∧ raw ≥ 2 ^ (bits - 1)
    then (Int.ofNat raw) - Int.ofNat (2 ^ bits) else Int.ofNat raw)

def engineeringRadixDigits (radix : Nat) : Nat → Nat → List Char → List Char
  | 0, _, acc => acc
  | fuel + 1, number, acc =>
    let digit := number % radix
    let acc := Char.ofNat (if digit < 10 then 48 + digit else 65 + digit - 10) :: acc
    if number < radix then acc else engineeringRadixDigits radix fuel (number / radix) acc

def engineeringRadixEncode (radix bits : Nat) (number : Int) (places : Option Rat) : Option String := do
  let width ← engineeringRadixWidth places
  let limit := Int.ofNat (2 ^ (bits - 1))
  if number < -limit ∨ number ≥ limit then none else
  let raw := if number < 0 then (number + Int.ofNat (2 ^ bits)).toNat else number.toNat
  let digits := engineeringRadixDigits radix 10 raw []
  let width := if number < 0 then some 10 else width
  match width with
  | none => some (String.ofList digits)
  | some size =>
      if digits.length > size then none
      else some (String.ofList (List.replicate (size - digits.length) '0' ++ digits))

theorem engineeringRadix_typed_error_order :
    engineeringRadixDecimalInputs (.error .na) (some (.number 0)) = .error .num ∧
    engineeringRadixDecimalInputs (.error .na) (some (.error .div0)) = .error .div0 ∧
    engineeringRadixDecimalInputs (.logical true) none = .error .value ∧
    engineeringRadixDecimalInputs .missingArg none = .error .na ∧
    engineeringRadixDecimalInputs (.number (-1)) (some .missingArg) = .ok (-1, none) := by native_decide

theorem engineeringRadix_parse_and_encoding_witnesses :
    engineeringRadixParse 2 10 "1111111111" = some (-1) ∧
    engineeringRadixParse 16 40 "FFFFFFFFFF" = some (-1) ∧
    engineeringRadixParse 16 40 "ff" = some 255 ∧
    engineeringRadixParse 8 30 "" = some 0 ∧
    engineeringRadixParse 2 10 " 1" = none ∧
    engineeringRadixEncode 16 40 (-1) (some 1) = some "FFFFFFFFFF" ∧
    engineeringRadixEncode 2 10 3 (some 1) = none ∧
    engineeringRadixEncode 2 10 3 (some 4) = some "0011" := by native_decide

def engineeringRadixTextMeta : FunctionMeta := {
  functionId := "FUNC.ENGINEERING_RADIX_TEXT"
  arity := { min := 1, max := 2 }
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

def engineeringRadixDecimalMeta : FunctionMeta := {
  engineeringRadixTextMeta with
    functionId := "FUNC.ENGINEERING_RADIX_DECIMAL"
    arity := Arity.exact 1
}

def dec2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2BIN" }
def dec2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2HEX" }
def dec2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.DEC2OCT" }
def bin2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.BIN2DEC" }
def bin2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.BIN2HEX" }
def bin2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.BIN2OCT" }
def hex2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.HEX2BIN" }
def hex2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.HEX2DEC" }
def hex2octMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.HEX2OCT" }
def oct2binMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.OCT2BIN" }
def oct2decMeta : FunctionMeta := { engineeringRadixDecimalMeta with functionId := "FUNC.OCT2DEC" }
def oct2hexMeta : FunctionMeta := { engineeringRadixTextMeta with functionId := "FUNC.OCT2HEX" }

theorem engineeringRadixFamily_profiles :
    dec2binMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dec2hexMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ dec2octMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ bin2decMeta.arity = Arity.exact 1
    ∧ bin2hexMeta.arity.min = 1
    ∧ bin2octMeta.arity.max = 2
    ∧ hex2binMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ hex2decMeta.arity = Arity.exact 1
    ∧ hex2octMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ oct2binMeta.hostInteraction = HostInteractionClass.none
    ∧ oct2decMeta.arity = Arity.exact 1
    ∧ oct2hexMeta.volatility = VolatilityClass.nonvolatile := by
  simp [
    engineeringRadixTextMeta,
    engineeringRadixDecimalMeta,
    dec2binMeta,
    dec2hexMeta,
    dec2octMeta,
    bin2decMeta,
    bin2hexMeta,
    bin2octMeta,
    hex2binMeta,
    hex2decMeta,
    hex2octMeta,
    oct2binMeta,
    oct2decMeta,
    oct2hexMeta
  ]

theorem engineeringRadixFamily_decimal_vs_text_arity :
    bin2decMeta.arity = Arity.exact 1
    ∧ hex2decMeta.arity = Arity.exact 1
    ∧ oct2decMeta.arity = Arity.exact 1
    ∧ dec2binMeta.arity = { min := 1, max := 2 }
    ∧ bin2hexMeta.arity = { min := 1, max := 2 }
    ∧ hex2octMeta.arity = { min := 1, max := 2 } := by
  simp [
    engineeringRadixTextMeta,
    engineeringRadixDecimalMeta,
    dec2binMeta,
    bin2decMeta,
    bin2hexMeta,
    hex2decMeta,
    hex2octMeta,
    oct2decMeta
  ]

end OxFunc.Functions
